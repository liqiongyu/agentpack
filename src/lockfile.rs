use std::path::Path;

use anyhow::Context as _;
use serde::{Deserialize, Serialize};
use sha2::Digest as _;
use walkdir::WalkDir;

use crate::config::{GitSource, LocalPathSource, Manifest, ModuleType, SourceKind};
use crate::fs::write_atomic;
use crate::paths::RepoPaths;
use crate::store::Store;
use crate::user_error::UserError;

const LOCKFILE_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Lockfile {
    pub version: u32,
    pub generated_at: String,
    pub modules: Vec<LockedModule>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LockedModule {
    pub id: String,
    #[serde(rename = "type")]
    pub module_type: ModuleType,
    pub resolved_source: ResolvedSource,
    pub resolved_version: String,
    pub sha256: String,
    pub file_manifest: Vec<FileEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResolvedSource {
    #[serde(default)]
    pub local_path: Option<ResolvedLocalPathSource>,
    #[serde(default)]
    pub git: Option<ResolvedGitSource>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResolvedLocalPathSource {
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResolvedGitSource {
    pub url: String,
    pub commit: String,
    #[serde(default)]
    pub subdir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileEntry {
    pub path: String,
    pub sha256: String,
    pub bytes: u64,
}

impl Lockfile {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let raw = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                return Err(anyhow::Error::new(
                    UserError::new("E_LOCKFILE_MISSING", format!("missing lockfile: {}", path.display()))
                        .with_details(serde_json::json!({
                            "path": path.to_string_lossy(),
                            "reason_code": "lockfile_missing",
                            "next_actions": ["run_lock", "run_update", "retry_command"],
                            "hint": "run `agentpack update` (or `agentpack lock`) to generate agentpack.lock.json",
                        })),
                ));
            }
            Err(err) => {
                return Err(err).with_context(|| format!("read {}", path.display()));
            }
        };

        let lock: Lockfile = serde_json::from_str(&raw).map_err(|err| {
            anyhow::Error::new(
                UserError::new(
                    "E_LOCKFILE_INVALID",
                    format!("invalid lockfile json: {}", path.display()),
                )
                .with_details(serde_json::json!({
                    "path": path.to_string_lossy(),
                    "error": err.to_string(),
                    "reason_code": "lockfile_invalid_json",
                    "next_actions": ["regenerate_lockfile", "retry_command"],
                })),
            )
        })?;

        if lock.version != LOCKFILE_VERSION {
            return Err(anyhow::Error::new(
                UserError::new(
                    "E_LOCKFILE_UNSUPPORTED_VERSION",
                    format!("unsupported lockfile version: {}", lock.version),
                )
                .with_details(serde_json::json!({
                    "path": path.to_string_lossy(),
                    "version": lock.version,
                    "supported": [LOCKFILE_VERSION],
                    "reason_code": "lockfile_unsupported_version",
                    "next_actions": ["upgrade_agentpack", "regenerate_lockfile", "retry_command"],
                    "hint": "upgrade agentpack or regenerate agentpack.lock.json with `agentpack lock`",
                })),
            ));
        }
        Ok(lock)
    }

    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        let mut out = serde_json::to_string_pretty(self).context("serialize lockfile")?;
        if !out.ends_with('\n') {
            out.push('\n');
        }
        write_atomic(path, out.as_bytes()).with_context(|| format!("write {}", path.display()))?;
        Ok(())
    }
}

pub fn generate_lockfile(
    repo: &RepoPaths,
    manifest: &Manifest,
    store: &Store,
) -> anyhow::Result<Lockfile> {
    let repo_root = manifest.repo_root(&repo.manifest_path);
    let mut locked_modules = Vec::new();

    for module in &manifest.modules {
        if !module.enabled {
            continue;
        }

        let (resolved_source, resolved_version, module_root) = match module.source.kind() {
            SourceKind::LocalPath => {
                let lp: &LocalPathSource = module
                    .source
                    .local_path
                    .as_ref()
                    .context("missing local_path")?;
                let abs = repo_root.join(&lp.path);
                let rel = lp.path.replace('\\', "/");
                (
                    ResolvedSource {
                        local_path: Some(ResolvedLocalPathSource { path: rel }),
                        git: None,
                    },
                    "local".to_string(),
                    abs,
                )
            }
            SourceKind::Git => {
                let gs: &GitSource = module.source.git.as_ref().context("missing git source")?;
                let commit = store.resolve_git_commit(gs)?;
                let checkout = store.ensure_git_checkout(&module.id, gs, &commit)?;
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
            SourceKind::Invalid => anyhow::bail!("invalid source for module {}", module.id),
        };

        let (file_manifest, module_hash) = hash_tree(&module_root)
            .with_context(|| format!("hash module {} at {}", module.id, module_root.display()))?;

        locked_modules.push(LockedModule {
            id: module.id.clone(),
            module_type: module.module_type.clone(),
            resolved_source,
            resolved_version,
            sha256: module_hash,
            file_manifest,
        });
    }

    locked_modules.sort_by(|a, b| a.id.cmp(&b.id));
    let generated_at = time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .context("format timestamp")?;

    Ok(Lockfile {
        version: LOCKFILE_VERSION,
        generated_at,
        modules: locked_modules,
    })
}

pub fn hash_tree(root: &Path) -> anyhow::Result<(Vec<FileEntry>, String)> {
    if root.is_file() {
        let file_name = root
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("file")
            .to_string();
        let bytes = std::fs::read(root).with_context(|| format!("read {}", root.display()))?;
        let sha = sha256_hex(&bytes);
        let entry = FileEntry {
            path: file_name,
            sha256: sha.clone(),
            bytes: bytes.len() as u64,
        };
        let module_hash =
            sha256_hex(format!("{}\n{}\n{}\n", entry.path, entry.sha256, entry.bytes).as_bytes());
        return Ok((vec![entry], module_hash));
    }

    let mut files = Vec::new();
    for e in WalkDir::new(root).follow_links(false) {
        let e = e?;
        if e.file_type().is_dir() {
            if e.file_name() == ".git" {
                continue;
            }
            continue;
        }
        if e.path().components().any(|c| c.as_os_str() == ".git") {
            continue;
        }
        let rel = e
            .path()
            .strip_prefix(root)
            .unwrap_or(e.path())
            .to_string_lossy()
            .replace('\\', "/");
        let bytes =
            std::fs::read(e.path()).with_context(|| format!("read {}", e.path().display()))?;
        let sha = sha256_hex(&bytes);
        files.push(FileEntry {
            path: rel,
            sha256: sha,
            bytes: bytes.len() as u64,
        });
    }

    files.sort_by(|a, b| a.path.cmp(&b.path));
    let mut module_hasher = sha2::Sha256::new();
    for f in &files {
        module_hasher.update(f.path.as_bytes());
        module_hasher.update(b"\n");
        module_hasher.update(f.sha256.as_bytes());
        module_hasher.update(b"\n");
        module_hasher.update(f.bytes.to_string().as_bytes());
        module_hasher.update(b"\n");
    }
    let module_hash = hex::encode(module_hasher.finalize());
    Ok((files, module_hash))
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut h = sha2::Sha256::new();
    h.update(bytes);
    hex::encode(h.finalize())
}
