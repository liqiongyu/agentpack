use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::Context as _;
use serde::Serialize;

use crate::hash::sha256_hex;
use crate::user_error::UserError;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TargetPath {
    pub target: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct DesiredFile {
    pub bytes: Vec<u8>,
    pub module_ids: Vec<String>,
}

pub type DesiredState = BTreeMap<TargetPath, DesiredFile>;
pub type ManagedPaths = BTreeSet<TargetPath>;

pub fn insert_desired_file(
    desired: &mut DesiredState,
    target: impl Into<String>,
    path: PathBuf,
    bytes: Vec<u8>,
    module_ids: Vec<String>,
) -> anyhow::Result<()> {
    let target = target.into();
    let path_str = path.to_string_lossy().to_string();
    let key = TargetPath {
        target: target.clone(),
        path,
    };

    if let Some(existing) = desired.get_mut(&key) {
        if existing.bytes == bytes {
            let mut merged = BTreeSet::new();
            merged.extend(existing.module_ids.iter().cloned());
            merged.extend(module_ids);
            existing.module_ids = merged.into_iter().collect();
            return Ok(());
        }

        let details = serde_json::json!({
            "target": target,
            "path": path_str,
            "existing": {
                "sha256": sha256_hex(&existing.bytes),
                "module_ids": existing.module_ids.clone(),
            },
            "new": {
                "sha256": sha256_hex(&bytes),
                "module_ids": module_ids,
            },
        });

        return Err(anyhow::Error::new(
            UserError::new(
                "E_DESIRED_STATE_CONFLICT",
                format!(
                    "conflicting desired outputs for {}:{}",
                    key.target,
                    key.path.display()
                ),
            )
            .with_details(details),
        ));
    }

    desired.insert(key, DesiredFile { bytes, module_ids });
    Ok(())
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Op {
    Create,
    Update,
    Delete,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlanChange {
    pub target: String,
    pub op: Op,
    pub path: String,
    pub before_sha256: Option<String>,
    pub after_sha256: Option<String>,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct PlanSummary {
    pub create: u64,
    pub update: u64,
    pub delete: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlanResult {
    pub changes: Vec<PlanChange>,
    pub summary: PlanSummary,
}

pub fn plan(desired: &DesiredState, managed: Option<&ManagedPaths>) -> anyhow::Result<PlanResult> {
    let mut changes = Vec::new();

    for (tp, desired_file) in desired {
        let after_sha = sha256_hex(&desired_file.bytes);
        match std::fs::read(&tp.path) {
            Ok(existing) => {
                let before_sha = sha256_hex(&existing);
                if before_sha != after_sha {
                    changes.push(PlanChange {
                        target: tp.target.clone(),
                        op: Op::Update,
                        path: tp.path.to_string_lossy().to_string(),
                        before_sha256: Some(before_sha),
                        after_sha256: Some(after_sha),
                        reason: "content differs".to_string(),
                    });
                }
            }
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                changes.push(PlanChange {
                    target: tp.target.clone(),
                    op: Op::Create,
                    path: tp.path.to_string_lossy().to_string(),
                    before_sha256: None,
                    after_sha256: Some(after_sha),
                    reason: "file missing".to_string(),
                });
            }
            Err(err) => {
                return Err(err).with_context(|| format!("read {}", tp.path.display()));
            }
        }
    }

    if let Some(managed) = managed {
        for tp in managed {
            if desired.contains_key(tp) {
                continue;
            }
            if tp.path.exists() {
                let before_sha = sha256_hex(&std::fs::read(&tp.path)?);
                changes.push(PlanChange {
                    target: tp.target.clone(),
                    op: Op::Delete,
                    path: tp.path.to_string_lossy().to_string(),
                    before_sha256: Some(before_sha),
                    after_sha256: None,
                    reason: "no longer managed".to_string(),
                });
            }
        }
    }

    changes.sort_by(|a, b| {
        (a.target.as_str(), a.path.as_str()).cmp(&(b.target.as_str(), b.path.as_str()))
    });

    let mut summary = PlanSummary::default();
    for c in &changes {
        match c.op {
            Op::Create => summary.create += 1,
            Op::Update => summary.update += 1,
            Op::Delete => summary.delete += 1,
        }
    }

    Ok(PlanResult { changes, summary })
}

pub fn load_managed_paths_from_snapshot(
    snapshot: &crate::state::DeploymentSnapshot,
) -> anyhow::Result<ManagedPaths> {
    let mut out = ManagedPaths::new();
    if !snapshot.managed_files.is_empty() {
        for f in &snapshot.managed_files {
            out.insert(TargetPath {
                target: f.target.clone(),
                path: PathBuf::from(&f.path),
            });
        }
        return Ok(out);
    }

    for c in &snapshot.changes {
        if c.op == "create" || c.op == "update" {
            out.insert(TargetPath {
                target: c.target.clone(),
                path: PathBuf::from(&c.path),
            });
        }
    }
    Ok(out)
}

pub fn read_text(path: &Path) -> anyhow::Result<Option<String>> {
    let bytes = std::fs::read(path)?;
    if bytes.contains(&0) {
        return Ok(None);
    }
    Ok(String::from_utf8(bytes).ok())
}
