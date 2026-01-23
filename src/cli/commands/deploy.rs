use crate::app::deploy_json::{
    deploy_json_data_applied, deploy_json_data_dry_run, deploy_json_data_no_changes,
};
use crate::engine::Engine;
use crate::handlers::deploy::{ConfirmationStyle, DeployApplyOutcome, deploy_apply_in};
use crate::handlers::read_only::read_only_context_in;
use crate::output::{JsonEnvelope, print_json};

use super::Ctx;

pub(crate) fn run(ctx: &Ctx<'_>, apply: bool, adopt: bool) -> anyhow::Result<()> {
    let engine = Engine::load(ctx.cli.repo.as_deref(), ctx.cli.machine.as_deref())?;
    let crate::handlers::read_only::ReadOnlyContext {
        targets,
        desired,
        plan,
        warnings,
        roots,
    } = read_only_context_in(&engine, &ctx.cli.profile, &ctx.cli.target)?;

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
            let data = deploy_json_data_dry_run(ctx.cli.profile.as_str(), targets, plan);
            let mut envelope = JsonEnvelope::ok("deploy", data)
                .with_command_meta(ctx.cli.command_id(), ctx.cli.command_path());
            envelope.warnings = warnings;
            print_json(&envelope)?;
        }
        return Ok(());
    }

    let mut outcome = deploy_apply_in(
        &engine,
        &plan,
        &desired,
        &roots,
        adopt,
        ctx.cli.yes,
        if ctx.cli.json {
            ConfirmationStyle::JsonYes {
                command_id: "deploy --apply",
            }
        } else {
            ConfirmationStyle::Interactive
        },
    )?;

    if matches!(outcome, DeployApplyOutcome::NeedsConfirmation) {
        if !super::super::util::confirm("Apply changes?")? {
            println!("Aborted");
            return Ok(());
        }
        outcome = deploy_apply_in(
            &engine,
            &plan,
            &desired,
            &roots,
            adopt,
            true,
            ConfirmationStyle::Interactive,
        )?;
    }

    match outcome {
        DeployApplyOutcome::NoChanges => {
            if ctx.cli.json {
                let data = deploy_json_data_no_changes(ctx.cli.profile.as_str(), targets, plan);
                let mut envelope = JsonEnvelope::ok("deploy", data)
                    .with_command_meta(ctx.cli.command_id(), ctx.cli.command_path());
                envelope.warnings = warnings;
                print_json(&envelope)?;
            } else {
                println!("No changes");
            }
            Ok(())
        }
        DeployApplyOutcome::Applied { snapshot_id } => {
            if ctx.cli.json {
                let data =
                    deploy_json_data_applied(ctx.cli.profile.as_str(), targets, plan, snapshot_id);
                let mut envelope = JsonEnvelope::ok("deploy", data)
                    .with_command_meta(ctx.cli.command_id(), ctx.cli.command_path());
                envelope.warnings = warnings;
                print_json(&envelope)?;
            } else {
                println!("Applied. Snapshot: {snapshot_id}");
            }
            Ok(())
        }
        DeployApplyOutcome::NeedsConfirmation => {
            anyhow::bail!("deploy apply requires confirmation, but confirmation was not provided")
        }
    }
}
