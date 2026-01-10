use std::path::{Path, PathBuf};

use anyhow::Context as _;

use crate::config::{Manifest, Module, SourceKind};
use crate::fs::copy_tree;
use crate::lockfile::Lockfile;
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

fn resolve_upstream_module_root(
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
                        let checkout_dir = store.git_checkout_dir(&module.id, &gs.commit);
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
