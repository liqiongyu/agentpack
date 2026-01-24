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

    pub fn git_repo_required(
        command: impl Into<String>,
        repo_dir: &std::path::Path,
    ) -> anyhow::Error {
        let command = command.into();
        anyhow::Error::new(
            Self::new(
                "E_GIT_REPO_REQUIRED",
                format!(
                    "config repo is not a git repository (required for '{command}'): {}",
                    repo_dir.display()
                ),
            )
            .with_details(serde_json::json!({
                "command": command,
                "repo": repo_dir.display().to_string(),
                "repo_posix": crate::paths::path_to_posix_string(repo_dir),
                "reason_code": "git_repo_required",
                "next_actions": ["init_git_repo", "retry_command"],
                "hint": "Initialize git in the config repo (agentpack init --git), or run the command in a git-backed config repo.",
            })),
        )
    }

    pub fn git_detached_head(
        command: impl Into<String>,
        repo_dir: &std::path::Path,
    ) -> anyhow::Error {
        let command = command.into();
        anyhow::Error::new(
            Self::new(
                "E_GIT_DETACHED_HEAD",
                format!("refusing to run '{command}' on detached HEAD"),
            )
            .with_details(serde_json::json!({
                "command": command,
                "repo": repo_dir.display().to_string(),
                "repo_posix": crate::paths::path_to_posix_string(repo_dir),
                "reason_code": "git_detached_head",
                "next_actions": ["checkout_branch", "retry_command"],
                "hint": "Check out a branch (not detached HEAD), then retry.",
            })),
        )
    }

    pub fn git_remote_missing(
        command: impl Into<String>,
        repo_dir: &std::path::Path,
        remote: &str,
    ) -> anyhow::Error {
        let command = command.into();
        anyhow::Error::new(
            Self::new(
                "E_GIT_REMOTE_MISSING",
                format!("git remote '{remote}' not found (required for '{command}')"),
            )
            .with_details(serde_json::json!({
                "command": command,
                "repo": repo_dir.display().to_string(),
                "repo_posix": crate::paths::path_to_posix_string(repo_dir),
                "remote": remote,
                "reason_code": "git_remote_missing",
                "next_actions": ["set_git_remote", "retry_command"],
                "hint": format!("Set the remote via `agentpack remote set <url> --name {remote}` (or `git remote add {remote} <url>`), then retry."),
            })),
        )
    }

    pub fn git_not_found(cwd: &std::path::Path, args: &[&str]) -> anyhow::Error {
        anyhow::Error::new(
            Self::new(
                "E_GIT_NOT_FOUND",
                "git executable not found (is git installed and on PATH?)",
            )
            .with_details(serde_json::json!({
                "cwd": cwd.display().to_string(),
                "cwd_posix": crate::paths::path_to_posix_string(cwd),
                "args": args.iter().map(|s| s.to_string()).collect::<Vec<String>>(),
                "reason_code": "git_not_found",
                "next_actions": ["install_git", "retry_command"],
                "hint": "Install git and ensure `git` is available on PATH, then retry.",
            })),
        )
    }

    pub fn git_worktree_dirty(
        command: impl Into<String>,
        repo_dir: &std::path::Path,
    ) -> anyhow::Error {
        let command = command.into();
        anyhow::Error::new(
            Self::new(
                "E_GIT_WORKTREE_DIRTY",
                format!("refusing to run '{command}' with a dirty git working tree (commit or stash first)"),
            )
            .with_details(serde_json::json!({
                "command": command,
                "repo": repo_dir.display().to_string(),
                "repo_posix": crate::paths::path_to_posix_string(repo_dir),
                "reason_code": "git_worktree_dirty",
                "next_actions": ["commit_or_stash", "retry_command"],
                "hint": "Commit or stash your changes, then retry.",
            })),
        )
    }

    pub fn confirm_required(command: impl Into<String>) -> anyhow::Error {
        let command = command.into();
        anyhow::Error::new(
            Self::new(
                "E_CONFIRM_REQUIRED",
                format!("refusing to run '{command}' in --json mode without --yes"),
            )
            .with_details(serde_json::json!({
                "command": command,
                "reason_code": "confirm_required",
                "next_actions": ["retry_with_yes"],
            })),
        )
    }

    pub fn confirm_token_required() -> Self {
        Self::new(
            "E_CONFIRM_TOKEN_REQUIRED",
            "deploy_apply requires confirm_token from the deploy tool",
        )
        .with_details(serde_json::json!({
            "reason_code": "confirm_token_required",
            "next_actions": ["call_deploy", "retry_deploy_apply"],
            "hint": "Call the deploy tool first and pass data.confirm_token to deploy_apply."
        }))
    }

    pub fn confirm_token_expired() -> Self {
        Self::new("E_CONFIRM_TOKEN_EXPIRED", "confirm_token is expired").with_details(
            serde_json::json!({
                "reason_code": "confirm_token_expired",
                "next_actions": ["call_deploy", "retry_deploy_apply"],
                "hint": "Re-run the deploy tool to obtain a fresh confirm_token.",
            }),
        )
    }

    pub fn confirm_token_mismatch() -> Self {
        Self::new(
            "E_CONFIRM_TOKEN_MISMATCH",
            "confirm_token does not match the current deploy plan",
        )
        .with_details(serde_json::json!({
            "reason_code": "confirm_token_mismatch",
            "next_actions": ["call_deploy", "retry_deploy_apply"],
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
        Some(user_err) => {
            let details = add_default_reason_code_and_next_actions(
                user_err.code.as_str(),
                user_err.details.clone(),
            );
            (
                user_err.code.as_str(),
                std::borrow::Cow::Borrowed(user_err.message.as_str()),
                details,
            )
        }
        None => (
            "E_UNEXPECTED",
            std::borrow::Cow::Owned(err.to_string()),
            None,
        ),
    }
}

fn add_default_reason_code_and_next_actions(
    code: &str,
    details: Option<serde_json::Value>,
) -> Option<serde_json::Value> {
    let (reason_code, next_actions) = match code {
        "E_CONFIG_INVALID" => (
            "config_invalid",
            serde_json::json!(["fix_config", "retry_command"]),
        ),
        "E_CONFIG_UNSUPPORTED_VERSION" => (
            "config_unsupported_version",
            serde_json::json!(["upgrade_agentpack", "fix_config_version", "retry_command"]),
        ),
        "E_OVERLAY_NOT_FOUND" => (
            "overlay_not_found",
            serde_json::json!(["run_overlay_edit", "retry_command"]),
        ),
        "E_OVERLAY_BASELINE_MISSING" => (
            "overlay_baseline_missing",
            serde_json::json!(["run_overlay_edit", "retry_command"]),
        ),
        "E_OVERLAY_BASELINE_UNSUPPORTED" => (
            "overlay_baseline_unsupported",
            serde_json::json!([
                "init_git_repo",
                "commit_or_stash",
                "run_overlay_edit",
                "retry_command"
            ]),
        ),
        _ => return details,
    };

    match details {
        None => Some(serde_json::json!({
            "reason_code": reason_code,
            "next_actions": next_actions,
        })),
        Some(serde_json::Value::Object(mut map)) => {
            map.entry("reason_code".to_string())
                .or_insert_with(|| serde_json::Value::String(reason_code.to_string()));
            map.entry("next_actions".to_string())
                .or_insert_with(|| next_actions);
            Some(serde_json::Value::Object(map))
        }
        Some(other) => Some(other),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Context;

    #[test]
    fn find_user_error_finds_wrapped_user_error() {
        let base = anyhow::Error::new(
            UserError::new("E_CONFIRM_REQUIRED", "hello")
                .with_details(serde_json::json!({ "k": "v" })),
        );
        let wrapped: anyhow::Error = Err::<(), _>(base).context("outer context").unwrap_err();

        let user_err = find_user_error(&wrapped).expect("expected UserError in chain");
        assert_eq!(user_err.code, "E_CONFIRM_REQUIRED");
        assert_eq!(user_err.message, "hello");
        assert_eq!(user_err.details.as_ref().unwrap()["k"], "v");
    }

    #[test]
    fn anyhow_error_parts_for_envelope_uses_user_error_when_present() {
        let base = anyhow::Error::new(
            UserError::new("E_CONFIRM_REQUIRED", "hello")
                .with_details(serde_json::json!({ "k": "v" })),
        );
        let wrapped: anyhow::Error = Err::<(), _>(base).context("outer context").unwrap_err();

        let (code, message, details) = anyhow_error_parts_for_envelope(&wrapped);
        assert_eq!(code, "E_CONFIRM_REQUIRED");
        assert_eq!(message.as_ref(), "hello");
        assert_eq!(details.unwrap()["k"], "v");
    }

    #[test]
    fn anyhow_error_parts_for_envelope_falls_back_to_unexpected_for_non_user_error() {
        let err = anyhow::anyhow!("boom");

        let (code, message, details) = anyhow_error_parts_for_envelope(&err);
        assert_eq!(code, "E_UNEXPECTED");
        assert_eq!(message.as_ref(), "boom");
        assert!(details.is_none());
    }
}
