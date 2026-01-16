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
