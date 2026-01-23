#[derive(Debug)]
pub struct UserError {
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

impl std::fmt::Display for UserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for UserError {}

impl UserError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            details: None,
        }
    }

    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }

    pub fn confirm_required(command: impl Into<String>) -> anyhow::Error {
        let command = command.into();
        anyhow::Error::new(
            Self::new(
                "E_CONFIRM_REQUIRED",
                format!("refusing to run '{command}' in --json mode without --yes"),
            )
            .with_details(serde_json::json!({ "command": command })),
        )
    }

    pub fn confirm_token_required() -> Self {
        Self::new(
            "E_CONFIRM_TOKEN_REQUIRED",
            "deploy_apply requires confirm_token from the deploy tool",
        )
        .with_details(serde_json::json!({
            "hint": "Call the deploy tool first and pass data.confirm_token to deploy_apply."
        }))
    }

    pub fn confirm_token_expired() -> Self {
        Self::new("E_CONFIRM_TOKEN_EXPIRED", "confirm_token is expired").with_details(
            serde_json::json!({ "hint": "Re-run the deploy tool to obtain a fresh confirm_token." }),
        )
    }

    pub fn confirm_token_mismatch() -> Self {
        Self::new(
            "E_CONFIRM_TOKEN_MISMATCH",
            "confirm_token does not match the current deploy plan",
        )
        .with_details(serde_json::json!({
            "hint": "Re-run the deploy tool and ensure the apply uses the matching confirm_token."
        }))
    }
}

pub(crate) fn find_user_error(err: &anyhow::Error) -> Option<&UserError> {
    err.chain().find_map(|e| e.downcast_ref::<UserError>())
}

pub(crate) fn anyhow_error_parts_for_envelope(
    err: &anyhow::Error,
) -> (
    &'_ str,
    std::borrow::Cow<'_, str>,
    Option<serde_json::Value>,
) {
    let user_err = find_user_error(err);
    match user_err {
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
    }
}
