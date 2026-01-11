use std::path::{Path, PathBuf};

use anyhow::Context as _;
use serde::{Deserialize, Serialize};

use crate::paths::AgentpackHome;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManagedFile {
    pub target: String,
    pub path: String,
    pub sha256: String,
}

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
    #[serde(default = "default_snapshot_kind")]
    pub kind: String,
    pub id: String,
    pub created_at: String,
    pub targets: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub managed_files: Vec<ManagedFile>,
    pub changes: Vec<AppliedChange>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rolled_back_to: Option<String>,
    pub lockfile_sha256: Option<String>,
    pub backup_root: String,
}

fn default_snapshot_kind() -> String {
    "deploy".to_string()
}

impl DeploymentSnapshot {
    pub fn path(home: &AgentpackHome, id: &str) -> PathBuf {
        home.snapshots_dir.join(format!("{id}.json"))
    }

    pub fn backup_root(home: &AgentpackHome, id: &str) -> PathBuf {
        home.snapshots_dir.join(id).join("backup")
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

pub fn latest_snapshot(
    home: &AgentpackHome,
    kinds: &[&str],
) -> anyhow::Result<Option<DeploymentSnapshot>> {
    if !home.snapshots_dir.exists() {
        return Ok(None);
    }

    let mut best: Option<(std::time::SystemTime, DeploymentSnapshot)> = None;
    for entry in std::fs::read_dir(&home.snapshots_dir)
        .with_context(|| format!("read {}", home.snapshots_dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        let meta = entry.metadata()?;
        let modified = meta.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH);
        let snapshot = DeploymentSnapshot::load(&path)?;
        if !kinds.is_empty() && !kinds.iter().any(|k| snapshot.kind == *k) {
            continue;
        }
        match &best {
            Some((best_time, _)) if *best_time > modified => {}
            Some((best_time, best_snapshot))
                if *best_time == modified && best_snapshot.id >= snapshot.id => {}
            _ => best = Some((modified, snapshot)),
        }
    }

    Ok(best.map(|(_, snapshot)| snapshot))
}
