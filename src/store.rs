use std::path::{Path, PathBuf};

use anyhow::Context as _;

use crate::config::GitSource;
use crate::git::{clone_checkout_git, resolve_git_ref};
use crate::paths::AgentpackHome;

#[derive(Debug, Clone)]
pub struct Store {
    root: PathBuf,
}

impl Store {
    pub fn new(home: &AgentpackHome) -> Self {
        Self {
            root: home.store_dir.clone(),
        }
    }

    pub fn ensure_layout(&self) -> anyhow::Result<()> {
        std::fs::create_dir_all(self.root.join("git")).context("create store dir")?;
        Ok(())
    }

    pub fn resolve_git_commit(&self, src: &GitSource) -> anyhow::Result<String> {
        resolve_git_ref(&src.url, &src.ref_name)
    }

    pub fn git_checkout_dir(&self, module_id: &str, commit: &str) -> PathBuf {
        self.root
            .join("git")
            .join(sanitize_module_id(module_id))
            .join(commit)
    }

    pub fn ensure_git_checkout(
        &self,
        module_id: &str,
        src: &GitSource,
        commit: &str,
    ) -> anyhow::Result<PathBuf> {
        self.ensure_layout()?;
        let dir = self.git_checkout_dir(module_id, commit);
        clone_checkout_git(&src.url, &src.ref_name, commit, &dir, src.shallow)?;
        Ok(dir)
    }

    pub fn module_root_in_checkout(checkout_dir: &Path, subdir: &str) -> PathBuf {
        if subdir.trim().is_empty() {
            checkout_dir.to_path_buf()
        } else {
            checkout_dir.join(subdir)
        }
    }
}

pub fn sanitize_module_id(module_id: &str) -> String {
    module_id
        .chars()
        .map(|c| match c {
            ':' | '/' | '\\' => '_',
            _ if c.is_ascii_alphanumeric() || c == '-' || c == '_' => c,
            _ => '_',
        })
        .collect()
}
