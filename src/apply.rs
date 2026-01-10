use std::path::{Path, PathBuf};

use anyhow::Context as _;
use std::io::Write as _;
use tempfile::NamedTempFile;

use crate::deploy::{DesiredState, Op, PlanResult, TargetPath};
use crate::hash::sha256_hex;
use crate::paths::AgentpackHome;
use crate::state::{AppliedChange, DeploymentSnapshot};
use crate::store::sanitize_module_id;

pub fn apply_plan(
    home: &AgentpackHome,
    plan: &PlanResult,
    desired: &DesiredState,
    lockfile_path: Option<&Path>,
) -> anyhow::Result<DeploymentSnapshot> {
    std::fs::create_dir_all(&home.deployments_dir).context("create deployments dir")?;

    let now = time::OffsetDateTime::now_utc();
    let id = now.unix_timestamp_nanos().to_string();
    let created_at = now
        .format(&time::format_description::well_known::Rfc3339)
        .context("format timestamp")?;

    let backup_root = DeploymentSnapshot::backup_root(home, &id);
    std::fs::create_dir_all(&backup_root).context("create backup root")?;

    let lockfile_sha256 = lockfile_path
        .and_then(|p| std::fs::read(p).ok())
        .map(|b| sha256_hex(&b));

    let mut applied = Vec::new();
    for c in &plan.changes {
        let path = PathBuf::from(&c.path);
        let backup_path = match c.op {
            Op::Create => None,
            Op::Update | Op::Delete => {
                if path.exists() {
                    Some(backup_file(&backup_root, &c.target, &path)?)
                } else {
                    None
                }
            }
        };

        match c.op {
            Op::Create | Op::Update => {
                let key = TargetPath {
                    target: c.target.clone(),
                    path: path.clone(),
                };
                let bytes = desired
                    .get(&key)
                    .with_context(|| format!("missing desired bytes for {}", c.path))?;

                if path.exists() {
                    std::fs::remove_file(&path).ok();
                }
                write_atomic(&path, bytes)?;

                let actual = std::fs::read(&path)?;
                let actual_sha = sha256_hex(&actual);
                if let Some(expected) = &c.after_sha256 {
                    if &actual_sha != expected {
                        anyhow::bail!(
                            "write verification failed for {}: expected {}, got {}",
                            path.display(),
                            expected,
                            actual_sha
                        );
                    }
                }
            }
            Op::Delete => {
                if path.exists() {
                    std::fs::remove_file(&path)
                        .with_context(|| format!("remove {}", path.display()))?;
                }
            }
        }

        applied.push(AppliedChange {
            target: c.target.clone(),
            op: match c.op {
                Op::Create => "create",
                Op::Update => "update",
                Op::Delete => "delete",
            }
            .to_string(),
            path: c.path.clone(),
            backup_path: backup_path
                .as_ref()
                .map(|p| p.to_string_lossy().to_string()),
            before_sha256: c.before_sha256.clone(),
            after_sha256: c.after_sha256.clone(),
        });
    }

    let targets = {
        let mut set = std::collections::BTreeSet::new();
        for c in &applied {
            set.insert(c.target.clone());
        }
        set.into_iter().collect()
    };

    let snapshot = DeploymentSnapshot {
        id: id.clone(),
        created_at,
        targets,
        changes: applied,
        lockfile_sha256,
        backup_root: backup_root.to_string_lossy().to_string(),
    };

    let snapshot_path = DeploymentSnapshot::path(home, &id);
    snapshot.save(&snapshot_path)?;

    Ok(snapshot)
}

pub fn rollback(home: &AgentpackHome, snapshot_id: &str) -> anyhow::Result<()> {
    let snapshot_path = DeploymentSnapshot::path(home, snapshot_id);
    let snapshot = DeploymentSnapshot::load(&snapshot_path)
        .with_context(|| format!("load snapshot {}", snapshot_path.display()))?;

    for c in &snapshot.changes {
        let path = PathBuf::from(&c.path);
        match (&c.op[..], &c.backup_path) {
            ("create", None) => {
                if path.exists() {
                    std::fs::remove_file(&path).ok();
                }
            }
            ("update" | "delete", Some(backup)) => {
                let backup_path = PathBuf::from(backup);
                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent).ok();
                }
                std::fs::copy(&backup_path, &path).with_context(|| {
                    format!("restore {} -> {}", backup_path.display(), path.display())
                })?;
            }
            _ => {}
        }
    }

    Ok(())
}

fn backup_file(backup_root: &Path, target: &str, path: &Path) -> anyhow::Result<PathBuf> {
    let target_dir = backup_root.join(sanitize_module_id(target));
    std::fs::create_dir_all(&target_dir).context("create target backup dir")?;
    let key = sha256_hex(path.to_string_lossy().as_bytes());
    let backup_path = target_dir.join(key.chars().take(16).collect::<String>());
    std::fs::copy(path, &backup_path)
        .with_context(|| format!("backup {} -> {}", path.display(), backup_path.display()))?;
    Ok(backup_path)
}

fn write_atomic(path: &Path, bytes: &[u8]) -> anyhow::Result<()> {
    let parent = path
        .parent()
        .with_context(|| format!("invalid path: {}", path.display()))?;
    std::fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;

    let mut tmp = NamedTempFile::new_in(parent).context("create temp file")?;
    tmp.write_all(bytes).context("write temp file")?;
    tmp.flush().ok();

    tmp.persist(path)
        .map(|_| ())
        .map_err(|e| anyhow::anyhow!(e.error))
        .with_context(|| format!("persist {}", path.display()))
}
