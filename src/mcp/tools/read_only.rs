use anyhow::Context as _;

pub(super) async fn call_read_only_in_process(
    command: &'static str,
    args: super::CommonArgs,
) -> anyhow::Result<(String, serde_json::Value)> {
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

        let (text, envelope) = match result {
            Ok(crate::handlers::read_only::ReadOnlyContext {
                targets,
                plan,
                warnings,
                ..
            }) => {
                let data = crate::app::plan_json::plan_json_data(profile, targets, plan);
                let mut envelope = crate::output::JsonEnvelope::ok(command, data);
                envelope.warnings = warnings;
                let text = serde_json::to_string_pretty(&envelope)?;
                let envelope = serde_json::to_value(&envelope)?;
                (text, envelope)
            }
            Err(err) => {
                let envelope = super::envelope_from_anyhow_error(command, &err);
                let text = serde_json::to_string_pretty(&envelope)?;
                (text, envelope)
            }
        };

        Ok((text, envelope))
    })
    .await
    .context("mcp read-only handler task join")?
}
