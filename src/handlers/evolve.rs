use anyhow::Context as _;

use crate::config::{Module, ModuleType};
use crate::deploy::TargetPath;
use crate::engine::Engine;
use crate::user_error::UserError;
use time::macros::format_description;

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

#[derive(Debug, Clone, Copy, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum EvolveScope {
    Global,
    Machine,
    Project,
}

#[derive(Debug)]
pub(crate) enum EvolveProposeOutcome {
    NeedsConfirmation,
    Noop(EvolveProposeNoopReport),
    DryRun(EvolveProposeDryRunReport),
    Created(EvolveProposeCreatedReport),
}

#[derive(Debug)]
pub(crate) struct EvolveProposeNoopReport {
    pub(crate) reason: &'static str,
    pub(crate) summary: EvolveProposeSummary,
    pub(crate) skipped: Vec<EvolveProposeSkippedItem>,
    pub(crate) warnings: Vec<String>,
}

#[derive(Debug)]
pub(crate) struct EvolveProposeDryRunReport {
    pub(crate) summary: EvolveProposeSummary,
    pub(crate) candidates: Vec<EvolveProposeItem>,
    pub(crate) skipped: Vec<EvolveProposeSkippedItem>,
    pub(crate) warnings: Vec<String>,
}

#[derive(Debug)]
pub(crate) struct EvolveProposeCreatedReport {
    pub(crate) branch: String,
    pub(crate) scope: EvolveScope,
    pub(crate) files: Vec<String>,
    pub(crate) files_posix: Vec<String>,
    pub(crate) committed: bool,
    pub(crate) commit_warning: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub(crate) struct EvolveProposeItem {
    pub(crate) module_id: String,
    pub(crate) target: String,
    pub(crate) path: String,
    pub(crate) path_posix: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub(crate) struct EvolveProposeSuggestion {
    pub(crate) action: String,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub(crate) struct EvolveProposeSkippedItem {
    pub(crate) target: String,
    pub(crate) path: String,
    pub(crate) path_posix: String,
    pub(crate) reason: String,
    pub(crate) reason_code: String,
    pub(crate) reason_message: String,
    pub(crate) next_actions: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) module_id: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub(crate) module_ids: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub(crate) suggestions: Vec<EvolveProposeSuggestion>,
}

#[derive(Default, Debug, Clone, serde::Serialize)]
pub(crate) struct EvolveProposeSummary {
    pub(crate) drifted_proposeable: u64,
    pub(crate) drifted_skipped: u64,
    pub(crate) skipped_missing: u64,
    pub(crate) skipped_multi_module: u64,
    pub(crate) skipped_read_error: u64,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct EvolveProposeInput<'a> {
    pub(crate) profile: &'a str,
    pub(crate) target_filter: &'a str,
    pub(crate) action_prefix: &'a str,
    pub(crate) module_filter: Option<&'a str>,
    pub(crate) scope: EvolveScope,
    pub(crate) branch_override: Option<&'a str>,
    pub(crate) dry_run: bool,
    pub(crate) confirmed: bool,
    pub(crate) json: bool,
}

type MarkedSectionCandidates = Vec<(String, Vec<u8>)>;

fn try_propose_marked_instructions_sections(
    desired_bytes: &[u8],
    actual_bytes: &[u8],
    module_ids: &[String],
) -> anyhow::Result<Option<MarkedSectionCandidates>> {
    if !desired_bytes
        .windows(crate::markers::MODULE_SECTION_START_PREFIX.len())
        .any(|w| w == crate::markers::MODULE_SECTION_START_PREFIX.as_bytes())
    {
        return Ok(None);
    }
    if !actual_bytes
        .windows(crate::markers::MODULE_SECTION_START_PREFIX.len())
        .any(|w| w == crate::markers::MODULE_SECTION_START_PREFIX.as_bytes())
    {
        return Ok(None);
    }

    let desired = match crate::markers::parse_module_sections_from_bytes(desired_bytes) {
        Ok(v) => v,
        Err(_) => return Ok(None),
    };
    let actual = match crate::markers::parse_module_sections_from_bytes(actual_bytes) {
        Ok(v) => v,
        Err(_) => return Ok(None),
    };

    if module_ids
        .iter()
        .any(|id| !desired.contains_key(id) || !actual.contains_key(id))
    {
        return Ok(None);
    }

    let mut out = Vec::new();
    for module_id in module_ids {
        let Some(desired_text) = desired.get(module_id) else {
            continue;
        };
        let Some(actual_text) = actual.get(module_id) else {
            continue;
        };
        if desired_text != actual_text {
            out.push((module_id.clone(), actual_text.as_bytes().to_vec()));
        }
    }

    if out.is_empty() {
        Ok(None)
    } else {
        Ok(Some(out))
    }
}

fn evolve_propose_reason_message(reason: &str) -> String {
    match reason {
        "missing" => "expected managed output is missing on disk (use evolve.restore or deploy to recreate)".to_string(),
        "multi_module_output" => "output is produced by multiple modules and cannot be proposed safely (add markers or split outputs)".to_string(),
        _ => reason.to_string(),
    }
}

fn evolve_propose_suggestions(reason: &str) -> Vec<EvolveProposeSuggestion> {
    match reason {
        "missing" => vec![
            EvolveProposeSuggestion {
                action: "agentpack evolve restore".to_string(),
                reason: "restore missing desired outputs (create-only)".to_string(),
            },
            EvolveProposeSuggestion {
                action: "agentpack deploy --apply".to_string(),
                reason: "re-deploy desired state and write/update manifests".to_string(),
            },
        ],
        "multi_module_output" => vec![
            EvolveProposeSuggestion {
                action: "Add per-module markers to aggregated instructions outputs".to_string(),
                reason: "allow evolve.propose to map drift back to a single module".to_string(),
            },
            EvolveProposeSuggestion {
                action: "Split aggregated outputs so each file maps to one module".to_string(),
                reason: "avoid multi-module outputs that cannot be proposed safely".to_string(),
            },
        ],
        _ => Vec::new(),
    }
}

fn evolve_propose_next_actions_for_missing(
    action_prefix: &str,
    module_id: Option<&str>,
) -> Vec<String> {
    let mut actions = Vec::new();

    let mut restore = format!("{action_prefix} evolve restore");
    if let Some(module_id) = module_id {
        restore.push_str(&format!(" --module-id {module_id}"));
    }
    restore.push_str(" --yes --json");
    actions.push(restore);

    actions.push(format!("{action_prefix} deploy --apply --yes --json"));

    actions
}

#[derive(Debug, Clone, Copy)]
enum OverlayScope {
    Global,
    Machine,
    Project,
}

fn overlay_dir_for_scope(
    engine: &Engine,
    module_id: &str,
    scope: OverlayScope,
) -> std::path::PathBuf {
    let fs_key = crate::ids::module_fs_key(module_id);
    let canonical = match scope {
        OverlayScope::Global => engine.repo.repo_dir.join("overlays").join(&fs_key),
        OverlayScope::Machine => engine
            .repo
            .repo_dir
            .join("overlays/machines")
            .join(&engine.machine_id)
            .join(&fs_key),
        OverlayScope::Project => engine
            .repo
            .repo_dir
            .join("projects")
            .join(&engine.project.project_id)
            .join("overlays")
            .join(&fs_key),
    };

    let legacy_fs_key = crate::ids::module_fs_key_unbounded(module_id);
    let legacy_fs_key = (legacy_fs_key != fs_key).then(|| match scope {
        OverlayScope::Global => engine.repo.repo_dir.join("overlays").join(&legacy_fs_key),
        OverlayScope::Machine => engine
            .repo
            .repo_dir
            .join("overlays/machines")
            .join(&engine.machine_id)
            .join(&legacy_fs_key),
        OverlayScope::Project => engine
            .repo
            .repo_dir
            .join("projects")
            .join(&engine.project.project_id)
            .join("overlays")
            .join(&legacy_fs_key),
    });

    let legacy = crate::ids::is_safe_legacy_path_component(module_id).then(|| match scope {
        OverlayScope::Global => engine.repo.repo_dir.join("overlays").join(module_id),
        OverlayScope::Machine => engine
            .repo
            .repo_dir
            .join("overlays/machines")
            .join(&engine.machine_id)
            .join(module_id),
        OverlayScope::Project => engine
            .repo
            .repo_dir
            .join("projects")
            .join(&engine.project.project_id)
            .join("overlays")
            .join(module_id),
    });

    if canonical.exists() {
        canonical
    } else if legacy_fs_key.as_ref().is_some_and(|p| p.exists()) {
        legacy_fs_key.expect("legacy fs_key exists")
    } else if legacy.as_ref().is_some_and(|p| p.exists()) {
        legacy.expect("legacy exists")
    } else {
        canonical
    }
}

fn module_rel_path_for_output(
    module: &Module,
    module_id: &str,
    output: &TargetPath,
    roots: &[crate::targets::TargetRoot],
) -> Option<String> {
    match module.module_type {
        ModuleType::Instructions => Some("AGENTS.md".to_string()),
        ModuleType::Prompt | ModuleType::Command => output
            .path
            .file_name()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string()),
        ModuleType::Skill => {
            let best = crate::targets::best_root_for(roots, &output.target, &output.path)?;
            let rel = output.path.strip_prefix(&best.root).ok()?;
            let rel_str = rel.to_string_lossy().replace('\\', "/");
            let skill_name = crate::cli::util::module_name_from_id(module_id);
            let Some((first, rest)) = rel_str.split_once('/') else {
                return Some(rel_str);
            };
            if first == skill_name && !rest.is_empty() {
                Some(rest.to_string())
            } else {
                Some(rel_str)
            }
        }
    }
}

pub(crate) fn evolve_propose_in(
    engine: &Engine,
    input: EvolveProposeInput<'_>,
) -> anyhow::Result<EvolveProposeOutcome> {
    let EvolveProposeInput {
        profile,
        target_filter,
        action_prefix,
        module_filter,
        scope,
        branch_override,
        dry_run,
        confirmed,
        json,
    } = input;

    let render = engine.desired_state(profile, target_filter)?;
    let desired = render.desired;
    let roots = render.roots;

    let mut summary = EvolveProposeSummary::default();
    let mut candidates: Vec<(String, TargetPath, Vec<u8>)> = Vec::new();
    let mut instructions_sections: std::collections::BTreeMap<String, Vec<u8>> =
        std::collections::BTreeMap::new();
    let mut skipped: Vec<EvolveProposeSkippedItem> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    for (tp, desired_file) in &desired {
        if let Some(filter) = module_filter {
            if !desired_file.module_ids.iter().any(|id| id == filter) {
                continue;
            }
        }

        let actual = match std::fs::read(&tp.path) {
            Ok(bytes) => Some(bytes),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => None,
            Err(err) => {
                summary.skipped_read_error += 1;
                warnings.push(format!(
                    "evolve.propose: skipped {} {}: failed to read deployed file: {err}",
                    tp.target,
                    tp.path.display()
                ));
                continue;
            }
        };

        let is_drifted = match &actual {
            Some(bytes) => bytes != &desired_file.bytes,
            None => true,
        };
        if !is_drifted {
            continue;
        }

        if desired_file.module_ids.len() != 1 {
            match &actual {
                None => {
                    summary.drifted_skipped += 1;
                    summary.skipped_missing += 1;
                    let reason = "missing".to_string();
                    skipped.push(EvolveProposeSkippedItem {
                        target: tp.target.clone(),
                        path: tp.path.to_string_lossy().to_string(),
                        path_posix: crate::paths::path_to_posix_string(&tp.path),
                        reason_code: reason.clone(),
                        reason_message: evolve_propose_reason_message(&reason),
                        next_actions: evolve_propose_next_actions_for_missing(action_prefix, None),
                        reason,
                        module_id: None,
                        module_ids: desired_file.module_ids.clone(),
                        suggestions: evolve_propose_suggestions("missing"),
                    });
                }
                Some(actual) => {
                    if let Some(section_candidates) = try_propose_marked_instructions_sections(
                        &desired_file.bytes,
                        actual,
                        &desired_file.module_ids,
                    )? {
                        for (module_id, bytes) in section_candidates {
                            if let Some(prev) = instructions_sections.get(&module_id) {
                                if prev != &bytes {
                                    warnings.push(format!(
                                        "evolve.propose: skipped {} {} section for {}: conflicting edits across aggregated outputs",
                                        tp.target,
                                        tp.path.display(),
                                        module_id
                                    ));
                                    continue;
                                }
                                continue;
                            }

                            instructions_sections.insert(module_id.clone(), bytes.clone());
                            summary.drifted_proposeable += 1;
                            candidates.push((module_id, tp.clone(), bytes));
                        }
                        continue;
                    }

                    summary.drifted_skipped += 1;
                    summary.skipped_multi_module += 1;
                    let reason = "multi_module_output".to_string();
                    skipped.push(EvolveProposeSkippedItem {
                        target: tp.target.clone(),
                        path: tp.path.to_string_lossy().to_string(),
                        path_posix: crate::paths::path_to_posix_string(&tp.path),
                        reason_code: reason.clone(),
                        reason_message: evolve_propose_reason_message(&reason),
                        next_actions: Vec::new(),
                        reason,
                        module_id: None,
                        module_ids: desired_file.module_ids.clone(),
                        suggestions: evolve_propose_suggestions("multi_module_output"),
                    });
                }
            }
            continue;
        }

        let module_id = desired_file.module_ids[0].clone();
        match actual {
            Some(actual) => {
                summary.drifted_proposeable += 1;
                candidates.push((module_id, tp.clone(), actual));
            }
            None => {
                summary.drifted_skipped += 1;
                summary.skipped_missing += 1;
                let reason = "missing".to_string();
                skipped.push(EvolveProposeSkippedItem {
                    target: tp.target.clone(),
                    path: tp.path.to_string_lossy().to_string(),
                    path_posix: crate::paths::path_to_posix_string(&tp.path),
                    reason_code: reason.clone(),
                    reason_message: evolve_propose_reason_message(&reason),
                    next_actions: evolve_propose_next_actions_for_missing(
                        action_prefix,
                        Some(module_id.as_str()),
                    ),
                    reason,
                    module_id: Some(module_id),
                    module_ids: Vec::new(),
                    suggestions: evolve_propose_suggestions("missing"),
                });
            }
        }
    }

    let mut items: Vec<EvolveProposeItem> = candidates
        .iter()
        .map(|(module_id, tp, _)| EvolveProposeItem {
            module_id: module_id.clone(),
            target: tp.target.clone(),
            path: tp.path.to_string_lossy().to_string(),
            path_posix: crate::paths::path_to_posix_string(&tp.path),
        })
        .collect();
    items.sort_by(|a, b| {
        (a.module_id.as_str(), a.path.as_str()).cmp(&(b.module_id.as_str(), b.path.as_str()))
    });

    skipped.sort_by(|a, b| {
        (a.reason.as_str(), a.target.as_str(), a.path.as_str()).cmp(&(
            b.reason.as_str(),
            b.target.as_str(),
            b.path.as_str(),
        ))
    });

    if items.is_empty() {
        let reason = if skipped.is_empty() {
            "no_drift"
        } else {
            "no_proposeable_drift"
        };

        return Ok(EvolveProposeOutcome::Noop(EvolveProposeNoopReport {
            reason,
            summary,
            skipped,
            warnings,
        }));
    }

    if dry_run {
        return Ok(EvolveProposeOutcome::DryRun(EvolveProposeDryRunReport {
            candidates: items,
            skipped,
            summary,
            warnings,
        }));
    }

    if !confirmed {
        if json {
            return Err(UserError::confirm_required("evolve propose"));
        }
        return Ok(EvolveProposeOutcome::NeedsConfirmation);
    }

    let repo_dir = engine.repo.repo_dir.as_path();
    if !repo_dir.join(".git").exists() {
        return Err(UserError::git_repo_required("evolve propose", repo_dir));
    }

    let status = crate::git::git_in(repo_dir, &["status", "--porcelain"])?;
    if !status.trim().is_empty() {
        return Err(UserError::git_worktree_dirty("evolve propose", repo_dir));
    }

    let original = crate::git::git_in(repo_dir, &["rev-parse", "--abbrev-ref", "HEAD"])?;
    let original = original.trim().to_string();

    let branch = branch_override.map(|s| s.to_string()).unwrap_or_else(|| {
        let scope_str = match scope {
            EvolveScope::Global => "global",
            EvolveScope::Machine => "machine",
            EvolveScope::Project => "project",
        };
        let module = if let Some(filter) = module_filter {
            crate::store::sanitize_module_id(filter)
        } else {
            let mut uniq: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
            for i in &items {
                uniq.insert(i.module_id.as_str());
            }
            if uniq.len() == 1 {
                crate::store::sanitize_module_id(uniq.iter().next().copied().unwrap_or("multi"))
            } else {
                "multi".to_string()
            }
        };
        let now = time::OffsetDateTime::now_utc();
        let timestamp = now
            .format(format_description!(
                "[year][month][day]T[hour][minute][second]Z"
            ))
            .unwrap_or_else(|_| "unknown".to_string());
        let nanos = now.nanosecond();
        format!("evolve/propose-{scope_str}-{module}-{timestamp}-{nanos:09}")
    });

    crate::git::git_in(repo_dir, &["checkout", "-b", branch.as_str()])?;

    let mut touched = Vec::new();
    for (module_id, output, actual) in &candidates {
        let Some(module) = engine.manifest.modules.iter().find(|m| m.id == *module_id) else {
            continue;
        };
        let Some(module_rel) = module_rel_path_for_output(module, module_id, output, &roots) else {
            continue;
        };

        let overlay_dir = match scope {
            EvolveScope::Global => overlay_dir_for_scope(engine, module_id, OverlayScope::Global),
            EvolveScope::Machine => overlay_dir_for_scope(engine, module_id, OverlayScope::Machine),
            EvolveScope::Project => overlay_dir_for_scope(engine, module_id, OverlayScope::Project),
        };

        let dst = overlay_dir.join(&module_rel);
        if let Some(parent) = dst.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("create {}", parent.display()))?;
        }
        crate::fs::write_atomic(&dst, actual)
            .with_context(|| format!("write {}", dst.display()))?;
        touched.push(
            dst.strip_prefix(&engine.repo.repo_dir)
                .unwrap_or(&dst)
                .to_string_lossy()
                .to_string(),
        );
    }

    if touched.is_empty() {
        crate::git::git_in(repo_dir, &["checkout", original.as_str()]).ok();
        anyhow::bail!("no proposeable files (only multi-module outputs or unknown modules)");
    }

    crate::git::git_in(repo_dir, &["add", "-A"])?;

    let commit = std::process::Command::new("git")
        .current_dir(repo_dir)
        .args(["commit", "-m", "chore(evolve): propose overlay updates"])
        .output();

    let (committed, commit_warning) = match commit {
        Ok(out) if out.status.success() => (true, None),
        Ok(out) => (
            false,
            Some(format!(
                "git commit failed: {}",
                String::from_utf8_lossy(&out.stderr)
            )),
        ),
        Err(err) => (false, Some(format!("failed to run git commit: {err}"))),
    };

    if committed {
        crate::git::git_in(repo_dir, &["checkout", original.as_str()]).ok();
    }

    let files_posix: Vec<String> = touched.iter().map(|p| p.replace('\\', "/")).collect();
    Ok(EvolveProposeOutcome::Created(EvolveProposeCreatedReport {
        branch,
        scope,
        files: touched,
        files_posix,
        committed,
        commit_warning,
    }))
}
