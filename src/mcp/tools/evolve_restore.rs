use anyhow::Context as _;

use crate::user_error::UserError;

pub(super) async fn call_evolve_restore_in_process(
    args: super::EvolveRestoreArgs,
) -> anyhow::Result<(String, serde_json::Value)> {
    tokio::task::spawn_blocking(move || {
        let command_path = ["evolve", "restore"];
        let meta = super::CommandMeta {
            command: "evolve.restore",
            command_id: "evolve restore",
            command_path: &command_path,
        };

        let repo_override = args.common.repo.as_ref().map(std::path::PathBuf::from);
        let profile = args.common.profile.as_deref().unwrap_or("default");
        let target = args.common.target.as_deref().unwrap_or("all");
        let machine_override = args.common.machine.as_deref();
        let dry_run = args.common.dry_run.unwrap_or(false);

        let engine = match crate::engine::Engine::load(repo_override.as_deref(), machine_override) {
            Ok(v) => v,
            Err(err) => {
                let envelope = super::envelope_from_anyhow_error(meta, &err);
                let text = serde_json::to_string_pretty(&envelope)?;
                return Ok((text, envelope));
            }
        };

        let result = crate::handlers::evolve::evolve_restore_in(
            &engine,
            profile,
            target,
            args.module_id.as_deref(),
            dry_run,
            args.yes,
            true,
        );

        let (text, envelope) = match result {
            Ok(crate::handlers::evolve::EvolveRestoreOutcome::Done(report)) => {
                let data = crate::app::evolve_restore_json::evolve_restore_json_data(
                    report.restored,
                    report.summary,
                    report.reason,
                );
                let mut envelope = crate::output::JsonEnvelope::ok(meta.command, data)
                    .with_command_meta(meta.command_id_string(), meta.command_path_vec());
                envelope.warnings = report.warnings;
                let text = serde_json::to_string_pretty(&envelope)?;
                let envelope = serde_json::to_value(&envelope)?;
                (text, envelope)
            }
            Ok(crate::handlers::evolve::EvolveRestoreOutcome::NeedsConfirmation) => {
                let err = UserError::confirm_required("evolve restore");
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
    .context("mcp evolve_restore handler task join")?
}
