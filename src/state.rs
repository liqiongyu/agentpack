use std::path::{Path, PathBuf};

use anyhow::Context as _;
use serde::{Deserialize, Serialize};

use crate::paths::AgentpackHome;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppliedChange {
    pub target: String,
    pub op: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backup_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before_sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after_sha256: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentSnapshot {
    pub id: String,
    pub created_at: String,
    pub targets: Vec<String>,
    pub changes: Vec<AppliedChange>,
    pub lockfile_sha256: Option<String>,
    pub backup_root: String,
}

impl DeploymentSnapshot {
    pub fn path(home: &AgentpackHome, id: &str) -> PathBuf {
        home.deployments_dir.join(format!("{id}.json"))
    }

    pub fn backup_root(home: &AgentpackHome, id: &str) -> PathBuf {
        home.deployments_dir.join(id).join("backup")
    }

    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let raw =
            std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
        serde_json::from_str(&raw).context("parse snapshot json")
    }

    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        let mut out = serde_json::to_string_pretty(self).context("serialize snapshot")?;
        if !out.ends_with('\n') {
            out.push('\n');
        }
        std::fs::write(path, out).with_context(|| format!("write {}", path.display()))?;
        Ok(())
    }
}
