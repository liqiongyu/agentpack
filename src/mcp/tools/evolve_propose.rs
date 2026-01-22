use anyhow::Context as _;

use crate::user_error::UserError;

pub(super) async fn call_evolve_propose_in_process(
    args: super::EvolveProposeArgs,
) -> anyhow::Result<(String, serde_json::Value)> {
    tokio::task::spawn_blocking(move || {
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
                let user_err = err.chain().find_map(|e| e.downcast_ref::<UserError>());
                let code = user_err
                    .map(|e| e.code.clone())
                    .unwrap_or_else(|| "E_UNEXPECTED".to_string());
                let message = user_err
                    .map(|e| e.message.clone())
                    .unwrap_or_else(|| err.to_string());
                let details = user_err.and_then(|e| e.details.clone());
                let envelope = super::envelope_error("evolve.propose", &code, &message, details);
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
                let mut envelope = crate::output::JsonEnvelope::ok("evolve.propose", data);
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
                let mut envelope = crate::output::JsonEnvelope::ok("evolve.propose", data);
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
                let envelope = crate::output::JsonEnvelope::ok("evolve.propose", data);
                let text = serde_json::to_string_pretty(&envelope)?;
                let envelope = serde_json::to_value(&envelope)?;
                (text, envelope)
            }
            Ok(crate::handlers::evolve::EvolveProposeOutcome::NeedsConfirmation) => {
                let err = UserError::confirm_required("evolve propose");
                let user_err = err.chain().find_map(|e| e.downcast_ref::<UserError>());
                let code = user_err
                    .map(|e| e.code.clone())
                    .unwrap_or_else(|| "E_UNEXPECTED".to_string());
                let message = user_err
                    .map(|e| e.message.clone())
                    .unwrap_or_else(|| err.to_string());
                let details = user_err.and_then(|e| e.details.clone());
                let envelope = super::envelope_error("evolve.propose", &code, &message, details);
                let text = serde_json::to_string_pretty(&envelope)?;
                (text, envelope)
            }
            Err(err) => {
                let user_err = err.chain().find_map(|e| e.downcast_ref::<UserError>());
                let code = user_err
                    .map(|e| e.code.clone())
                    .unwrap_or_else(|| "E_UNEXPECTED".to_string());
                let message = user_err
                    .map(|e| e.message.clone())
                    .unwrap_or_else(|| err.to_string());
                let details = user_err.and_then(|e| e.details.clone());
                let envelope = super::envelope_error("evolve.propose", &code, &message, details);
                let text = serde_json::to_string_pretty(&envelope)?;
                (text, envelope)
            }
        };

        Ok((text, envelope))
    })
    .await
    .context("mcp evolve_propose handler task join")?
}
