use anyhow::Context as _;

pub(super) async fn call_rollback_in_process(
    args: super::RollbackArgs,
) -> anyhow::Result<(String, serde_json::Value)> {
    tokio::task::spawn_blocking(move || {
        let command_path = ["rollback"];
        let meta = super::CommandMeta {
            command: "rollback",
            command_id: "rollback",
            command_path: &command_path,
        };

        let home = crate::paths::AgentpackHome::resolve().context("resolve agentpack home")?;

        let super::RollbackArgs {
            repo: _repo_override,
            to: snapshot_id,
            yes,
        } = args;
        let result = crate::handlers::rollback::rollback(&home, &snapshot_id, true, yes);

        let (text, envelope) = match result {
            Ok(event) => {
                let data = crate::app::rollback_json::rollback_json_data(&snapshot_id, &event.id);
                let envelope = crate::output::JsonEnvelope::ok(meta.command, data)
                    .with_command_meta(meta.command_id_string(), meta.command_path_vec());
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
    .context("mcp rollback handler task join")?
}
