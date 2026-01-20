use anyhow::Context as _;

use crate::deploy::TargetPath;
use crate::engine::Engine;
use crate::user_error::UserError;

#[derive(Debug)]
pub(crate) enum EvolveRestoreOutcome {
    NeedsConfirmation,
    Done(EvolveRestoreReport),
}

#[derive(Debug, Clone, serde::Serialize)]
pub(crate) struct EvolveRestoreItem {
    pub(crate) target: String,
    pub(crate) path: String,
    pub(crate) path_posix: String,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub(crate) module_ids: Vec<String>,
}

#[derive(Default, Debug, Clone, serde::Serialize)]
pub(crate) struct EvolveRestoreSummary {
    pub(crate) missing: u64,
    pub(crate) restored: u64,
    pub(crate) skipped_existing: u64,
    pub(crate) skipped_read_error: u64,
}

#[derive(Debug)]
pub(crate) struct EvolveRestoreReport {
    pub(crate) restored: Vec<EvolveRestoreItem>,
    pub(crate) summary: EvolveRestoreSummary,
    pub(crate) warnings: Vec<String>,
    pub(crate) reason: &'static str,
}

pub(crate) fn evolve_restore_in(
    engine: &Engine,
    profile: &str,
    target_filter: &str,
    module_filter: Option<&str>,
    dry_run: bool,
    confirmed: bool,
    json: bool,
) -> anyhow::Result<EvolveRestoreOutcome> {
    let render = engine.desired_state(profile, target_filter)?;
    let desired = render.desired;

    let mut warnings = render.warnings;
    let mut summary = EvolveRestoreSummary::default();
    let mut missing: Vec<(TargetPath, Vec<u8>, Vec<String>)> = Vec::new();

    for (tp, desired_file) in &desired {
        if let Some(filter) = module_filter {
            if !desired_file.module_ids.iter().any(|id| id == filter) {
                continue;
            }
        }

        match std::fs::metadata(&tp.path) {
            Ok(_) => {
                summary.skipped_existing += 1;
            }
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                summary.missing += 1;
                missing.push((
                    tp.clone(),
                    desired_file.bytes.clone(),
                    desired_file.module_ids.clone(),
                ));
            }
            Err(err) => {
                summary.skipped_read_error += 1;
                warnings.push(format!(
                    "evolve.restore: skipped {} {}: failed to stat path: {err}",
                    tp.target,
                    tp.path.display()
                ));
            }
        }
    }

    if missing.is_empty() {
        return Ok(EvolveRestoreOutcome::Done(EvolveRestoreReport {
            restored: Vec::new(),
            summary,
            warnings,
            reason: "no_missing",
        }));
    }

    if !dry_run && !confirmed {
        if json {
            return Err(UserError::confirm_required("evolve restore"));
        }
        return Ok(EvolveRestoreOutcome::NeedsConfirmation);
    }

    let mut restored: Vec<EvolveRestoreItem> = Vec::new();
    for (tp, bytes, module_ids) in missing {
        if !dry_run {
            crate::fs::write_atomic(&tp.path, &bytes)
                .with_context(|| format!("write {}", tp.path.display()))?;
            summary.restored += 1;
        }

        restored.push(EvolveRestoreItem {
            target: tp.target.clone(),
            path: tp.path.to_string_lossy().to_string(),
            path_posix: crate::paths::path_to_posix_string(&tp.path),
            module_ids,
        });
    }

    restored.sort_by(|a, b| {
        (a.target.as_str(), a.path.as_str()).cmp(&(b.target.as_str(), b.path.as_str()))
    });

    let reason = if dry_run { "dry_run" } else { "restored" };

    Ok(EvolveRestoreOutcome::Done(EvolveRestoreReport {
        restored,
        summary,
        warnings,
        reason,
    }))
}
