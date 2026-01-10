use std::path::{Path, PathBuf};

use anyhow::Context as _;
use sha2::Digest as _;

use crate::git::git_in;

#[derive(Debug, Clone)]
pub struct ProjectContext {
    pub cwd: PathBuf,
    pub project_root: PathBuf,
    pub project_id: String,
    pub origin_url: Option<String>,
}

impl ProjectContext {
    pub fn detect(cwd: &Path) -> anyhow::Result<Self> {
        let cwd = cwd.to_path_buf();
        let (project_root, origin_url) = match detect_git_root(&cwd) {
            Ok(root) => {
                let origin = git_in(&root, &["remote", "get-url", "origin"]).ok();
                (
                    root,
                    origin
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty()),
                )
            }
            Err(_) => (cwd.clone(), None),
        };

        let project_id = compute_project_id(origin_url.as_deref(), &project_root)?;
        Ok(Self {
            cwd,
            project_root,
            project_id,
            origin_url,
        })
    }
}

fn detect_git_root(cwd: &Path) -> anyhow::Result<PathBuf> {
    let out = git_in(cwd, &["rev-parse", "--show-toplevel"]).context("git rev-parse")?;
    Ok(PathBuf::from(out.trim()))
}

fn compute_project_id(origin_url: Option<&str>, project_root: &Path) -> anyhow::Result<String> {
    let basis = if let Some(u) = origin_url {
        normalize_git_remote(u)
    } else {
        project_root
            .canonicalize()
            .unwrap_or_else(|_| project_root.to_path_buf())
            .to_string_lossy()
            .to_string()
    };

    let mut hasher = sha2::Sha256::new();
    hasher.update(basis.as_bytes());
    let hex = hex::encode(hasher.finalize());
    Ok(hex.chars().take(16).collect())
}

fn normalize_git_remote(url: &str) -> String {
    let u = url.trim().trim_end_matches(".git");
    // Basic normalization:
    // - strip protocol/userinfo
    // - map ssh form git@github.com:org/repo -> github.com/org/repo
    if let Some(rest) = u.strip_prefix("git@") {
        let rest = rest.replace(':', "/");
        return rest.to_lowercase();
    }
    if let Some(rest) = u.strip_prefix("https://") {
        return rest.to_lowercase();
    }
    if let Some(rest) = u.strip_prefix("http://") {
        return rest.to_lowercase();
    }
    u.to_lowercase()
}
