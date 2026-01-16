use std::path::Path;

use anyhow::Context as _;
use serde::{Deserialize, Serialize};

use crate::config::{GitSource, Source, SourceKind};
use crate::fs::write_atomic;
use crate::lockfile::{FileEntry, ResolvedGitSource, ResolvedLocalPathSource, ResolvedSource};
use crate::paths::AgentpackHome;
use crate::paths::path_to_posix_string;
use crate::store::Store;
use crate::user_error::UserError;

pub(crate) const ORG_CONFIG_FILE: &str = "agentpack.org.yaml";
pub(crate) const ORG_LOCKFILE_FILE: &str = "agentpack.org.lock.json";

pub(crate) const ORG_CONFIG_VERSION: u32 = 1;
pub(crate) const ORG_LOCKFILE_VERSION: u32 = 1;

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct OrgConfig {
    pub version: u32,
    #[serde(default)]
    pub policy_pack: Option<PolicyPackConfig>,
    #[serde(default)]
    pub distribution_policy: Option<DistributionPolicyConfig>,
    #[serde(default)]
    pub supply_chain_policy: Option<SupplyChainPolicyConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct PolicyPackConfig {
    pub source: String,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct DistributionPolicyConfig {
    #[serde(default)]
    pub required_targets: Vec<String>,
    #[serde(default)]
    pub required_modules: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct SupplyChainPolicyConfig {
    #[serde(default)]
    pub allowed_git_remotes: Vec<String>,
}

impl OrgConfig {
    pub(crate) fn load_required(path: &Path) -> anyhow::Result<Self> {
        let raw = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                return Err(anyhow::Error::new(
                    UserError::new(
                        "E_POLICY_CONFIG_MISSING",
                        format!("missing policy config: {}", path.display()),
                    )
                    .with_details(serde_json::json!({
                        "path": path.to_string_lossy(),
                        "hint": "create repo/agentpack.org.yaml (governance is opt-in)",
                    })),
                ));
            }
            Err(err) => return Err(err).with_context(|| format!("read {}", path.display())),
        };

        let cfg: OrgConfig = serde_yaml::from_str(&raw).map_err(|err| {
            anyhow::Error::new(
                UserError::new(
                    "E_POLICY_CONFIG_INVALID",
                    format!("invalid policy config yaml: {}", path.display()),
                )
                .with_details(serde_json::json!({
                    "path": path.to_string_lossy(),
                    "error": err.to_string(),
                })),
            )
        })?;

        if cfg.version != ORG_CONFIG_VERSION {
            return Err(anyhow::Error::new(
                UserError::new(
                    "E_POLICY_CONFIG_UNSUPPORTED_VERSION",
                    format!("unsupported policy config version: {}", cfg.version),
                )
                .with_details(serde_json::json!({
                    "path": path.to_string_lossy(),
                    "version": cfg.version,
                    "supported": [ORG_CONFIG_VERSION],
                })),
            ));
        }

        Ok(cfg)
    }

    pub(crate) fn load_optional(path: &Path) -> anyhow::Result<Option<Self>> {
        let raw = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(err) => return Err(err).with_context(|| format!("read {}", path.display())),
        };

        let cfg: OrgConfig = serde_yaml::from_str(&raw)
            .with_context(|| format!("parse policy config yaml: {}", path.display()))?;
        Ok(Some(cfg))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct OrgLockfile {
    pub version: u32,
    pub policy_pack: LockedPolicyPack,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct LockedPolicyPack {
    pub source: Source,
    pub resolved_source: ResolvedSource,
    pub resolved_version: String,
    pub sha256: String,
    pub file_manifest: Vec<FileEntry>,
}

impl OrgLockfile {
    pub(crate) fn load(path: &Path) -> anyhow::Result<Self> {
        let raw =
            std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
        let lock: OrgLockfile =
            serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
        if lock.version != ORG_LOCKFILE_VERSION {
            anyhow::bail!("unsupported policy lockfile version: {}", lock.version);
        }
        Ok(lock)
    }

    pub(crate) fn save(&self, path: &Path) -> anyhow::Result<()> {
        let mut out = serde_json::to_string_pretty(self).context("serialize policy lockfile")?;
        if !out.ends_with('\n') {
            out.push('\n');
        }
        write_atomic(path, out.as_bytes()).with_context(|| format!("write {}", path.display()))?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub(crate) struct PolicyLockReport {
    pub lockfile_path: String,
    pub lockfile_path_posix: String,
    pub resolved_version: String,
    pub sha256: String,
    pub files: usize,
}

pub(crate) fn lock_policy_pack(
    home: &AgentpackHome,
    repo_dir: &Path,
) -> anyhow::Result<PolicyLockReport> {
    let cfg_path = repo_dir.join(ORG_CONFIG_FILE);
    let cfg = OrgConfig::load_required(&cfg_path)?;
    let Some(pack) = cfg.policy_pack.as_ref() else {
        return Err(anyhow::Error::new(
            UserError::new(
                "E_POLICY_CONFIG_INVALID",
                "policy_pack is missing in agentpack.org.yaml".to_string(),
            )
            .with_details(serde_json::json!({
                "path": cfg_path.to_string_lossy(),
                "missing": ["policy_pack"],
                "hint": "add policy_pack.source (local:... or git:...)",
            })),
        ));
    };
    if pack.source.trim().is_empty() {
        return Err(anyhow::Error::new(
            UserError::new(
                "E_POLICY_CONFIG_INVALID",
                "policy_pack.source is empty in agentpack.org.yaml".to_string(),
            )
            .with_details(serde_json::json!({
                "path": cfg_path.to_string_lossy(),
                "field": "policy_pack.source",
            })),
        ));
    }

    let source: Source = crate::source::parse_source_spec(pack.source.trim()).map_err(|err| {
        anyhow::Error::new(
            UserError::new(
                "E_POLICY_CONFIG_INVALID",
                "unsupported policy_pack.source".to_string(),
            )
            .with_details(serde_json::json!({
                "path": cfg_path.to_string_lossy(),
                "field": "policy_pack.source",
                "value": pack.source,
                "error": err.to_string(),
                "hint": "expected local:... or git:...#ref=...&subdir=...",
            })),
        )
    })?;

    let (resolved_source, resolved_version, root) = match source.kind() {
        SourceKind::LocalPath => {
            let lp = source.local_path.as_ref().context("missing local_path")?;
            let abs = repo_dir.join(&lp.path);
            let rel = lp.path.replace('\\', "/");
            (
                ResolvedSource {
                    local_path: Some(ResolvedLocalPathSource { path: rel.clone() }),
                    git: None,
                },
                "local".to_string(),
                abs,
            )
        }
        SourceKind::Git => {
            let gs: &GitSource = source.git.as_ref().context("missing git source")?;
            let store = Store::new(home);
            let commit = store.resolve_git_commit(gs)?;
            let checkout = store.ensure_git_checkout("policy_pack", gs, &commit)?;
            let root = Store::module_root_in_checkout(&checkout, &gs.subdir);
            (
                ResolvedSource {
                    local_path: None,
                    git: Some(ResolvedGitSource {
                        url: gs.url.clone(),
                        commit: commit.clone(),
                        subdir: gs.subdir.clone(),
                    }),
                },
                commit,
                root,
            )
        }
        SourceKind::Invalid => anyhow::bail!("invalid policy pack source"),
    };

    let (file_manifest, module_hash) =
        crate::lockfile::hash_tree(&root).with_context(|| format!("hash {}", root.display()))?;

    let lockfile = OrgLockfile {
        version: ORG_LOCKFILE_VERSION,
        policy_pack: LockedPolicyPack {
            source,
            resolved_source,
            resolved_version: resolved_version.clone(),
            sha256: module_hash.clone(),
            file_manifest,
        },
    };

    let lock_path = repo_dir.join(ORG_LOCKFILE_FILE);
    lockfile.save(&lock_path)?;

    Ok(PolicyLockReport {
        lockfile_path: lock_path.to_string_lossy().to_string(),
        lockfile_path_posix: path_to_posix_string(&lock_path),
        resolved_version,
        sha256: module_hash,
        files: lockfile.policy_pack.file_manifest.len(),
    })
}
