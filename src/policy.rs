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

#[derive(Debug, Clone)]
pub(crate) struct PolicyAuditOutcome {
    pub report: PolicyAuditReport,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct PolicyAuditReport {
    pub root: String,
    pub root_posix: String,
    pub lockfile: PolicyAuditLockfileInfo,
    pub modules: Vec<PolicyAuditModule>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub org_policy_pack: Option<PolicyAuditOrgPolicyPack>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub change_summary: Option<PolicyAuditChangeSummary>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct PolicyAuditLockfileInfo {
    pub lockfile_path: String,
    pub lockfile_path_posix: String,
    pub version: u32,
    pub generated_at: String,
    pub modules: usize,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct PolicyAuditModule {
    pub id: String,
    #[serde(rename = "type")]
    pub module_type: crate::config::ModuleType,
    pub resolved_source: crate::lockfile::ResolvedSource,
    pub resolved_version: String,
    pub sha256: String,
    pub files: usize,
    pub bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct PolicyAuditOrgPolicyPack {
    pub lockfile_path: String,
    pub lockfile_path_posix: String,
    pub source: crate::config::Source,
    pub resolved_source: crate::lockfile::ResolvedSource,
    pub resolved_version: String,
    pub sha256: String,
    pub files: usize,
    pub bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct PolicyAuditChangeSummary {
    pub base_ref: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_commit: Option<String>,
    pub modules_added: Vec<String>,
    pub modules_removed: Vec<String>,
    pub modules_changed: Vec<PolicyAuditModuleChange>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct PolicyAuditModuleChange {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module_type: Option<PolicyAuditFieldChange<crate::config::ModuleType>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved_source: Option<PolicyAuditFieldChange<crate::lockfile::ResolvedSource>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved_version: Option<PolicyAuditFieldChange<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha256: Option<PolicyAuditFieldChange<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files: Option<PolicyAuditFieldChange<usize>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bytes: Option<PolicyAuditFieldChange<u64>>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct PolicyAuditFieldChange<T> {
    pub before: T,
    pub after: T,
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

pub(crate) fn audit(root: &Path) -> anyhow::Result<PolicyAuditOutcome> {
    let root_str = root.to_string_lossy().to_string();
    if !root.exists() {
        anyhow::bail!("policy audit root does not exist: {root_str}");
    }

    let mut warnings = Vec::new();

    let lockfile_path = root.join("agentpack.lock.json");
    let lock =
        crate::lockfile::Lockfile::load(&lockfile_path).context("load agentpack.lock.json")?;

    let mut modules: Vec<PolicyAuditModule> = lock
        .modules
        .iter()
        .map(|m| {
            let bytes = m.file_manifest.iter().map(|f| f.bytes).sum::<u64>();
            PolicyAuditModule {
                id: m.id.clone(),
                module_type: m.module_type.clone(),
                resolved_source: m.resolved_source.clone(),
                resolved_version: m.resolved_version.clone(),
                sha256: m.sha256.clone(),
                files: m.file_manifest.len(),
                bytes,
            }
        })
        .collect();
    modules.sort_by(|a, b| a.id.cmp(&b.id));

    let lockfile = PolicyAuditLockfileInfo {
        lockfile_path: lockfile_path.to_string_lossy().to_string(),
        lockfile_path_posix: crate::paths::path_to_posix_string(&lockfile_path),
        version: lock.version,
        generated_at: lock.generated_at,
        modules: modules.len(),
    };

    let org_policy_pack = load_org_policy_pack_lock(root, &mut warnings);
    let change_summary =
        compute_lockfile_change_summary(root, lockfile.version, &modules, &mut warnings);

    Ok(PolicyAuditOutcome {
        report: PolicyAuditReport {
            root: root_str,
            root_posix: crate::paths::path_to_posix_string(root),
            lockfile,
            modules,
            org_policy_pack,
            change_summary,
        },
        warnings,
    })
}

fn load_org_policy_pack_lock(
    root: &Path,
    warnings: &mut Vec<String>,
) -> Option<PolicyAuditOrgPolicyPack> {
    let lock_path = root.join(crate::policy_pack::ORG_LOCKFILE_FILE);
    if !lock_path.is_file() {
        return None;
    }

    let lock = match crate::policy_pack::OrgLockfile::load(&lock_path) {
        Ok(lock) => lock,
        Err(err) => {
            warnings.push(format!(
                "policy audit: failed to load {}: {err}",
                crate::policy_pack::ORG_LOCKFILE_FILE
            ));
            return None;
        }
    };

    let files = lock.policy_pack.file_manifest.len();
    let bytes = lock
        .policy_pack
        .file_manifest
        .iter()
        .map(|f| f.bytes)
        .sum::<u64>();

    Some(PolicyAuditOrgPolicyPack {
        lockfile_path: lock_path.to_string_lossy().to_string(),
        lockfile_path_posix: crate::paths::path_to_posix_string(&lock_path),
        source: lock.policy_pack.source,
        resolved_source: lock.policy_pack.resolved_source,
        resolved_version: lock.policy_pack.resolved_version,
        sha256: lock.policy_pack.sha256,
        files,
        bytes,
    })
}

fn compute_lockfile_change_summary(
    root: &Path,
    lockfile_version: u32,
    current_modules: &[PolicyAuditModule],
    warnings: &mut Vec<String>,
) -> Option<PolicyAuditChangeSummary> {
    let Ok(inside) = crate::git::git_in(root, &["rev-parse", "--is-inside-work-tree"]) else {
        return None;
    };
    if inside.trim() != "true" {
        return None;
    }

    let base_ref = "HEAD^".to_string();
    let base_commit = match crate::git::git_in(root, &["rev-parse", "HEAD^"]) {
        Ok(s) => Some(s.trim().to_string()),
        Err(err) => {
            warnings.push(format!(
                "policy audit: change summary unavailable (no parent commit for {base_ref}): {err}"
            ));
            return None;
        }
    };

    let prev_raw = match crate::git::git_in(root, &["show", "HEAD^:agentpack.lock.json"]) {
        Ok(s) => s,
        Err(err) => {
            warnings.push(format!(
                "policy audit: change summary unavailable (git show {base_ref}:agentpack.lock.json failed): {err}"
            ));
            return None;
        }
    };

    let prev_lock: crate::lockfile::Lockfile = match serde_json::from_str(&prev_raw) {
        Ok(lock) => lock,
        Err(err) => {
            warnings.push(format!(
                "policy audit: change summary unavailable (failed to parse {base_ref}:agentpack.lock.json): {err}"
            ));
            return None;
        }
    };
    if prev_lock.version != lockfile_version {
        warnings.push(format!(
            "policy audit: change summary unavailable (lockfile version changed: {} -> {})",
            prev_lock.version, lockfile_version
        ));
        return None;
    }

    let mut prev_modules: Vec<PolicyAuditModule> = prev_lock
        .modules
        .into_iter()
        .map(|m| {
            let bytes = m.file_manifest.iter().map(|f| f.bytes).sum::<u64>();
            PolicyAuditModule {
                id: m.id,
                module_type: m.module_type,
                resolved_source: m.resolved_source,
                resolved_version: m.resolved_version,
                sha256: m.sha256,
                files: m.file_manifest.len(),
                bytes,
            }
        })
        .collect();
    prev_modules.sort_by(|a, b| a.id.cmp(&b.id));

    let prev_by_id: BTreeMap<String, PolicyAuditModule> = prev_modules
        .into_iter()
        .map(|m| (m.id.clone(), m))
        .collect();
    let current_by_id: BTreeMap<String, PolicyAuditModule> = current_modules
        .iter()
        .cloned()
        .map(|m| (m.id.clone(), m))
        .collect();

    let mut modules_added = Vec::new();
    for id in current_by_id.keys() {
        if !prev_by_id.contains_key(id) {
            modules_added.push(id.clone());
        }
    }

    let mut modules_removed = Vec::new();
    for id in prev_by_id.keys() {
        if !current_by_id.contains_key(id) {
            modules_removed.push(id.clone());
        }
    }

    let mut modules_changed = Vec::new();
    for (id, current) in &current_by_id {
        let Some(prev) = prev_by_id.get(id) else {
            continue;
        };

        let mut change = PolicyAuditModuleChange {
            id: id.clone(),
            module_type: None,
            resolved_source: None,
            resolved_version: None,
            sha256: None,
            files: None,
            bytes: None,
        };

        if prev.module_type != current.module_type {
            change.module_type = Some(PolicyAuditFieldChange {
                before: prev.module_type.clone(),
                after: current.module_type.clone(),
            });
        }
        if prev.resolved_source != current.resolved_source {
            change.resolved_source = Some(PolicyAuditFieldChange {
                before: prev.resolved_source.clone(),
                after: current.resolved_source.clone(),
            });
        }
        if prev.resolved_version != current.resolved_version {
            change.resolved_version = Some(PolicyAuditFieldChange {
                before: prev.resolved_version.clone(),
                after: current.resolved_version.clone(),
            });
        }
        if prev.sha256 != current.sha256 {
            change.sha256 = Some(PolicyAuditFieldChange {
                before: prev.sha256.clone(),
                after: current.sha256.clone(),
            });
        }
        if prev.files != current.files {
            change.files = Some(PolicyAuditFieldChange {
                before: prev.files,
                after: current.files,
            });
        }
        if prev.bytes != current.bytes {
            change.bytes = Some(PolicyAuditFieldChange {
                before: prev.bytes,
                after: current.bytes,
            });
        }

        if change.module_type.is_some()
            || change.resolved_source.is_some()
            || change.resolved_version.is_some()
            || change.sha256.is_some()
            || change.files.is_some()
            || change.bytes.is_some()
        {
            modules_changed.push(change);
        }
    }

    Some(PolicyAuditChangeSummary {
        base_ref,
        base_commit,
        modules_added,
        modules_removed,
        modules_changed,
    })
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
        allowed_git_remotes.push(crate::policy_allowlist::normalize_git_remote_for_policy(
            value,
        ));
    }

    allowed_git_remotes.sort();
    allowed_git_remotes.dedup();

    if allowed_git_remotes.is_empty() && !policy.require_lockfile {
        return;
    }

    if !allowed_git_remotes.is_empty() {
        if let Some(pack) = cfg.policy_pack.as_ref() {
            if let Ok(source) = crate::source::parse_source_spec(pack.source.trim()) {
                if source.kind() == crate::config::SourceKind::Git {
                    if let Some(gs) = source.git.as_ref() {
                        let normalized_remote =
                            crate::policy_allowlist::normalize_git_remote_for_policy(
                                gs.url.as_str(),
                            );
                        if !allowed_git_remotes.iter().any(|a| {
                            crate::policy_allowlist::remote_matches_allowlist(
                                normalized_remote.as_str(),
                                a.as_str(),
                            )
                        }) {
                            out.push(PolicyLintIssue {
                                rule: "policy_pack_allowed_git_remotes".to_string(),
                                path: crate::policy_pack::ORG_CONFIG_FILE.to_string(),
                                path_posix: crate::policy_pack::ORG_CONFIG_FILE.to_string(),
                                message: "git remote is not allowlisted for policy pack"
                                    .to_string(),
                                details: Some(serde_json::json!({
                                    "remote": gs.url,
                                    "remote_normalized": normalized_remote,
                                    "allowed_git_remotes": allowed_git_remotes,
                                })),
                            });
                        }
                    }
                }
            }
        }
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

    if policy.require_lockfile {
        let enabled_git_modules: Vec<_> = manifest
            .modules
            .iter()
            .filter(|m| m.enabled && m.source.git.is_some())
            .collect();
        if !enabled_git_modules.is_empty() {
            lint_supply_chain_lockfile(root, &enabled_git_modules, out);
        }
    }

    if !allowed_git_remotes.is_empty() {
        for module in &manifest.modules {
            let Some(gs) = module.source.git.as_ref() else {
                continue;
            };

            let normalized_remote =
                crate::policy_allowlist::normalize_git_remote_for_policy(gs.url.as_str());
            if allowed_git_remotes.iter().any(|a| {
                crate::policy_allowlist::remote_matches_allowlist(
                    normalized_remote.as_str(),
                    a.as_str(),
                )
            }) {
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
}

fn is_hex_sha(s: &str) -> bool {
    if s.len() != 40 {
        return false;
    }
    s.chars().all(|c| c.is_ascii_hexdigit())
}

fn lint_supply_chain_lockfile(
    root: &Path,
    enabled_git_modules: &[&crate::config::Module],
    out: &mut Vec<PolicyLintIssue>,
) {
    let lockfile_path = root.join("agentpack.lock.json");
    if !lockfile_path.is_file() {
        out.push(PolicyLintIssue {
            rule: "supply_chain_lockfile_missing".to_string(),
            path: "agentpack.lock.json".to_string(),
            path_posix: "agentpack.lock.json".to_string(),
            message: "missing lockfile required by supply_chain_policy (run `agentpack lock`)"
                .to_string(),
            details: Some(serde_json::json!({
                "path": lockfile_path.to_string_lossy(),
                "enabled_git_modules": enabled_git_modules.iter().map(|m| m.id.as_str()).collect::<Vec<_>>(),
            })),
        });
        return;
    }

    let lock = match crate::lockfile::Lockfile::load(&lockfile_path) {
        Ok(lock) => lock,
        Err(err) => {
            out.push(PolicyLintIssue {
                rule: "supply_chain_lockfile_invalid".to_string(),
                path: "agentpack.lock.json".to_string(),
                path_posix: "agentpack.lock.json".to_string(),
                message: "invalid lockfile required by supply_chain_policy".to_string(),
                details: Some(serde_json::json!({
                    "path": lockfile_path.to_string_lossy(),
                    "error": err.to_string(),
                })),
            });
            return;
        }
    };

    let mut by_id: BTreeMap<&str, &crate::lockfile::LockedModule> = BTreeMap::new();
    for m in &lock.modules {
        by_id.insert(m.id.as_str(), m);
    }

    for module in enabled_git_modules {
        let Some(cfg_git) = module.source.git.as_ref() else {
            continue;
        };

        let Some(locked) = by_id.get(module.id.as_str()) else {
            out.push(PolicyLintIssue {
                rule: "supply_chain_lockfile_missing_module".to_string(),
                path: "agentpack.lock.json".to_string(),
                path_posix: "agentpack.lock.json".to_string(),
                message: format!("lockfile is missing required module entry: {}", module.id),
                details: Some(serde_json::json!({
                    "module_id": module.id,
                })),
            });
            continue;
        };

        let Some(locked_git) = locked.resolved_source.git.as_ref() else {
            out.push(PolicyLintIssue {
                rule: "supply_chain_lockfile_source_mismatch".to_string(),
                path: "agentpack.lock.json".to_string(),
                path_posix: "agentpack.lock.json".to_string(),
                message: format!("lockfile entry is not git-sourced for module {}", module.id),
                details: Some(serde_json::json!({
                    "module_id": module.id,
                    "config": { "git_url": cfg_git.url },
                    "lock": { "resolved_source": locked.resolved_source },
                })),
            });
            continue;
        };

        let cfg_norm =
            crate::policy_allowlist::normalize_git_remote_for_policy(cfg_git.url.as_str());
        let lock_norm =
            crate::policy_allowlist::normalize_git_remote_for_policy(locked_git.url.as_str());
        if cfg_norm != lock_norm {
            out.push(PolicyLintIssue {
                rule: "supply_chain_lockfile_url_mismatch".to_string(),
                path: "agentpack.lock.json".to_string(),
                path_posix: "agentpack.lock.json".to_string(),
                message: format!("lockfile remote URL mismatch for module {}", module.id),
                details: Some(serde_json::json!({
                    "module_id": module.id,
                    "config_url": cfg_git.url,
                    "config_url_normalized": cfg_norm,
                    "lock_url": locked_git.url,
                    "lock_url_normalized": lock_norm,
                    "hint": "run `agentpack lock` (or `agentpack update`) to regenerate agentpack.lock.json",
                })),
            });
        }

        if !is_hex_sha(&locked_git.commit) {
            out.push(PolicyLintIssue {
                rule: "supply_chain_lockfile_unpinned_commit".to_string(),
                path: "agentpack.lock.json".to_string(),
                path_posix: "agentpack.lock.json".to_string(),
                message: format!(
                    "lockfile commit is not a 40-hex SHA for module {}",
                    module.id
                ),
                details: Some(serde_json::json!({
                    "module_id": module.id,
                    "commit": locked_git.commit,
                })),
            });
        }
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
