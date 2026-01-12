use crate::output::{JsonEnvelope, JsonError, print_json};
use crate::user_error::UserError;

use super::args::Cli;

pub(crate) fn print_anyhow_error(cli: &Cli, err: &anyhow::Error) {
    let user_err = err.chain().find_map(|e| e.downcast_ref::<UserError>());
    let envelope = JsonEnvelope::<serde_json::Value>::err(
        cli.command_name(),
        vec![JsonError {
            code: user_err
                .map(|e| e.code.clone())
                .unwrap_or_else(|| "E_UNEXPECTED".to_string()),
            message: user_err
                .map(|e| e.message.clone())
                .unwrap_or_else(|| err.to_string()),
            details: user_err.and_then(|e| e.details.clone()),
        }],
    );
    let _ = print_json(&envelope);
}
