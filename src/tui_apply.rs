use std::path::Path;

use crate::deploy::load_managed_paths_from_snapshot;
use crate::deploy::plan as compute_plan;
use crate::engine::Engine;
use crate::state::latest_snapshot;
use crate::targets::TargetRoot;
use crate::user_error::UserError;

#[derive(Debug)]
pub enum ApplyOutcome {
    Applied { snapshot_id: String },
    NoChanges,
}

pub fn apply_from_tui(
    repo_override: Option<&Path>,
    machine_override: Option<&str>,
    profile: &str,
    target_filter: &str,
    adopt: bool,
    confirmed: bool,
) -> anyhow::Result<ApplyOutcome> {
    if !confirmed {
        return Err(anyhow::Error::new(UserError::new(
            "E_CONFIRM_REQUIRED",
            "refusing to apply without explicit confirmation",
        )));
    }

    let engine = Engine::load(repo_override, machine_override)?;
    apply_from_tui_in(&engine, profile, target_filter, adopt, confirmed)
}

pub fn apply_from_tui_in(
    engine: &Engine,
    profile: &str,
    target_filter: &str,
    adopt: bool,
    confirmed: bool,
) -> anyhow::Result<ApplyOutcome> {
    if !confirmed {
        return Err(anyhow::Error::new(UserError::new(
            "E_CONFIRM_REQUIRED",
            "refusing to apply without explicit confirmation",
        )));
    }

    let _targets = crate::target_selection::selected_targets(&engine.manifest, target_filter)?;
    let render = engine.desired_state(profile, target_filter)?;
    let desired = render.desired;
    let mut warnings = render.warnings;
    let roots = render.roots;
    let managed_paths_from_manifest =
        crate::target_manifest::load_managed_paths_from_manifests(&roots)?;
    warnings.extend(managed_paths_from_manifest.warnings);
    let managed_paths_from_manifest = managed_paths_from_manifest.managed_paths;
    let managed_paths = if !managed_paths_from_manifest.is_empty() {
        Some(filter_managed(managed_paths_from_manifest, target_filter))
    } else {
        latest_snapshot(&engine.home, &["deploy", "rollback"])?
            .as_ref()
            .map(load_managed_paths_from_snapshot)
            .transpose()?
            .map(|m| filter_managed(m, target_filter))
    };
    let plan = compute_plan(&desired, managed_paths.as_ref())?;

    let adopt_updates: Vec<&crate::deploy::PlanChange> = plan
        .changes
        .iter()
        .filter(|c| matches!(c.update_kind, Some(crate::deploy::UpdateKind::AdoptUpdate)))
        .collect();
    if !adopt_updates.is_empty() && !adopt {
        let mut sample_paths: Vec<String> = adopt_updates.iter().map(|c| c.path.clone()).collect();
        sample_paths.sort();
        sample_paths.truncate(20);

        return Err(anyhow::Error::new(
            UserError::new(
                "E_ADOPT_CONFIRM_REQUIRED",
                "refusing to overwrite unmanaged existing files without --adopt",
            )
            .with_details(serde_json::json!({
                "flag": "--adopt",
                "adopt_updates": adopt_updates.len(),
                "sample_paths": sample_paths,
            })),
        ));
    }

    let needs_manifests = manifests_missing_for_desired(&roots, &desired);

    if plan.changes.is_empty() && !needs_manifests {
        return Ok(ApplyOutcome::NoChanges);
    }

    let lockfile_path = if engine.repo.lockfile_path.exists() {
        Some(engine.repo.lockfile_path.as_path())
    } else {
        None
    };
    let snapshot = crate::apply::apply_plan(
        &engine.home,
        "deploy",
        &plan,
        &desired,
        lockfile_path,
        &roots,
    )?;

    Ok(ApplyOutcome::Applied {
        snapshot_id: snapshot.id,
    })
}

fn filter_managed(
    managed: crate::deploy::ManagedPaths,
    target_filter: &str,
) -> crate::deploy::ManagedPaths {
    managed
        .into_iter()
        .filter(|tp| target_filter == "all" || tp.target == target_filter)
        .collect()
}

fn best_root_idx(roots: &[TargetRoot], target: &str, path: &Path) -> Option<usize> {
    roots
        .iter()
        .enumerate()
        .filter(|(_, r)| r.target == target)
        .filter(|(_, r)| path.strip_prefix(&r.root).is_ok())
        .max_by_key(|(_, r)| r.root.components().count())
        .map(|(idx, _)| idx)
}

fn manifests_missing_for_desired(
    roots: &[TargetRoot],
    desired: &crate::deploy::DesiredState,
) -> bool {
    if roots.is_empty() {
        return false;
    }

    let mut used: Vec<bool> = vec![false; roots.len()];
    for tp in desired.keys() {
        if let Some(idx) = best_root_idx(roots, &tp.target, &tp.path) {
            used[idx] = true;
        }
    }

    for (idx, root) in roots.iter().enumerate() {
        if !used[idx] {
            continue;
        }

        let preferred = crate::target_manifest::manifest_path_for_target(&root.root, &root.target);
        if preferred.exists() {
            continue;
        }

        let legacy = crate::target_manifest::legacy_manifest_path(&root.root);
        if !legacy.exists() {
            return true;
        }

        let (manifest, _warnings) =
            crate::target_manifest::read_target_manifest_soft(&legacy, &root.target);
        if manifest.is_none() {
            return true;
        }
    }

    false
}
