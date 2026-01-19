use std::path::{Path, PathBuf};

use anyhow::Context as _;
use serde::{Deserialize, Serialize};

use crate::config::{GitSource, Manifest, Module, SourceKind};
use crate::fs::{copy_tree, copy_tree_missing_only, write_atomic};
use crate::lockfile::{FileEntry, Lockfile, hash_tree};
use crate::paths::{AgentpackHome, RepoPaths};
use crate::store::Store;
use crate::user_error::UserError;

mod util;
pub(super) use util::{
    delete_overlay_file, join_posix, path_relative_posix, validate_posix_relpath,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum OverlayKind {
    Dir,
    Patch,
}

fn default_overlay_kind() -> OverlayKind {
    OverlayKind::Dir
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct OverlayMeta {
    #[serde(default = "default_overlay_kind")]
    pub(super) overlay_kind: OverlayKind,
}

#[derive(Debug, Serialize, Deserialize)]
pub(super) struct OverlayBaseline {
    pub(super) version: u32,
    pub(super) created_at: String,
    pub(super) upstream_sha256: String,
    pub(super) file_manifest: Vec<FileEntry>,
    #[serde(default)]
    pub(super) upstream: Option<BaselineUpstream>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(super) enum BaselineUpstream {
    Git {
        url: String,
        commit: String,
        #[serde(default)]
        subdir: String,
    },
    LocalPath {
        /// The upstream module root path, relative to the config repo root (POSIX style).
        repo_rel_path: String,
        #[serde(default)]
        repo_git_rev: Option<String>,
        #[serde(default)]
        repo_dirty: Option<bool>,
    },
}

pub(super) fn overlay_baseline_path(overlay_dir: &Path) -> PathBuf {
    overlay_dir.join(".agentpack").join("baseline.json")
}

pub(super) fn overlay_module_id_path(overlay_dir: &Path) -> PathBuf {
    overlay_dir.join(".agentpack").join("module_id")
}

pub(super) fn overlay_meta_path(overlay_dir: &Path) -> PathBuf {
    overlay_dir.join(".agentpack").join("overlay.json")
}

pub(super) fn write_overlay_module_id(module_id: &str, overlay_dir: &Path) -> anyhow::Result<()> {
    let meta_dir = overlay_dir.join(".agentpack");
    std::fs::create_dir_all(&meta_dir).context("create overlay metadata dir")?;

    let path = overlay_module_id_path(overlay_dir);
    let mut content = module_id.to_string();
    if !content.ends_with('\n') {
        content.push('\n');
    }
    write_atomic(&path, content.as_bytes()).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

pub(super) fn write_overlay_meta_default_dir(overlay_dir: &Path) -> anyhow::Result<()> {
    write_overlay_meta(overlay_dir, OverlayKind::Dir)
}

pub(super) fn write_overlay_meta(
    overlay_dir: &Path,
    overlay_kind: OverlayKind,
) -> anyhow::Result<()> {
    let meta_dir = overlay_dir.join(".agentpack");
    std::fs::create_dir_all(&meta_dir).context("create overlay metadata dir")?;

    let path = overlay_meta_path(overlay_dir);
    let meta = OverlayMeta { overlay_kind };

    let mut out = serde_json::to_string_pretty(&meta).context("serialize overlay meta")?;
    if !out.ends_with('\n') {
        out.push('\n');
    }

    write_atomic(&path, out.as_bytes()).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

pub(super) fn read_overlay_meta(overlay_dir: &Path) -> anyhow::Result<OverlayMeta> {
    let path = overlay_meta_path(overlay_dir);
    if !path.exists() {
        return Ok(OverlayMeta {
            overlay_kind: OverlayKind::Dir,
        });
    }

    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let meta: OverlayMeta = serde_json::from_str(&raw).map_err(|err| {
        anyhow::Error::new(
            UserError::new(
                "E_CONFIG_INVALID",
                format!(
                    "invalid overlay metadata (expected JSON) at {}: {}",
                    path.display(),
                    err
                ),
            )
            .with_details(serde_json::json!({
                "overlay_dir": overlay_dir.to_string_lossy(),
                "meta_path": path.to_string_lossy(),
                "hint": "delete the file to fall back to overlay_kind=dir, or fix it to include {\"overlay_kind\": \"dir\"|\"patch\"}",
            })),
        )
    })?;

    Ok(meta)
}

pub fn ensure_overlay_skeleton(
    home: &AgentpackHome,
    repo: &RepoPaths,
    manifest: &Manifest,
    module_id: &str,
    overlay_dir: &Path,
) -> anyhow::Result<super::OverlaySkeleton> {
    ensure_overlay_skeleton_impl(home, repo, manifest, module_id, overlay_dir, true)
}

pub fn ensure_overlay_skeleton_sparse(
    home: &AgentpackHome,
    repo: &RepoPaths,
    manifest: &Manifest,
    module_id: &str,
    overlay_dir: &Path,
) -> anyhow::Result<super::OverlaySkeleton> {
    ensure_overlay_skeleton_impl(home, repo, manifest, module_id, overlay_dir, false)
}

pub fn materialize_overlay_from_upstream(
    home: &AgentpackHome,
    repo: &RepoPaths,
    manifest: &Manifest,
    module_id: &str,
    overlay_dir: &Path,
) -> anyhow::Result<()> {
    let module = manifest
        .modules
        .iter()
        .find(|m| m.id == module_id)
        .with_context(|| format!("module not found: {module_id}"))?;

    let upstream_root = resolve_upstream_module_root(home, repo, module)?;

    std::fs::create_dir_all(overlay_dir).context("create overlay dir")?;
    copy_tree_missing_only(&upstream_root, overlay_dir).with_context(|| {
        format!(
            "materialize upstream {} -> {}",
            upstream_root.display(),
            overlay_dir.display()
        )
    })?;

    if !overlay_baseline_path(overlay_dir).exists() {
        write_overlay_baseline(home, repo, module, &upstream_root, overlay_dir)?;
    }
    if !overlay_module_id_path(overlay_dir).exists() {
        write_overlay_module_id(module_id, overlay_dir)?;
    }
    if !overlay_meta_path(overlay_dir).exists() {
        write_overlay_meta_default_dir(overlay_dir)?;
    }

    Ok(())
}

fn ensure_overlay_skeleton_impl(
    home: &AgentpackHome,
    repo: &RepoPaths,
    manifest: &Manifest,
    module_id: &str,
    overlay_dir: &Path,
    copy_upstream: bool,
) -> anyhow::Result<super::OverlaySkeleton> {
    let module = manifest
        .modules
        .iter()
        .find(|m| m.id == module_id)
        .with_context(|| format!("module not found: {module_id}"))?;

    let upstream_root = resolve_upstream_module_root(home, repo, module)?;

    let created = !overlay_dir.exists();
    if created {
        std::fs::create_dir_all(overlay_dir).context("create overlay dir")?;
        if copy_upstream {
            copy_tree(&upstream_root, overlay_dir).with_context(|| {
                format!(
                    "copy upstream {} -> {}",
                    upstream_root.display(),
                    overlay_dir.display()
                )
            })?;
        }
    }

    if !overlay_baseline_path(overlay_dir).exists() {
        write_overlay_baseline(home, repo, module, &upstream_root, overlay_dir)?;
    }
    if !overlay_module_id_path(overlay_dir).exists() {
        write_overlay_module_id(module_id, overlay_dir)?;
    }
    if !overlay_meta_path(overlay_dir).exists() {
        write_overlay_meta_default_dir(overlay_dir)?;
    }

    Ok(super::OverlaySkeleton {
        dir: overlay_dir.to_path_buf(),
        created,
    })
}

pub fn resolve_upstream_module_root(
    home: &AgentpackHome,
    repo: &RepoPaths,
    module: &Module,
) -> anyhow::Result<PathBuf> {
    let repo_root = repo
        .manifest_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| repo.repo_dir.clone());

    match module.source.kind() {
        SourceKind::LocalPath => {
            let lp = module
                .source
                .local_path
                .as_ref()
                .context("missing local_path")?;
            Ok(repo_root.join(&lp.path))
        }
        SourceKind::Git => {
            // Prefer lockfile resolution for reproducibility.
            let lock = Lockfile::load(&repo.lockfile_path).ok();
            if let Some(lock) = lock {
                if let Some(lm) = lock.modules.iter().find(|m| m.id == module.id) {
                    if let Some(gs) = &lm.resolved_source.git {
                        let store = Store::new(home);
                        let checkout_dir = ensure_locked_git_checkout(&store, &module.id, gs)?;
                        let root = Store::module_root_in_checkout(&checkout_dir, &gs.subdir);
                        return Ok(root);
                    }
                }
            }

            // Fallback to manifest ref resolution.
            let src = module.source.git.as_ref().context("missing git source")?;
            let store = Store::new(home);
            let commit = store.resolve_git_commit(src)?;
            let checkout_dir = store.ensure_git_checkout(&module.id, src, &commit)?;
            Ok(Store::module_root_in_checkout(&checkout_dir, &src.subdir))
        }
        SourceKind::Invalid => anyhow::bail!("invalid source for module {}", module.id),
    }
}

fn ensure_locked_git_checkout(
    store: &Store,
    module_id: &str,
    locked: &crate::lockfile::ResolvedGitSource,
) -> anyhow::Result<PathBuf> {
    // Lockfile stores the exact commit; use the commit itself as the ref name to avoid
    // requiring the original branch/tag in order to populate the checkout.
    let src = GitSource {
        url: locked.url.clone(),
        ref_name: locked.commit.clone(),
        subdir: locked.subdir.clone(),
        shallow: false,
    };
    store.ensure_git_checkout(module_id, &src, &locked.commit)
}

pub(super) fn write_overlay_baseline(
    home: &AgentpackHome,
    repo: &RepoPaths,
    module: &Module,
    upstream_root: &Path,
    overlay_dir: &Path,
) -> anyhow::Result<()> {
    let (file_manifest, module_hash) = hash_tree(upstream_root)
        .with_context(|| format!("hash upstream {}", upstream_root.display()))?;

    let created_at = time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .context("format timestamp")?;

    let upstream = match module.source.kind() {
        SourceKind::Git => write_baseline_upstream_git(home, repo, module)?,
        SourceKind::LocalPath => write_baseline_upstream_local(repo, upstream_root)?,
        SourceKind::Invalid => None,
    };

    let baseline = OverlayBaseline {
        version: 2,
        created_at,
        upstream_sha256: module_hash,
        file_manifest,
        upstream,
    };

    let meta_dir = overlay_dir.join(".agentpack");
    std::fs::create_dir_all(&meta_dir).context("create overlay metadata dir")?;

    let baseline_path = overlay_baseline_path(overlay_dir);
    let mut out = serde_json::to_string_pretty(&baseline).context("serialize baseline")?;
    if !out.ends_with('\n') {
        out.push('\n');
    }
    write_atomic(&baseline_path, out.as_bytes())
        .with_context(|| format!("write {}", baseline_path.display()))?;
    Ok(())
}

fn write_baseline_upstream_git(
    home: &AgentpackHome,
    repo: &RepoPaths,
    module: &Module,
) -> anyhow::Result<Option<BaselineUpstream>> {
    let lock = Lockfile::load(&repo.lockfile_path).ok();
    if let Some(lock) = lock {
        if let Some(lm) = lock.modules.iter().find(|m| m.id == module.id) {
            if let Some(gs) = &lm.resolved_source.git {
                return Ok(Some(BaselineUpstream::Git {
                    url: gs.url.clone(),
                    commit: gs.commit.clone(),
                    subdir: gs.subdir.clone(),
                }));
            }
        }
    }

    let src = module.source.git.as_ref().context("missing git source")?;
    let store = Store::new(home);
    let commit = store.resolve_git_commit(src)?;
    Ok(Some(BaselineUpstream::Git {
        url: src.url.clone(),
        commit,
        subdir: src.subdir.clone(),
    }))
}

fn write_baseline_upstream_local(
    repo: &RepoPaths,
    upstream_root: &Path,
) -> anyhow::Result<Option<BaselineUpstream>> {
    let repo_root = repo
        .manifest_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| repo.repo_dir.clone());

    let repo_rel_path = upstream_root
        .strip_prefix(&repo_root)
        .ok()
        .map(|p| p.to_string_lossy().replace('\\', "/"));
    let Some(repo_rel_path) = repo_rel_path else {
        return Ok(None);
    };

    let repo_git_rev = crate::git::git_in(&repo_root, &["rev-parse", "HEAD"])
        .ok()
        .map(|s| s.trim().to_string());
    let repo_dirty = crate::git::git_in(&repo_root, &["status", "--porcelain"])
        .ok()
        .map(|s| !s.trim().is_empty());

    Ok(Some(BaselineUpstream::LocalPath {
        repo_rel_path,
        repo_git_rev,
        repo_dirty,
    }))
}
