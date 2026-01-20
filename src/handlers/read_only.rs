use std::path::Path;

use crate::deploy::ManagedPaths;
use crate::deploy::load_managed_paths_from_snapshot;
use crate::deploy::plan as compute_plan;
use crate::engine::Engine;
use crate::state::latest_snapshot;
use crate::targets::TargetRoot;

#[derive(Debug)]
pub(crate) struct ReadOnlyContext {
    pub(crate) targets: Vec<String>,
    pub(crate) desired: crate::deploy::DesiredState,
    pub(crate) plan: crate::deploy::PlanResult,
    pub(crate) warnings: Vec<String>,
    pub(crate) roots: Vec<TargetRoot>,
}

pub(crate) fn read_only_context(
    repo_override: Option<&Path>,
    machine_override: Option<&str>,
    profile: &str,
    target_filter: &str,
) -> anyhow::Result<ReadOnlyContext> {
    let engine = Engine::load(repo_override, machine_override)?;
    read_only_context_in(&engine, profile, target_filter)
}

pub(crate) fn read_only_context_in(
    engine: &Engine,
    profile: &str,
    target_filter: &str,
) -> anyhow::Result<ReadOnlyContext> {
    let targets = crate::target_selection::selected_targets(&engine.manifest, target_filter)?;
    let render = engine.desired_state(profile, target_filter)?;
    let desired = render.desired;
    let mut warnings = render.warnings;
    let roots = render.roots;

    let managed_paths = managed_paths_for_plan(engine, &roots, target_filter, &mut warnings)?;
    let plan = compute_plan(&desired, managed_paths.as_ref())?;

    Ok(ReadOnlyContext {
        targets,
        desired,
        plan,
        warnings,
        roots,
    })
}

fn managed_paths_for_plan(
    engine: &Engine,
    roots: &[TargetRoot],
    target_filter: &str,
    warnings: &mut Vec<String>,
) -> anyhow::Result<Option<ManagedPaths>> {
    let managed_paths_from_manifest =
        crate::target_manifest::load_managed_paths_from_manifests(roots)?;
    warnings.extend(managed_paths_from_manifest.warnings);
    let managed_paths_from_manifest = managed_paths_from_manifest.managed_paths;

    if !managed_paths_from_manifest.is_empty() {
        return Ok(Some(filter_managed(
            managed_paths_from_manifest,
            target_filter,
        )));
    }

    let latest = latest_snapshot(&engine.home, &["deploy", "rollback"])?;
    Ok(latest
        .as_ref()
        .map(load_managed_paths_from_snapshot)
        .transpose()?
        .map(|m| filter_managed(m, target_filter)))
}

fn filter_managed(managed: ManagedPaths, target_filter: &str) -> ManagedPaths {
    managed
        .into_iter()
        .filter(|tp| target_filter == "all" || tp.target == target_filter)
        .collect()
}
