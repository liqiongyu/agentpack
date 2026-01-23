use anyhow::Context as _;

pub(super) async fn deploy_plan_envelope_in_process(
    args: super::CommonArgs,
) -> anyhow::Result<serde_json::Value> {
    tokio::task::spawn_blocking(move || {
        let command_path = ["deploy"];
        let meta = super::CommandMeta {
            command: "deploy",
            command_id: "deploy",
            command_path: &command_path,
        };

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
                let mut envelope = crate::output::JsonEnvelope::ok(meta.command, data)
                    .with_command_meta(meta.command_id_string(), meta.command_path_vec());
                envelope.warnings = warnings;
                serde_json::to_value(&envelope).context("serialize deploy envelope")
            }
            Err(err) => Ok(super::envelope_from_anyhow_error(meta, &err)),
        }
    })
    .await
    .context("mcp deploy handler task join")?
}
