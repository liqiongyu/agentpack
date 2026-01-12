use crate::deploy::load_managed_paths_from_snapshot;
use crate::deploy::plan as compute_plan;
use crate::engine::Engine;
use crate::output::{JsonEnvelope, print_json};
use crate::state::latest_snapshot;
use crate::user_error::UserError;

use super::Ctx;

pub(crate) fn run(ctx: &Ctx<'_>, apply: bool, adopt: bool) -> anyhow::Result<()> {
    let engine = Engine::load(ctx.cli.repo.as_deref(), ctx.cli.machine.as_deref())?;
    let targets = super::super::util::selected_targets(&engine.manifest, &ctx.cli.target)?;
    let render = engine.desired_state(&ctx.cli.profile, &ctx.cli.target)?;
    let desired = render.desired;
    let warnings = render.warnings;
    let roots = render.roots;
    let managed_paths_from_manifest =
        crate::target_manifest::load_managed_paths_from_manifests(&roots)?;
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

    let will_apply = apply && !ctx.cli.dry_run;

    if !ctx.cli.json {
        for w in &warnings {
            eprintln!("Warning: {w}");
        }
        println!(
            "Plan: +{} ~{} -{}",
            plan.summary.create, plan.summary.update, plan.summary.delete
        );
        super::super::util::print_diff(&plan, &desired)?;
    }

    if !will_apply {
        if ctx.cli.json {
            let mut envelope = JsonEnvelope::ok(
                "deploy",
                serde_json::json!({
                    "applied": false,
                    "profile": ctx.cli.profile,
                    "targets": targets,
                    "changes": plan.changes,
                    "summary": plan.summary,
                }),
            );
            envelope.warnings = warnings;
            print_json(&envelope)?;
        }
        return Ok(());
    }

    super::super::util::require_yes_for_json_mutation(ctx.cli, "deploy --apply")?;

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

    let needs_manifests = super::super::util::manifests_missing_for_desired(&roots, &desired);

    if plan.changes.is_empty() && !needs_manifests {
        if ctx.cli.json {
            let mut envelope = JsonEnvelope::ok(
                "deploy",
                serde_json::json!({
                    "applied": false,
                    "reason": "no_changes",
                    "profile": ctx.cli.profile,
                    "targets": targets,
                    "changes": plan.changes,
                    "summary": plan.summary,
                }),
            );
            envelope.warnings = warnings;
            print_json(&envelope)?;
        } else {
            println!("No changes");
        }
        return Ok(());
    }

    if !ctx.cli.yes && !ctx.cli.json && !super::super::util::confirm("Apply changes?")? {
        println!("Aborted");
        return Ok(());
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

    if ctx.cli.json {
        let mut envelope = JsonEnvelope::ok(
            "deploy",
            serde_json::json!({
                "applied": true,
                "snapshot_id": snapshot.id,
                "profile": ctx.cli.profile,
                "targets": targets,
                "changes": plan.changes,
                "summary": plan.summary,
            }),
        );
        envelope.warnings = warnings;
        print_json(&envelope)?;
    } else {
        println!("Applied. Snapshot: {}", snapshot.id);
    }

    Ok(())
}
