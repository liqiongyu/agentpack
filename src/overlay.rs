use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::Context as _;
use serde::{Deserialize, Serialize};

use crate::config::GitSource;
use crate::config::{Manifest, Module, SourceKind};
use crate::fs::{copy_tree, list_files};
use crate::lockfile::{FileEntry, Lockfile, hash_tree};
use crate::paths::{AgentpackHome, RepoPaths};
use crate::project::ProjectContext;
use crate::store::Store;

#[derive(Debug, Clone)]
pub struct OverlayPaths {
    pub global_dir: PathBuf,
    pub project_dir: PathBuf,
}

#[derive(Debug, Clone)]
pub struct OverlaySkeleton {
    pub dir: PathBuf,
    pub created: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct OverlayBaseline {
    version: u32,
    created_at: String,
    upstream_sha256: String,
    file_manifest: Vec<FileEntry>,
}

pub fn resolve_overlay_paths(
    repo: &RepoPaths,
    project: &ProjectContext,
    module_id: &str,
) -> OverlayPaths {
    OverlayPaths {
        global_dir: repo.repo_dir.join("overlays").join(module_id),
        project_dir: repo
            .repo_dir
            .join("projects")
            .join(&project.project_id)
            .join("overlays")
            .join(module_id),
    }
}

pub fn ensure_overlay_skeleton(
    home: &AgentpackHome,
    repo: &RepoPaths,
    manifest: &Manifest,
    module_id: &str,
    project: Option<&ProjectContext>,
) -> anyhow::Result<OverlaySkeleton> {
    let module = manifest
        .modules
        .iter()
        .find(|m| m.id == module_id)
        .with_context(|| format!("module not found: {module_id}"))?;

    let upstream_root = resolve_upstream_module_root(home, repo, module)?;

    let overlay_dir = if let Some(project) = project {
        resolve_overlay_paths(repo, project, module_id).project_dir
    } else {
        repo.repo_dir.join("overlays").join(module_id)
    };

    let created = !overlay_dir.exists();
    if created {
        std::fs::create_dir_all(&overlay_dir).context("create overlay dir")?;
        copy_tree(&upstream_root, &overlay_dir).with_context(|| {
            format!(
                "copy upstream {} -> {}",
                upstream_root.display(),
                overlay_dir.display()
            )
        })?;
    }

    if !overlay_baseline_path(&overlay_dir).exists() {
        write_overlay_baseline(&upstream_root, &overlay_dir)?;
    }

    Ok(OverlaySkeleton {
        dir: overlay_dir,
        created,
    })
}

pub fn compose_module_tree(
    upstream_root: &Path,
    overlays: &[&Path],
    out_dir: &Path,
) -> anyhow::Result<()> {
    std::fs::create_dir_all(out_dir).context("create module out dir")?;
    copy_tree(upstream_root, out_dir).context("copy upstream")?;
    for overlay in overlays {
        if overlay.exists() {
            copy_tree(overlay, out_dir)
                .with_context(|| format!("apply overlay {}", overlay.display()))?;
        }
    }
    Ok(())
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
    let checkout_dir = store.git_checkout_dir(module_id, &locked.commit);
    if checkout_dir.exists() {
        return Ok(checkout_dir);
    }

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

pub fn overlay_drift_warnings(
    module_id: &str,
    overlay_kind: &str,
    upstream_root: &Path,
    overlay_dir: &Path,
) -> anyhow::Result<Vec<String>> {
    if !overlay_dir.exists() {
        return Ok(Vec::new());
    }

    let baseline_path = overlay_baseline_path(overlay_dir);
    if !baseline_path.exists() {
        return Ok(Vec::new());
    }

    let raw = std::fs::read_to_string(&baseline_path)
        .with_context(|| format!("read {}", baseline_path.display()))?;
    let baseline: OverlayBaseline = serde_json::from_str(&raw).context("parse overlay baseline")?;
    let baseline_hash = baseline.upstream_sha256.clone();

    let (current_manifest, current_hash) = hash_tree(upstream_root)
        .with_context(|| format!("hash upstream {}", upstream_root.display()))?;

    let baseline_map: BTreeMap<String, String> = baseline
        .file_manifest
        .iter()
        .map(|f| (f.path.clone(), f.sha256.clone()))
        .collect();
    let current_map: BTreeMap<String, String> = current_manifest
        .into_iter()
        .map(|f| (f.path, f.sha256))
        .collect();

    let mut overlay_files = list_files(overlay_dir)?;
    overlay_files.sort();

    let mut warnings = Vec::new();
    for file in overlay_files {
        let rel = file
            .strip_prefix(overlay_dir)
            .unwrap_or(&file)
            .to_string_lossy()
            .replace('\\', "/");

        let Some(baseline_sha) = baseline_map.get(&rel) else {
            continue;
        };

        match current_map.get(&rel) {
            Some(current_sha) if current_sha != baseline_sha => warnings.push(format!(
                "overlay drift ({overlay_kind}) module {module_id}: upstream changed for {rel} ({baseline_sha} -> {current_sha})"
            )),
            None => warnings.push(format!(
                "overlay drift ({overlay_kind}) module {module_id}: upstream removed {rel} (baseline {baseline_sha})"
            )),
            _ => {}
        }
    }

    if warnings.is_empty() && baseline_hash != current_hash {
        warnings.push(format!(
            "overlay drift ({overlay_kind}) module {module_id}: upstream changed ({baseline} -> {current})",
            baseline = baseline_hash,
            current = current_hash
        ));
    }

    Ok(warnings)
}

fn overlay_baseline_path(overlay_dir: &Path) -> PathBuf {
    overlay_dir.join(".agentpack").join("baseline.json")
}

fn write_overlay_baseline(upstream_root: &Path, overlay_dir: &Path) -> anyhow::Result<()> {
    let (file_manifest, module_hash) = hash_tree(upstream_root)
        .with_context(|| format!("hash upstream {}", upstream_root.display()))?;

    let created_at = time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .context("format timestamp")?;

    let baseline = OverlayBaseline {
        version: 1,
        created_at,
        upstream_sha256: module_hash,
        file_manifest,
    };

    let meta_dir = overlay_dir.join(".agentpack");
    std::fs::create_dir_all(&meta_dir).context("create overlay metadata dir")?;

    let baseline_path = overlay_baseline_path(overlay_dir);
    let mut out = serde_json::to_string_pretty(&baseline).context("serialize baseline")?;
    if !out.ends_with('\n') {
        out.push('\n');
    }
    std::fs::write(&baseline_path, out)
        .with_context(|| format!("write {}", baseline_path.display()))?;
    Ok(())
}
