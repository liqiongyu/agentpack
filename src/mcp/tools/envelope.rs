use rmcp::model::{CallToolResult, Content};

use crate::user_error::UserError;

pub(super) fn envelope_from_anyhow_error(command: &str, err: &anyhow::Error) -> serde_json::Value {
    let user_err = crate::user_error::find_user_error(err);
    let (code, message, details) = match user_err {
        Some(user_err) => (
            user_err.code.as_str(),
            std::borrow::Cow::Borrowed(user_err.message.as_str()),
            user_err.details.clone(),
        ),
        None => (
            "E_UNEXPECTED",
            std::borrow::Cow::Owned(err.to_string()),
            None,
        ),
    };

    envelope_error(command, code, message.as_ref(), details)
}

pub(super) fn tool_result_unexpected(
    command: &str,
    message: impl std::fmt::Display,
) -> CallToolResult {
    let message = message.to_string();
    CallToolResult::structured_error(envelope_error(command, "E_UNEXPECTED", &message, None))
}

pub(super) fn envelope_error(
    command: &str,
    code: &str,
    message: &str,
    details: Option<serde_json::Value>,
) -> serde_json::Value {
    let mut err = serde_json::Map::from_iter([
        (
            "code".to_string(),
            serde_json::Value::String(code.to_string()),
        ),
        (
            "message".to_string(),
            serde_json::Value::String(message.to_string()),
        ),
    ]);
    if let Some(details) = details {
        err.insert("details".to_string(), details);
    }

    serde_json::Value::Object(serde_json::Map::from_iter([
        (
            "schema_version".to_string(),
            serde_json::Value::Number(1.into()),
        ),
        ("ok".to_string(), serde_json::Value::Bool(false)),
        (
            "command".to_string(),
            serde_json::Value::String(command.to_string()),
        ),
        (
            "version".to_string(),
            serde_json::Value::String(env!("CARGO_PKG_VERSION").to_string()),
        ),
        (
            "data".to_string(),
            serde_json::Value::Object(serde_json::Map::new()),
        ),
        ("warnings".to_string(), serde_json::Value::Array(Vec::new())),
        (
            "errors".to_string(),
            serde_json::Value::Array(vec![serde_json::Value::Object(err)]),
        ),
    ]))
}

pub(super) fn tool_result_from_envelope(
    text: String,
    envelope: serde_json::Value,
) -> CallToolResult {
    let ok = envelope
        .get("ok")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    CallToolResult {
        content: vec![Content::text(text)],
        structured_content: Some(envelope),
        is_error: Some(!ok),
        meta: None,
    }
}

pub(super) fn tool_result_from_user_error(command: &str, err: UserError) -> CallToolResult {
    CallToolResult::structured_error(envelope_error(
        command,
        &err.code,
        &err.message,
        err.details,
    ))
}
