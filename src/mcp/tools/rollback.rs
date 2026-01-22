use anyhow::Context as _;

use crate::user_error::UserError;

pub(super) async fn call_rollback_in_process(
    args: super::RollbackArgs,
) -> anyhow::Result<(String, serde_json::Value)> {
    tokio::task::spawn_blocking(move || {
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
                let envelope = crate::output::JsonEnvelope::ok("rollback", data);
                let text = serde_json::to_string_pretty(&envelope)?;
                let envelope = serde_json::to_value(&envelope)?;
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
                let envelope = super::envelope_error("rollback", &code, &message, details);
                let text = serde_json::to_string_pretty(&envelope)?;
                (text, envelope)
            }
        };

        Ok((text, envelope))
    })
    .await
    .context("mcp rollback handler task join")?
}
