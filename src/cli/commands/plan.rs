use crate::deploy::load_managed_paths_from_snapshot;
use crate::deploy::plan as compute_plan;
use crate::engine::Engine;
use crate::output::{JsonEnvelope, print_json};
use crate::state::latest_snapshot;

use super::Ctx;

pub(crate) fn run(ctx: &Ctx<'_>) -> anyhow::Result<()> {
    let engine = Engine::load(ctx.cli.repo.as_deref(), ctx.cli.machine.as_deref())?;
    let targets = super::super::util::selected_targets(&engine.manifest, &ctx.cli.target)?;
    let render = engine.desired_state(&ctx.cli.profile, &ctx.cli.target)?;
    let desired = render.desired;
    let mut warnings = render.warnings;
    let roots = render.roots;
    let managed_paths_from_manifest =
        crate::target_manifest::load_managed_paths_from_manifests(&roots)?;
    warnings.extend(managed_paths_from_manifest.warnings);
    let managed_paths_from_manifest = managed_paths_from_manifest.managed_paths;
    let managed_paths = if !managed_paths_from_manifest.is_empty() {
        Some(super::super::util::filter_managed(
            managed_paths_from_manifest,
            &ctx.cli.target,
        ))
    } else {
        latest_snapshot(&engine.home, &["deploy", "rollback"])?
            .as_ref()
            .map(load_managed_paths_from_snapshot)
            .transpose()?
            .map(|m| super::super::util::filter_managed(m, &ctx.cli.target))
    };

    let plan = compute_plan(&desired, managed_paths.as_ref())?;

    if ctx.cli.json {
        let mut envelope = JsonEnvelope::ok(
            "plan",
            serde_json::json!({
                "profile": ctx.cli.profile,
                "targets": targets,
                "changes": plan.changes,
                "summary": plan.summary,
            }),
        );
        envelope.warnings = warnings;
        print_json(&envelope)?;
    } else {
        for w in warnings {
            eprintln!("Warning: {w}");
        }
        println!(
            "Plan: +{} ~{} -{}",
            plan.summary.create, plan.summary.update, plan.summary.delete
        );
        for c in &plan.changes {
            println!("{:?} {} {}", c.op, c.target, c.path);
        }
    }

    Ok(())
}
