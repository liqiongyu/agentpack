use anyhow::Context as _;

use crate::user_error::UserError;

pub(super) async fn deploy_plan_envelope_in_process(
    args: super::CommonArgs,
) -> anyhow::Result<serde_json::Value> {
    tokio::task::spawn_blocking(move || {
        let repo_override = args.repo.as_ref().map(std::path::PathBuf::from);
        let profile = args.profile.as_deref().unwrap_or("default");
        let target = args.target.as_deref().unwrap_or("all");
        let machine_override = args.machine.as_deref();

        let result = crate::handlers::read_only::read_only_context(
            repo_override.as_deref(),
            machine_override,
            profile,
            target,
        );

        match result {
            Ok(crate::handlers::read_only::ReadOnlyContext {
                targets,
                plan,
                warnings,
                ..
            }) => {
                let data =
                    crate::app::deploy_json::deploy_json_data_dry_run(profile, targets, plan);
                let mut envelope = crate::output::JsonEnvelope::ok("deploy", data);
                envelope.warnings = warnings;
                serde_json::to_value(&envelope).context("serialize deploy envelope")
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
                Ok(super::envelope_error("deploy", &code, &message, details))
            }
        }
    })
    .await
    .context("mcp deploy handler task join")?
}
