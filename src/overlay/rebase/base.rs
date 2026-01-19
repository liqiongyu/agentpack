use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::Context as _;

use crate::config::{GitSource, Module, SourceKind};
use crate::lockfile::hash_tree;
use crate::paths::AgentpackHome;
use crate::store::Store;
use crate::user_error::UserError;

use super::super::layout::{BaselineUpstream, OverlayBaseline};

pub(super) enum RebaseBase {
    Dir(PathBuf),
    RepoGit {
        repo_git_rev: String,
        repo_rel_path: String,
    },
}

pub(super) fn resolve_rebase_base(
    home: &AgentpackHome,
    module: &Module,
    baseline: &OverlayBaseline,
    repo_root: &Path,
) -> anyhow::Result<RebaseBase> {
    if let Some(upstream) = baseline.upstream.clone() {
        match upstream {
            BaselineUpstream::Git {
                url,
                commit,
                subdir,
            } => {
                let store = Store::new(home);
                let src = GitSource {
                    url,
                    ref_name: commit.clone(),
                    subdir: subdir.clone(),
                    shallow: false,
                };
                let checkout_dir = store.ensure_git_checkout(&module.id, &src, &commit)?;
                return Ok(RebaseBase::Dir(Store::module_root_in_checkout(
                    &checkout_dir,
                    &subdir,
                )));
            }
            BaselineUpstream::LocalPath {
                repo_rel_path,
                repo_git_rev,
                repo_dirty: _,
            } => {
                let Some(repo_git_rev) = repo_git_rev else {
                    return Err(anyhow::Error::new(
                        UserError::new(
                            "E_OVERLAY_BASELINE_UNSUPPORTED",
                            "overlay baseline is missing repo git revision; cannot locate merge base"
                                .to_string(),
                        )
                        .with_details(serde_json::json!({
                            "repo_rel_path": repo_rel_path,
                            "hint": "ensure the config repo is a git repo and recreate the overlay baseline",
                        })),
                    ));
                };
                return Ok(RebaseBase::RepoGit {
                    repo_git_rev,
                    repo_rel_path,
                });
            }
        }
    }

    // Backwards-compatibility: older baselines don't include upstream identity.
    if module.source.kind() == SourceKind::Git {
        let subdir = module
            .source
            .git
            .as_ref()
            .map(|g| g.subdir.clone())
            .unwrap_or_default();
        if let Some((_, root)) =
            find_git_checkout_matching_hash(home, &module.id, &subdir, &baseline.upstream_sha256)?
        {
            return Ok(RebaseBase::Dir(root));
        }
    }

    if module.source.kind() == SourceKind::LocalPath && repo_root.join(".git").exists() {
        return Err(anyhow::Error::new(
            UserError::new(
                "E_OVERLAY_BASELINE_UNSUPPORTED",
                "overlay baseline is missing upstream identity; cannot locate merge base"
                    .to_string(),
            )
            .with_details(serde_json::json!({
                "hint": "recreate the overlay baseline (agentpack overlay edit) after committing the repo state",
            })),
        ));
    }

    Err(anyhow::Error::new(UserError::new(
        "E_OVERLAY_BASELINE_UNSUPPORTED",
        "overlay baseline is missing upstream identity; cannot locate merge base".to_string(),
    )))
}

fn find_git_checkout_matching_hash(
    home: &AgentpackHome,
    module_id: &str,
    subdir: &str,
    baseline_hash: &str,
) -> anyhow::Result<Option<(String, PathBuf)>> {
    let dir = home
        .cache_dir
        .join("git")
        .join(crate::ids::module_fs_key(module_id));
    let entries = match std::fs::read_dir(&dir) {
        Ok(e) => e,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(err) => return Err(err).with_context(|| format!("read_dir {}", dir.display())),
    };

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let commit = entry.file_name().to_string_lossy().to_string();
        let root = Store::module_root_in_checkout(&path, subdir);
        if !root.exists() {
            continue;
        }
        let (_, hash) = hash_tree(&root).with_context(|| format!("hash {}", root.display()))?;
        if hash == baseline_hash {
            return Ok(Some((commit, root)));
        }
    }

    Ok(None)
}

pub(super) fn read_optional_file_bytes(
    root: &Path,
    rel_posix: &str,
) -> anyhow::Result<Option<Vec<u8>>> {
    let path = root.join(rel_posix);
    match std::fs::read(&path) {
        Ok(b) => Ok(Some(b)),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(err).with_context(|| format!("read {}", path.display())),
    }
}

pub(super) fn git_show_optional_bytes(
    repo_root: &Path,
    repo_git_rev: &str,
    repo_rel_root_posix: &str,
    rel_posix: &str,
) -> anyhow::Result<Option<Vec<u8>>> {
    let mut full = repo_rel_root_posix.trim_end_matches('/').to_string();
    if !full.is_empty() {
        full.push('/');
    }
    full.push_str(rel_posix);
    let spec = format!("{repo_git_rev}:{full}");

    let out = Command::new("git")
        .current_dir(repo_root)
        .arg("show")
        .arg(spec)
        .output()
        .context("git show")?;

    if out.status.success() {
        Ok(Some(out.stdout))
    } else {
        Ok(None)
    }
}
