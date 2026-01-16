use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::Context as _;
use serde::Serialize;
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize)]
pub struct PolicyLintIssue {
    pub rule: String,
    pub path: String,
    pub path_posix: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PolicyLintSummary {
    pub violations: usize,
    pub files_scanned: usize,
    pub skill_files: usize,
    pub claude_command_files: usize,
    pub rules: BTreeMap<String, usize>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PolicyLintReport {
    pub root: String,
    pub root_posix: String,
    pub issues: Vec<PolicyLintIssue>,
    pub summary: PolicyLintSummary,
}

const CLAUDE_COMMAND_DIRS: &[&str] = &[
    ".claude/commands",
    "templates/claude/commands",
    "modules/claude-commands",
];

const IGNORED_DIR_NAMES: &[&str] = &[".agentpack", ".git", "node_modules", "target"];

pub fn lint(root: &Path) -> anyhow::Result<PolicyLintReport> {
    let root_str = root.to_string_lossy().to_string();
    if !root.exists() {
        anyhow::bail!("policy lint root does not exist: {root_str}");
    }

    let mut issues = Vec::new();

    let skill_files = find_skill_files(root).context("find SKILL.md files")?;
    for path in &skill_files {
        lint_skill_file(root, path, &mut issues);
    }

    let claude_command_files = find_claude_command_files(root);
    for path in &claude_command_files {
        lint_claude_command_file(root, path, &mut issues);
    }

    let cfg = lint_org_config(root, &mut issues);
    if let Some(cfg) = cfg.as_ref() {
        lint_policy_pack_lock(root, cfg, &mut issues);
        lint_distribution_policy(root, cfg, &mut issues);
        lint_supply_chain_policy(root, cfg, &mut issues);
    }

    issues.sort_by(|a, b| {
        (&a.path_posix, &a.rule, &a.message).cmp(&(&b.path_posix, &b.rule, &b.message))
    });

    let mut rule_counts: BTreeMap<String, usize> = BTreeMap::new();
    for issue in &issues {
        *rule_counts.entry(issue.rule.clone()).or_insert(0) += 1;
    }

    let summary = PolicyLintSummary {
        violations: issues.len(),
        files_scanned: skill_files.len() + claude_command_files.len(),
        skill_files: skill_files.len(),
        claude_command_files: claude_command_files.len(),
        rules: rule_counts,
    };

    Ok(PolicyLintReport {
        root: root_str,
        root_posix: crate::paths::path_to_posix_string(root),
        issues,
        summary,
    })
}

fn normalize_git_remote_for_policy(url: &str) -> String {
    let mut u = url.trim().trim_end_matches(".git").to_string();
    if let Some(rest) = u.strip_prefix("git@") {
        u = rest.replace(':', "/");
    } else if let Some(rest) = u.strip_prefix("https://") {
        u = rest.to_string();
    } else if let Some(rest) = u.strip_prefix("http://") {
        u = rest.to_string();
    } else if let Some(rest) = u.strip_prefix("ssh://") {
        u = rest.to_string();
        if let Some((_, rest)) = u.split_once('@') {
            u = rest.to_string();
        }
        u = u.replace(':', "/");
    }
    u.trim_start_matches('/').to_lowercase()
}

fn remote_matches_allowlist(normalized_remote: &str, normalized_allow: &str) -> bool {
    if normalized_allow.is_empty() {
        return false;
    }
    if normalized_remote == normalized_allow {
        return true;
    }
    if !normalized_remote.starts_with(normalized_allow) {
        return false;
    }
    if normalized_allow.ends_with('/') {
        return true;
    }
    normalized_remote
        .as_bytes()
        .get(normalized_allow.len())
        .copied()
        == Some(b'/')
}

fn lint_org_config(
    root: &Path,
    out: &mut Vec<PolicyLintIssue>,
) -> Option<crate::policy_pack::OrgConfig> {
    let cfg_path = root.join(crate::policy_pack::ORG_CONFIG_FILE);
    let cfg = match crate::policy_pack::OrgConfig::load_optional(&cfg_path) {
        Ok(Some(cfg)) => cfg,
        Ok(None) => return None,
        Err(err) => {
            out.push(PolicyLintIssue {
                rule: "policy_config".to_string(),
                path: crate::policy_pack::ORG_CONFIG_FILE.to_string(),
                path_posix: crate::policy_pack::ORG_CONFIG_FILE.to_string(),
                message: "failed to parse policy config".to_string(),
                details: Some(serde_json::json!({ "error": err.to_string() })),
            });
            return None;
        }
    };

    if cfg.version != crate::policy_pack::ORG_CONFIG_VERSION {
        out.push(PolicyLintIssue {
            rule: "policy_config".to_string(),
            path: crate::policy_pack::ORG_CONFIG_FILE.to_string(),
            path_posix: crate::policy_pack::ORG_CONFIG_FILE.to_string(),
            message: format!("unsupported policy config version: {}", cfg.version),
            details: Some(serde_json::json!({
                "version": cfg.version,
                "supported": [crate::policy_pack::ORG_CONFIG_VERSION],
            })),
        });
        return None;
    }

    Some(cfg)
}

fn lint_policy_pack_lock(
    root: &Path,
    cfg: &crate::policy_pack::OrgConfig,
    out: &mut Vec<PolicyLintIssue>,
) {
    let Some(pack) = cfg.policy_pack.as_ref() else {
        return;
    };

    let source = match crate::source::parse_source_spec(pack.source.trim()) {
        Ok(s) => s,
        Err(err) => {
            out.push(PolicyLintIssue {
                rule: "policy_config".to_string(),
                path: crate::policy_pack::ORG_CONFIG_FILE.to_string(),
                path_posix: crate::policy_pack::ORG_CONFIG_FILE.to_string(),
                message: "invalid policy_pack.source".to_string(),
                details: Some(serde_json::json!({
                    "field": "policy_pack.source",
                    "value": pack.source,
                    "error": err.to_string(),
                })),
            });
            return;
        }
    };

    let lock_path = root.join(crate::policy_pack::ORG_LOCKFILE_FILE);
    if !lock_path.is_file() {
        out.push(PolicyLintIssue {
            rule: "policy_pack_lock".to_string(),
            path: crate::policy_pack::ORG_LOCKFILE_FILE.to_string(),
            path_posix: crate::policy_pack::ORG_LOCKFILE_FILE.to_string(),
            message: "missing policy lockfile (run `agentpack policy lock`)".to_string(),
            details: Some(serde_json::json!({
                "path": lock_path.to_string_lossy(),
            })),
        });
        return;
    }

    let lock = match crate::policy_pack::OrgLockfile::load(&lock_path) {
        Ok(lock) => lock,
        Err(err) => {
            out.push(PolicyLintIssue {
                rule: "policy_pack_lock".to_string(),
                path: crate::policy_pack::ORG_LOCKFILE_FILE.to_string(),
                path_posix: crate::policy_pack::ORG_LOCKFILE_FILE.to_string(),
                message: "invalid policy lockfile".to_string(),
                details: Some(serde_json::json!({
                    "path": lock_path.to_string_lossy(),
                    "error": err.to_string(),
                })),
            });
            return;
        }
    };

    if !policy_sources_match(&source, &lock.policy_pack.source) {
        out.push(PolicyLintIssue {
            rule: "policy_pack_lock".to_string(),
            path: crate::policy_pack::ORG_LOCKFILE_FILE.to_string(),
            path_posix: crate::policy_pack::ORG_LOCKFILE_FILE.to_string(),
            message:
                "policy lockfile does not match policy_pack config (run `agentpack policy lock`)"
                    .to_string(),
            details: Some(serde_json::json!({
                "config_source": source,
                "lock_source": lock.policy_pack.source,
            })),
        });
    }
}

fn lint_distribution_policy(
    root: &Path,
    cfg: &crate::policy_pack::OrgConfig,
    out: &mut Vec<PolicyLintIssue>,
) {
    let Some(policy) = cfg.distribution_policy.as_ref() else {
        return;
    };

    let manifest_path = root.join("agentpack.yaml");
    let manifest = match crate::config::Manifest::load(&manifest_path) {
        Ok(m) => m,
        Err(err) => {
            out.push(PolicyLintIssue {
                rule: "distribution_policy".to_string(),
                path: "agentpack.yaml".to_string(),
                path_posix: "agentpack.yaml".to_string(),
                message: "failed to load agentpack.yaml for distribution_policy".to_string(),
                details: Some(serde_json::json!({ "error": err.to_string() })),
            });
            return;
        }
    };

    let mut required_targets = Vec::new();
    for raw in &policy.required_targets {
        let value = raw.trim();
        if value.is_empty() {
            out.push(PolicyLintIssue {
                rule: "policy_config".to_string(),
                path: crate::policy_pack::ORG_CONFIG_FILE.to_string(),
                path_posix: crate::policy_pack::ORG_CONFIG_FILE.to_string(),
                message: "distribution_policy.required_targets contains an empty entry".to_string(),
                details: Some(
                    serde_json::json!({ "field": "distribution_policy.required_targets" }),
                ),
            });
            continue;
        }
        required_targets.push(value.to_string());
    }

    let mut required_modules = Vec::new();
    for raw in &policy.required_modules {
        let value = raw.trim();
        if value.is_empty() {
            out.push(PolicyLintIssue {
                rule: "policy_config".to_string(),
                path: crate::policy_pack::ORG_CONFIG_FILE.to_string(),
                path_posix: crate::policy_pack::ORG_CONFIG_FILE.to_string(),
                message: "distribution_policy.required_modules contains an empty entry".to_string(),
                details: Some(
                    serde_json::json!({ "field": "distribution_policy.required_modules" }),
                ),
            });
            continue;
        }
        required_modules.push(value.to_string());
    }

    if !required_targets.is_empty() {
        let mut missing: Vec<String> = required_targets
            .iter()
            .filter(|t| !manifest.targets.contains_key(t.as_str()))
            .cloned()
            .collect();
        missing.sort();
        missing.dedup();

        if !missing.is_empty() {
            out.push(PolicyLintIssue {
                rule: "distribution_required_targets".to_string(),
                path: "agentpack.yaml".to_string(),
                path_posix: "agentpack.yaml".to_string(),
                message: format!("missing required targets: {}", missing.join(", ")),
                details: Some(serde_json::json!({
                    "required": required_targets,
                    "missing": missing,
                })),
            });
        }
    }

    if !required_modules.is_empty() {
        let mut missing = Vec::new();
        let mut disabled = Vec::new();

        for id in &required_modules {
            match manifest.modules.iter().find(|m| m.id == *id) {
                None => missing.push(id.clone()),
                Some(m) if !m.enabled => disabled.push(id.clone()),
                Some(_) => {}
            }
        }

        missing.sort();
        missing.dedup();
        disabled.sort();
        disabled.dedup();

        if !missing.is_empty() || !disabled.is_empty() {
            out.push(PolicyLintIssue {
                rule: "distribution_required_modules".to_string(),
                path: "agentpack.yaml".to_string(),
                path_posix: "agentpack.yaml".to_string(),
                message:
                    "distribution policy requires enabled modules, but some are missing or disabled"
                        .to_string(),
                details: Some(serde_json::json!({
                    "required": required_modules,
                    "missing": missing,
                    "disabled": disabled,
                })),
            });
        }
    }
}

fn lint_supply_chain_policy(
    root: &Path,
    cfg: &crate::policy_pack::OrgConfig,
    out: &mut Vec<PolicyLintIssue>,
) {
    let Some(policy) = cfg.supply_chain_policy.as_ref() else {
        return;
    };

    let mut allowed_git_remotes = Vec::new();
    for raw in &policy.allowed_git_remotes {
        let value = raw.trim();
        if value.is_empty() {
            out.push(PolicyLintIssue {
                rule: "policy_config".to_string(),
                path: crate::policy_pack::ORG_CONFIG_FILE.to_string(),
                path_posix: crate::policy_pack::ORG_CONFIG_FILE.to_string(),
                message: "supply_chain_policy.allowed_git_remotes contains an empty entry"
                    .to_string(),
                details: Some(
                    serde_json::json!({ "field": "supply_chain_policy.allowed_git_remotes" }),
                ),
            });
            continue;
        }
        allowed_git_remotes.push(normalize_git_remote_for_policy(value));
    }

    allowed_git_remotes.sort();
    allowed_git_remotes.dedup();

    if allowed_git_remotes.is_empty() {
        return;
    }

    let manifest_path = root.join("agentpack.yaml");
    let manifest = match crate::config::Manifest::load(&manifest_path) {
        Ok(m) => m,
        Err(err) => {
            out.push(PolicyLintIssue {
                rule: "supply_chain_policy".to_string(),
                path: "agentpack.yaml".to_string(),
                path_posix: "agentpack.yaml".to_string(),
                message: "failed to load agentpack.yaml for supply_chain_policy".to_string(),
                details: Some(serde_json::json!({ "error": err.to_string() })),
            });
            return;
        }
    };

    for module in &manifest.modules {
        let Some(gs) = module.source.git.as_ref() else {
            continue;
        };

        let normalized_remote = normalize_git_remote_for_policy(gs.url.as_str());
        if allowed_git_remotes
            .iter()
            .any(|a| remote_matches_allowlist(normalized_remote.as_str(), a.as_str()))
        {
            continue;
        }

        out.push(PolicyLintIssue {
            rule: "supply_chain_allowed_git_remotes".to_string(),
            path: "agentpack.yaml".to_string(),
            path_posix: "agentpack.yaml".to_string(),
            message: format!("git remote is not allowlisted for module {}", module.id),
            details: Some(serde_json::json!({
                "module_id": module.id,
                "remote": gs.url,
                "remote_normalized": normalized_remote,
                "allowed_git_remotes": allowed_git_remotes,
            })),
        });
    }
}

fn policy_sources_match(a: &crate::config::Source, b: &crate::config::Source) -> bool {
    match (a.kind(), b.kind()) {
        (crate::config::SourceKind::LocalPath, crate::config::SourceKind::LocalPath) => {
            let a = a
                .local_path
                .as_ref()
                .map(|lp| lp.path.replace('\\', "/"))
                .unwrap_or_default();
            let b = b
                .local_path
                .as_ref()
                .map(|lp| lp.path.replace('\\', "/"))
                .unwrap_or_default();
            a == b
        }
        (crate::config::SourceKind::Git, crate::config::SourceKind::Git) => {
            let Some(a) = a.git.as_ref() else {
                return false;
            };
            let Some(b) = b.git.as_ref() else {
                return false;
            };
            a.url == b.url
                && a.ref_name == b.ref_name
                && a.subdir == b.subdir
                && a.shallow == b.shallow
        }
        _ => false,
    }
}

fn find_skill_files(root: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    for entry in WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| !should_ignore_entry(e))
    {
        let entry = entry?;
        if entry.file_type().is_file() && entry.file_name() == "SKILL.md" {
            out.push(entry.into_path());
        }
    }
    out.sort();
    Ok(out)
}

fn find_claude_command_files(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for rel in CLAUDE_COMMAND_DIRS {
        let dir = root.join(rel);
        if !dir.is_dir() {
            continue;
        }
        for entry in WalkDir::new(&dir)
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| !should_ignore_entry(e))
        {
            let Ok(entry) = entry else {
                continue;
            };
            if !entry.file_type().is_file() {
                continue;
            }
            if entry.path().extension().and_then(|s| s.to_str()) != Some("md") {
                continue;
            }
            out.push(entry.into_path());
        }
    }
    out.sort();
    out
}

fn should_ignore_entry(entry: &walkdir::DirEntry) -> bool {
    if !entry.file_type().is_dir() {
        return false;
    }
    IGNORED_DIR_NAMES
        .iter()
        .any(|name| entry.file_name() == *name)
}

fn lint_skill_file(root: &Path, path: &Path, out: &mut Vec<PolicyLintIssue>) {
    let rel = path.strip_prefix(root).unwrap_or(path);
    let rel_str = rel.to_string_lossy().to_string();
    let rel_posix = crate::paths::path_to_posix_string(rel);

    let text = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(err) => {
            out.push(PolicyLintIssue {
                rule: "io_read".to_string(),
                path: rel_str,
                path_posix: rel_posix,
                message: "failed to read file".to_string(),
                details: Some(serde_json::json!({ "error": err.to_string() })),
            });
            return;
        }
    };

    let frontmatter = match extract_yaml_frontmatter(&text) {
        Ok(Some(v)) => v,
        Ok(None) => {
            out.push(PolicyLintIssue {
                rule: "skill_frontmatter".to_string(),
                path: rel_str,
                path_posix: rel_posix,
                message: "missing YAML frontmatter (--- ... ---)".to_string(),
                details: Some(serde_json::json!({ "required_fields": ["name","description"] })),
            });
            return;
        }
        Err(err) => {
            out.push(PolicyLintIssue {
                rule: "skill_frontmatter".to_string(),
                path: rel_str,
                path_posix: rel_posix,
                message: "invalid YAML frontmatter".to_string(),
                details: Some(serde_json::json!({ "error": err.to_string() })),
            });
            return;
        }
    };

    let Some(map) = frontmatter.as_mapping() else {
        out.push(PolicyLintIssue {
            rule: "skill_frontmatter".to_string(),
            path: rel_str,
            path_posix: rel_posix,
            message: "frontmatter must be a YAML mapping".to_string(),
            details: Some(serde_json::json!({ "expected": "mapping" })),
        });
        return;
    };

    for key in ["name", "description"] {
        match yaml_get(map, key) {
            Some(serde_yaml::Value::String(s)) if !s.trim().is_empty() => {}
            Some(serde_yaml::Value::String(_)) => {
                out.push(PolicyLintIssue {
                    rule: "skill_frontmatter".to_string(),
                    path: rel_str.clone(),
                    path_posix: rel_posix.clone(),
                    message: format!("frontmatter {key} is empty"),
                    details: Some(serde_json::json!({ "field": key })),
                });
            }
            Some(_) => {
                out.push(PolicyLintIssue {
                    rule: "skill_frontmatter".to_string(),
                    path: rel_str.clone(),
                    path_posix: rel_posix.clone(),
                    message: format!("frontmatter {key} must be a string"),
                    details: Some(serde_json::json!({ "field": key, "expected": "string" })),
                });
            }
            None => {
                out.push(PolicyLintIssue {
                    rule: "skill_frontmatter".to_string(),
                    path: rel_str.clone(),
                    path_posix: rel_posix.clone(),
                    message: format!("frontmatter is missing {key}"),
                    details: Some(serde_json::json!({ "missing": [key] })),
                });
            }
        }
    }
}

fn lint_claude_command_file(root: &Path, path: &Path, out: &mut Vec<PolicyLintIssue>) {
    let rel = path.strip_prefix(root).unwrap_or(path);
    let rel_str = rel.to_string_lossy().to_string();
    let rel_posix = crate::paths::path_to_posix_string(rel);

    let text = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(err) => {
            out.push(PolicyLintIssue {
                rule: "io_read".to_string(),
                path: rel_str,
                path_posix: rel_posix,
                message: "failed to read file".to_string(),
                details: Some(serde_json::json!({ "error": err.to_string() })),
            });
            return;
        }
    };

    let uses_bash = uses_bash_tool(&text);
    if uses_bash {
        lint_claude_command_allowed_tools(&text, &rel_str, &rel_posix, out);
        lint_claude_command_dangerous_defaults(&text, &rel_str, &rel_posix, out);
    }
}

fn lint_claude_command_allowed_tools(
    markdown: &str,
    rel_str: &str,
    rel_posix: &str,
    out: &mut Vec<PolicyLintIssue>,
) {
    let frontmatter = match extract_yaml_frontmatter(markdown) {
        Ok(Some(v)) => v,
        Ok(None) => {
            out.push(PolicyLintIssue {
                rule: "claude_command_allowed_tools".to_string(),
                path: rel_str.to_string(),
                path_posix: rel_posix.to_string(),
                message: "uses bash tool but is missing YAML frontmatter (--- ... ---)".to_string(),
                details: None,
            });
            return;
        }
        Err(err) => {
            out.push(PolicyLintIssue {
                rule: "claude_command_allowed_tools".to_string(),
                path: rel_str.to_string(),
                path_posix: rel_posix.to_string(),
                message: "invalid YAML frontmatter".to_string(),
                details: Some(serde_json::json!({ "error": err.to_string() })),
            });
            return;
        }
    };

    let Some(map) = frontmatter.as_mapping() else {
        out.push(PolicyLintIssue {
            rule: "claude_command_allowed_tools".to_string(),
            path: rel_str.to_string(),
            path_posix: rel_posix.to_string(),
            message: "frontmatter must be a YAML mapping".to_string(),
            details: Some(serde_json::json!({ "expected": "mapping" })),
        });
        return;
    };

    let Some(allowed) = yaml_get(map, "allowed-tools") else {
        out.push(PolicyLintIssue {
            rule: "claude_command_allowed_tools".to_string(),
            path: rel_str.to_string(),
            path_posix: rel_posix.to_string(),
            message: "uses bash tool but frontmatter is missing allowed-tools".to_string(),
            details: Some(serde_json::json!({ "missing": ["allowed-tools"] })),
        });
        return;
    };

    if !allowed_tools_allows_bash(allowed) {
        out.push(PolicyLintIssue {
            rule: "claude_command_allowed_tools".to_string(),
            path: rel_str.to_string(),
            path_posix: rel_posix.to_string(),
            message: "uses bash tool but allowed-tools does not include Bash(...)".to_string(),
            details: None,
        });
    }
}

fn lint_claude_command_dangerous_defaults(
    markdown: &str,
    rel_str: &str,
    rel_posix: &str,
    out: &mut Vec<PolicyLintIssue>,
) {
    for (line, cmdline) in extract_bash_commands(markdown) {
        let invocations = extract_agentpack_invocations(&cmdline);
        for invocation in invocations {
            let Some(command_id) = agentpack_command_id(&invocation) else {
                continue;
            };

            let is_mutating = crate::cli::util::MUTATING_COMMAND_IDS
                .iter()
                .any(|id| id == &command_id.as_str());
            if !is_mutating {
                continue;
            }

            let has_json = invocation.iter().any(|t| *t == "--json");
            let has_yes = invocation.iter().any(|t| *t == "--yes");
            if has_json && has_yes {
                continue;
            }

            let mut missing = Vec::new();
            if !has_json {
                missing.push("--json");
            }
            if !has_yes {
                missing.push("--yes");
            }

            out.push(PolicyLintIssue {
                rule: "dangerous_defaults".to_string(),
                path: rel_str.to_string(),
                path_posix: rel_posix.to_string(),
                message: format!(
                    "mutating agentpack command '{command_id}' must include {}",
                    missing.join(" and ")
                ),
                details: Some(serde_json::json!({
                    "line": line,
                    "command_id": command_id,
                    "invocation": invocation.join(" "),
                    "missing": missing,
                })),
            });
        }
    }
}

fn uses_bash_tool(markdown: &str) -> bool {
    markdown.contains("!bash") || markdown.contains("!`bash`")
}

fn extract_yaml_frontmatter(markdown: &str) -> anyhow::Result<Option<serde_yaml::Value>> {
    let mut lines = markdown.lines();
    let first = lines.next().unwrap_or("").trim_end_matches('\r');
    if first != "---" {
        return Ok(None);
    }

    let mut fm = Vec::new();
    let mut found_end = false;
    for line in lines {
        let line = line.trim_end_matches('\r');
        if line == "---" {
            found_end = true;
            break;
        }
        fm.push(line);
    }

    if !found_end {
        anyhow::bail!("unterminated YAML frontmatter (missing closing ---)");
    }

    let value: serde_yaml::Value =
        serde_yaml::from_str(&fm.join("\n")).context("parse YAML frontmatter")?;
    Ok(Some(value))
}

fn yaml_get<'a>(map: &'a serde_yaml::Mapping, key: &str) -> Option<&'a serde_yaml::Value> {
    map.iter().find_map(|(k, v)| match k {
        serde_yaml::Value::String(s) if s == key => Some(v),
        _ => None,
    })
}

fn allowed_tools_allows_bash(allowed: &serde_yaml::Value) -> bool {
    match allowed {
        serde_yaml::Value::String(s) => s.contains("Bash("),
        serde_yaml::Value::Sequence(items) => items.iter().any(|v| match v {
            serde_yaml::Value::String(s) => s.contains("Bash("),
            _ => false,
        }),
        _ => false,
    }
}

fn extract_bash_commands(markdown: &str) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    let mut in_block = false;
    for (idx, raw) in markdown.lines().enumerate() {
        let line = raw.trim_end_matches('\r');
        let trimmed = line.trim();

        if in_block {
            if trimmed.is_empty() {
                in_block = false;
                continue;
            }
            out.push((idx + 1, trimmed.to_string()));
            continue;
        }

        if trimmed == "!bash" || trimmed == "!`bash`" {
            in_block = true;
        }
    }
    out
}

fn extract_agentpack_invocations(line: &str) -> Vec<Vec<String>> {
    let tokens: Vec<&str> = line.split_whitespace().collect();
    if tokens.is_empty() {
        return Vec::new();
    }

    let mut out = Vec::new();
    let mut i = 0;
    while i < tokens.len() {
        if is_agentpack_token(tokens[i]) {
            let start = i + 1;
            let mut end = start;
            while end < tokens.len() && !is_shell_separator(tokens[end]) {
                end += 1;
            }
            let argv: Vec<String> = tokens[start..end].iter().map(|s| s.to_string()).collect();
            if !argv.is_empty() {
                out.push(argv);
            }
            i = end;
        } else {
            i += 1;
        }
    }

    out
}

fn is_agentpack_token(token: &str) -> bool {
    if token == "agentpack" {
        return true;
    }
    let token = token.trim_matches(|c| c == '"' || c == '\'');
    token.ends_with("/agentpack") || token.ends_with("\\agentpack.exe")
}

fn is_shell_separator(token: &str) -> bool {
    matches!(token, "|" | "||" | "&&" | ";" | "&")
}

fn agentpack_command_id(argv: &[String]) -> Option<String> {
    let mut idx = 0;
    while idx < argv.len() {
        let t = argv[idx].as_str();
        if t == "--" {
            idx += 1;
            break;
        }
        if !t.starts_with('-') {
            break;
        }
        if matches!(t, "--repo" | "--profile" | "--target" | "--machine") {
            idx += 2;
            continue;
        }
        if matches!(t, "--json" | "--yes" | "--dry-run") {
            idx += 1;
            continue;
        }
        idx += 1;
    }

    let cmd = argv.get(idx)?.as_str();
    let rest: Vec<&str> = argv.iter().skip(idx + 1).map(|s| s.as_str()).collect();

    let command_id = match cmd {
        "deploy" => {
            if rest.contains(&"--apply") {
                "deploy --apply".to_string()
            } else {
                "deploy".to_string()
            }
        }
        "doctor" => {
            if rest.contains(&"--fix") {
                "doctor --fix".to_string()
            } else {
                "doctor".to_string()
            }
        }
        "overlay" => match rest.first().copied() {
            Some("edit") => "overlay edit".to_string(),
            Some("rebase") => "overlay rebase".to_string(),
            Some(other) => format!("overlay {other}"),
            None => "overlay".to_string(),
        },
        "remote" => match rest.first().copied() {
            Some("set") => "remote set".to_string(),
            Some(other) => format!("remote {other}"),
            None => "remote".to_string(),
        },
        "evolve" => match rest.first().copied() {
            Some("propose") => "evolve propose".to_string(),
            Some("restore") => "evolve restore".to_string(),
            Some(other) => format!("evolve {other}"),
            None => "evolve".to_string(),
        },
        other => other.to_string(),
    };

    Some(command_id)
}
