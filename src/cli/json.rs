use crate::output::{JsonEnvelope, JsonError, print_json};

use super::args::Cli;

pub(crate) fn print_anyhow_error(cli: &Cli, err: &anyhow::Error) {
    let (code, message, details) = crate::user_error::anyhow_error_parts_for_envelope(err);
    let mut envelope = JsonEnvelope::ok(cli.command_name(), serde_json::json!({}))
        .with_command_meta(cli.command_id(), cli.command_path());
    envelope.ok = false;
    envelope.errors = vec![JsonError {
        code: code.to_string(),
        message: message.into_owned(),
        details,
    }];
    let _ = print_json(&envelope);
}
