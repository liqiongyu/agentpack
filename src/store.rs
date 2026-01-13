use std::path::{Path, PathBuf};

use anyhow::Context as _;

use crate::config::GitSource;
use crate::git::{clone_checkout_git, resolve_git_ref};
use crate::hash::sha256_hex;
use crate::paths::AgentpackHome;

#[derive(Debug, Clone)]
pub struct Store {
    root: PathBuf,
}

impl Store {
    pub fn new(home: &AgentpackHome) -> Self {
        Self {
            root: home.cache_dir.clone(),
        }
    }

    pub fn ensure_layout(&self) -> anyhow::Result<()> {
        std::fs::create_dir_all(self.root.join("git")).context("create store dir")?;
        Ok(())
    }

    pub fn resolve_git_commit(&self, src: &GitSource) -> anyhow::Result<String> {
        resolve_git_ref(&src.url, &src.ref_name)
    }

    pub fn git_checkout_dir(&self, url: &str, commit: &str) -> PathBuf {
        self.git_checkout_dir_v3(url, commit)
    }

    fn git_checkout_dir_v3(&self, url: &str, commit: &str) -> PathBuf {
        self.root
            .join("git")
            .join(sha256_hex(url.as_bytes()))
            .join(commit)
    }

    fn git_checkout_dir_v2(&self, module_id: &str, commit: &str) -> PathBuf {
        self.root
            .join("git")
            .join(crate::ids::module_fs_key(module_id))
            .join(commit)
    }

    fn git_checkout_dir_v2_legacy_fs_key(&self, module_id: &str, commit: &str) -> Option<PathBuf> {
        let bounded = crate::ids::module_fs_key(module_id);
        let unbounded = crate::ids::module_fs_key_unbounded(module_id);
        if bounded == unbounded {
            return None;
        }

        Some(self.root.join("git").join(unbounded).join(commit))
    }

    fn git_checkout_dir_legacy(&self, module_id: &str, commit: &str) -> PathBuf {
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
        let canonical = self.git_checkout_dir_v3(&src.url, commit);
        if canonical.exists() {
            return Ok(canonical);
        }

        let migrate_or_use_legacy = |legacy: PathBuf| -> anyhow::Result<PathBuf> {
            if canonical.exists() {
                return Ok(canonical.clone());
            }
            let Some(parent) = canonical.parent() else {
                anyhow::bail!(
                    "canonical checkout dir missing parent: {}",
                    canonical.display()
                );
            };
            std::fs::create_dir_all(parent).context("create canonical checkout dir parent")?;
            match std::fs::rename(&legacy, &canonical) {
                Ok(()) => Ok(canonical.clone()),
                Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => {
                    Ok(canonical.clone())
                }
                Err(_) => Ok(legacy),
            }
        };

        let v2 = self.git_checkout_dir_v2(module_id, commit);
        if v2.exists() {
            return migrate_or_use_legacy(v2);
        }
        if let Some(legacy_fs_key) = self.git_checkout_dir_v2_legacy_fs_key(module_id, commit) {
            if legacy_fs_key.exists() {
                return migrate_or_use_legacy(legacy_fs_key);
            }
        }
        let legacy = self.git_checkout_dir_legacy(module_id, commit);
        if legacy.exists() {
            return migrate_or_use_legacy(legacy);
        }

        clone_checkout_git(&src.url, &src.ref_name, commit, &canonical, src.shallow)?;
        Ok(canonical)
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
