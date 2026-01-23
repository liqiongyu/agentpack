use rmcp::model::{CallToolResult, Content};

use crate::user_error::UserError;

#[derive(Debug, Clone, Copy)]
pub(super) struct CommandMeta<'a> {
    pub command: &'a str,
    pub command_id: &'a str,
    pub command_path: &'a [&'a str],
}

impl<'a> CommandMeta<'a> {
    pub(super) fn command_id_string(self) -> String {
        self.command_id.to_string()
    }

    pub(super) fn command_path_vec(self) -> Vec<String> {
        self.command_path.iter().map(|s| (*s).to_string()).collect()
    }
}

pub(super) fn envelope_from_anyhow_error(
    meta: CommandMeta<'_>,
    err: &anyhow::Error,
) -> serde_json::Value {
    let (code, message, details) = crate::user_error::anyhow_error_parts_for_envelope(err);

    envelope_error(meta, code, message.as_ref(), details)
}

pub(super) fn tool_result_unexpected(
    meta: CommandMeta<'_>,
    message: impl std::fmt::Display,
) -> CallToolResult {
    let message = message.to_string();
    CallToolResult::structured_error(envelope_error(meta, "E_UNEXPECTED", &message, None))
}

pub(super) fn envelope_error(
    meta: CommandMeta<'_>,
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
            serde_json::Value::String(meta.command.to_string()),
        ),
        (
            "command_id".to_string(),
            serde_json::Value::String(meta.command_id.to_string()),
        ),
        (
            "command_path".to_string(),
            serde_json::Value::Array(
                meta.command_path
                    .iter()
                    .map(|s| serde_json::Value::String((*s).to_string()))
                    .collect(),
            ),
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

pub(super) fn tool_result_from_user_error(meta: CommandMeta<'_>, err: UserError) -> CallToolResult {
    CallToolResult::structured_error(envelope_error(meta, &err.code, &err.message, err.details))
}
