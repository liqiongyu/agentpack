pub const COMPILED_TARGETS: &[&str] = &[
    #[cfg(feature = "target-codex")]
    "codex",
    #[cfg(feature = "target-claude-code")]
    "claude_code",
    #[cfg(feature = "target-cursor")]
    "cursor",
    #[cfg(feature = "target-vscode")]
    "vscode",
    #[cfg(feature = "target-jetbrains")]
    "jetbrains",
    #[cfg(feature = "target-zed")]
    "zed",
];

pub fn is_compiled_target(target: &str) -> bool {
    COMPILED_TARGETS.iter().any(|t| t == &target)
}

pub fn allowed_target_filters() -> Vec<&'static str> {
    let mut out = Vec::with_capacity(COMPILED_TARGETS.len() + 1);
    out.push("all");
    out.extend(COMPILED_TARGETS);
    out
}
