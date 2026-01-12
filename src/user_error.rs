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
}
