use anyhow::Context as _;

pub(super) async fn call_preview_in_process(
    args: super::PreviewArgs,
) -> anyhow::Result<(String, serde_json::Value)> {
    tokio::task::spawn_blocking(move || {
        let command_path = ["preview"];
        let meta = super::CommandMeta {
            command: "preview",
            command_id: "preview",
            command_path: &command_path,
        };

        let repo_override = args.common.repo.as_ref().map(std::path::PathBuf::from);
        let profile = args.common.profile.as_deref().unwrap_or("default");
        let target = args.common.target.as_deref().unwrap_or("all");
        let machine_override = args.common.machine.as_deref();

        let result = crate::handlers::read_only::read_only_context(
            repo_override.as_deref(),
            machine_override,
            profile,
            target,
        );

        let (text, envelope) = match result {
            Ok(crate::handlers::read_only::ReadOnlyContext {
                targets,
                desired,
                plan,
                mut warnings,
                roots,
            }) => {
                let data = crate::app::preview_json::preview_json_data(
                    profile,
                    targets,
                    plan,
                    desired,
                    roots,
                    args.diff,
                    &mut warnings,
                )?;

                let mut envelope = crate::output::JsonEnvelope::ok(meta.command, data)
                    .with_command_meta(meta.command_id_string(), meta.command_path_vec());
                envelope.warnings = warnings;

                let text = serde_json::to_string_pretty(&envelope)?;
                let envelope = serde_json::to_value(&envelope)?;
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
    .context("mcp preview handler task join")?
}
