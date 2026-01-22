use std::time::Instant;

use rmcp::model::CallToolResult;

pub(super) async fn call_deploy_tool(
    server: &super::AgentpackMcp,
    args: super::CommonArgs,
) -> CallToolResult {
    let binding = super::ConfirmTokenBinding::from(&args);
    match super::deploy_plan_envelope_in_process(args).await {
        Ok(mut envelope) => {
            let plan_hash = match super::confirm::compute_confirm_plan_hash(&binding, &envelope) {
                Ok(v) => v,
                Err(err) => {
                    return CallToolResult::structured_error(super::envelope_error(
                        "deploy",
                        "E_UNEXPECTED",
                        &err.to_string(),
                        None,
                    ));
                }
            };

            let token = match super::confirm::generate_confirm_token() {
                Ok(v) => v,
                Err(err) => {
                    return CallToolResult::structured_error(super::envelope_error(
                        "deploy",
                        "E_UNEXPECTED",
                        &err.to_string(),
                        None,
                    ));
                }
            };

            let now = Instant::now();
            {
                let mut store = server
                    .confirm_tokens
                    .lock()
                    .unwrap_or_else(|e| e.into_inner());
                super::confirm::insert_token(
                    &mut store,
                    token.clone(),
                    binding,
                    plan_hash.clone(),
                    now,
                );
            }

            let expires_at_utc = time::OffsetDateTime::now_utc()
                + time::Duration::seconds(
                    i64::try_from(super::CONFIRM_TOKEN_TTL.as_secs()).unwrap_or(i64::MAX),
                );
            let expires_at_utc =
                match expires_at_utc.format(&time::format_description::well_known::Rfc3339) {
                    Ok(v) => v,
                    Err(err) => {
                        return CallToolResult::structured_error(super::envelope_error(
                            "deploy",
                            "E_UNEXPECTED",
                            &err.to_string(),
                            None,
                        ));
                    }
                };

            let Some(data) = envelope.get_mut("data").and_then(|v| v.as_object_mut()) else {
                return CallToolResult::structured_error(super::envelope_error(
                    "deploy",
                    "E_UNEXPECTED",
                    "agentpack deploy envelope missing data object",
                    None,
                ));
            };
            data.insert(
                "confirm_token".to_string(),
                serde_json::Value::String(token),
            );
            data.insert(
                "confirm_plan_hash".to_string(),
                serde_json::Value::String(plan_hash),
            );
            data.insert(
                "confirm_token_expires_at".to_string(),
                serde_json::Value::String(expires_at_utc),
            );

            let text = match serde_json::to_string_pretty(&envelope) {
                Ok(v) => v,
                Err(err) => {
                    return CallToolResult::structured_error(super::envelope_error(
                        "deploy",
                        "E_UNEXPECTED",
                        &err.to_string(),
                        None,
                    ));
                }
            };
            super::tool_result_from_envelope(text, envelope)
        }
        Err(err) => CallToolResult::structured_error(super::envelope_error(
            "deploy",
            "E_UNEXPECTED",
            &err.to_string(),
            None,
        )),
    }
}
