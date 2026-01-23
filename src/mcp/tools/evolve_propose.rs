use anyhow::Context as _;

use crate::user_error::UserError;

pub(super) async fn call_evolve_propose_in_process(
    args: super::EvolveProposeArgs,
) -> anyhow::Result<(String, serde_json::Value)> {
    tokio::task::spawn_blocking(move || {
        let command_path = ["evolve", "propose"];
        let meta = super::CommandMeta {
            command: "evolve.propose",
            command_id: "evolve propose",
            command_path: &command_path,
        };

        let repo_override = args.common.repo.as_ref().map(std::path::PathBuf::from);
        let profile = args.common.profile.as_deref().unwrap_or("default");
        let target = args.common.target.as_deref().unwrap_or("all");
        let machine_override = args.common.machine.as_deref();
        let dry_run = args.common.dry_run.unwrap_or(false);

        let action_prefix = {
            let mut out = String::from("agentpack");
            if let Some(repo) = args.common.repo.as_deref() {
                out.push_str(&format!(" --repo {repo}"));
            }
            if profile != "default" {
                out.push_str(&format!(" --profile {profile}"));
            }
            if target != "all" {
                out.push_str(&format!(" --target {target}"));
            }
            if let Some(machine) = machine_override {
                out.push_str(&format!(" --machine {machine}"));
            }
            out
        };

        let scope = match args.scope.unwrap_or(super::EvolveScopeArg::Global) {
            super::EvolveScopeArg::Global => crate::handlers::evolve::EvolveScope::Global,
            super::EvolveScopeArg::Machine => crate::handlers::evolve::EvolveScope::Machine,
            super::EvolveScopeArg::Project => crate::handlers::evolve::EvolveScope::Project,
        };

        let engine = match crate::engine::Engine::load(repo_override.as_deref(), machine_override) {
            Ok(v) => v,
            Err(err) => {
                let envelope = super::envelope_from_anyhow_error(meta, &err);
                let text = serde_json::to_string_pretty(&envelope)?;
                return Ok((text, envelope));
            }
        };

        let result = crate::handlers::evolve::evolve_propose_in(
            &engine,
            crate::handlers::evolve::EvolveProposeInput {
                profile,
                target_filter: target,
                action_prefix: &action_prefix,
                module_filter: args.module_id.as_deref(),
                scope,
                branch_override: args.branch.as_deref(),
                dry_run,
                confirmed: args.yes,
                json: true,
            },
        );

        let (text, envelope) = match result {
            Ok(crate::handlers::evolve::EvolveProposeOutcome::Noop(report)) => {
                let data = crate::app::evolve_propose_json::evolve_propose_json_data_noop(
                    report.reason,
                    report.summary,
                    report.skipped,
                );
                let mut envelope = crate::output::JsonEnvelope::ok(meta.command, data)
                    .with_command_meta(meta.command_id_string(), meta.command_path_vec());
                envelope.warnings = report.warnings;
                let text = serde_json::to_string_pretty(&envelope)?;
                let envelope = serde_json::to_value(&envelope)?;
                (text, envelope)
            }
            Ok(crate::handlers::evolve::EvolveProposeOutcome::DryRun(report)) => {
                let data = crate::app::evolve_propose_json::evolve_propose_json_data_dry_run(
                    report.candidates,
                    report.skipped,
                    report.summary,
                );
                let mut envelope = crate::output::JsonEnvelope::ok(meta.command, data)
                    .with_command_meta(meta.command_id_string(), meta.command_path_vec());
                envelope.warnings = report.warnings;
                let text = serde_json::to_string_pretty(&envelope)?;
                let envelope = serde_json::to_value(&envelope)?;
                (text, envelope)
            }
            Ok(crate::handlers::evolve::EvolveProposeOutcome::Created(report)) => {
                let data = crate::app::evolve_propose_json::evolve_propose_json_data_created(
                    report.branch,
                    report.scope,
                    report.files,
                    report.files_posix,
                    report.committed,
                );
                let envelope = crate::output::JsonEnvelope::ok(meta.command, data)
                    .with_command_meta(meta.command_id_string(), meta.command_path_vec());
                let text = serde_json::to_string_pretty(&envelope)?;
                let envelope = serde_json::to_value(&envelope)?;
                (text, envelope)
            }
            Ok(crate::handlers::evolve::EvolveProposeOutcome::NeedsConfirmation) => {
                let err = UserError::confirm_required("evolve propose");
                let envelope = super::envelope_from_anyhow_error(meta, &err);
                let text = serde_json::to_string_pretty(&envelope)?;
                (text, envelope)
            }
            Err(err) => {
                let envelope = super::envelope_from_anyhow_error(meta, &err);
                let text = serde_json::to_string_pretty(&envelope)?;
                (text, envelope)
            }
        };

        Ok((text, envelope))
    })
    .await
    .context("mcp evolve_propose handler task join")?
}
