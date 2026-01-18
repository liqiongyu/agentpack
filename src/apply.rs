use std::path::{Path, PathBuf};

use anyhow::Context as _;

use crate::deploy::{DesiredState, Op, PlanResult, TargetPath};
use crate::fs::write_atomic;
use crate::hash::sha256_hex;
use crate::paths::AgentpackHome;
use crate::state::{AppliedChange, DeploymentSnapshot, ManagedFile, list_snapshots};
use crate::store::sanitize_module_id;
use crate::target_manifest::{ManagedManifestFile, TargetManifest, manifest_path_for_target};
use crate::targets::{TargetRoot, best_root_for};

pub fn apply_plan(
    home: &AgentpackHome,
    kind: &str,
    plan: &PlanResult,
    desired: &DesiredState,
    lockfile_path: Option<&Path>,
    roots: &[TargetRoot],
) -> anyhow::Result<DeploymentSnapshot> {
    std::fs::create_dir_all(&home.snapshots_dir).context("create snapshots dir")?;

    let now = time::OffsetDateTime::now_utc();
    let id = now.unix_timestamp_nanos().to_string();
    let created_at = now
        .format(&time::format_description::well_known::Rfc3339)
        .context("format timestamp")?;

    let backup_root = DeploymentSnapshot::backup_root(home, &id);
    std::fs::create_dir_all(&backup_root).context("create backup root")?;
    let state_root = DeploymentSnapshot::state_root(home, &id);
    std::fs::create_dir_all(&state_root).context("create snapshot state root")?;

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
                let desired_file = desired
                    .get(&key)
                    .with_context(|| format!("missing desired bytes for {}", c.path))?;

                write_atomic(&path, &desired_file.bytes)?;

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

    if kind == "deploy" || kind == "bootstrap" {
        applied.extend(write_target_manifests(
            &backup_root,
            &state_root,
            &created_at,
            &id,
            plan,
            desired,
            roots,
        )?);
    }

    store_snapshot_state_files(&state_root, desired)?;

    let mut managed_files: Vec<ManagedFile> = desired
        .iter()
        .map(|(tp, desired_file)| ManagedFile {
            target: tp.target.clone(),
            path: tp.path.to_string_lossy().to_string(),
            sha256: sha256_hex(&desired_file.bytes),
        })
        .collect();
    managed_files.sort_by(|a, b| {
        (a.target.as_str(), a.path.as_str()).cmp(&(b.target.as_str(), b.path.as_str()))
    });

    let targets = {
        let mut set = std::collections::BTreeSet::new();
        for f in &managed_files {
            set.insert(f.target.clone());
        }
        set.into_iter().collect()
    };

    let snapshot = DeploymentSnapshot {
        kind: kind.to_string(),
        id: id.clone(),
        created_at,
        targets,
        managed_files,
        changes: applied,
        rolled_back_to: None,
        lockfile_sha256,
        backup_root: backup_root.to_string_lossy().to_string(),
    };

    let snapshot_path = DeploymentSnapshot::path(home, &id);
    snapshot.save(&snapshot_path)?;

    Ok(snapshot)
}

fn write_target_manifests(
    backup_root: &Path,
    state_root: &Path,
    created_at: &str,
    snapshot_id: &str,
    plan: &PlanResult,
    desired: &DesiredState,
    roots: &[TargetRoot],
) -> anyhow::Result<Vec<AppliedChange>> {
    if roots.is_empty() {
        return Ok(Vec::new());
    }

    let mut per_root: Vec<Vec<ManagedManifestFile>> = vec![Vec::new(); roots.len()];
    for (tp, desired_file) in desired {
        let Some((idx, root)) = best_root_index(roots, &tp.target, &tp.path) else {
            continue;
        };
        let rel = tp
            .path
            .strip_prefix(&root.root)
            .with_context(|| format!("compute relpath for {}", tp.path.display()))?;
        let rel = rel.to_string_lossy().replace('\\', "/");
        per_root[idx].push(ManagedManifestFile {
            path: rel,
            sha256: sha256_hex(&desired_file.bytes),
            module_ids: desired_file.module_ids.clone(),
        });
    }

    let mut root_had_changes: Vec<bool> = vec![false; roots.len()];
    for c in &plan.changes {
        let path = PathBuf::from(&c.path);
        if let Some((idx, _)) = best_root_index(roots, &c.target, &path) {
            root_had_changes[idx] = true;
        }
    }

    let mut out = Vec::new();
    for (idx, root) in roots.iter().enumerate() {
        let manifest_path = manifest_path_for_target(&root.root, &root.target);
        let existed = manifest_path.exists();
        let should_write = existed || !per_root[idx].is_empty() || root_had_changes[idx];
        if !should_write {
            continue;
        }

        if !root.root.exists() && !per_root[idx].is_empty() {
            std::fs::create_dir_all(&root.root)
                .with_context(|| format!("create {}", root.root.display()))?;
        }
        if !root.root.exists() {
            continue;
        }

        let mut manifest = TargetManifest::new(
            root.target.clone(),
            created_at.to_string(),
            Some(snapshot_id.to_string()),
        );
        per_root[idx].sort_by(|a, b| a.path.cmp(&b.path));
        manifest.managed_files = per_root[idx].clone();

        let mut content = serde_json::to_string_pretty(&manifest)?;
        if !content.ends_with('\n') {
            content.push('\n');
        }

        let before_sha256 = if existed {
            Some(sha256_hex(&std::fs::read(&manifest_path)?))
        } else {
            None
        };
        let after_sha256 = Some(sha256_hex(content.as_bytes()));
        let backup_path = if existed {
            Some(backup_file(backup_root, &root.target, &manifest_path)?)
        } else {
            None
        };

        write_atomic(&manifest_path, content.as_bytes())?;

        let state_path = snapshot_state_path(state_root, &root.target, &manifest_path)?;
        write_atomic(&state_path, content.as_bytes())?;

        out.push(AppliedChange {
            target: root.target.clone(),
            op: if existed { "update" } else { "create" }.to_string(),
            path: manifest_path.to_string_lossy().to_string(),
            backup_path: backup_path.map(|p| p.to_string_lossy().to_string()),
            before_sha256,
            after_sha256,
        });
    }

    Ok(out)
}

fn best_root_index<'a>(
    roots: &'a [TargetRoot],
    target: &str,
    path: &Path,
) -> Option<(usize, &'a TargetRoot)> {
    let best = best_root_for(roots, target, path)?;
    roots.iter().enumerate().find(|(_, r)| *r == best)
}

pub fn rollback(home: &AgentpackHome, snapshot_id: &str) -> anyhow::Result<DeploymentSnapshot> {
    let target_path = DeploymentSnapshot::path(home, snapshot_id);
    let target_snapshot = DeploymentSnapshot::load(&target_path)
        .with_context(|| format!("load snapshot {}", target_path.display()))?;
    if target_snapshot.kind == "rollback" {
        if let Some(to) = &target_snapshot.rolled_back_to {
            anyhow::bail!(
                "snapshot {} is a rollback event; use --to {} instead",
                snapshot_id,
                to
            );
        }
        anyhow::bail!(
            "snapshot {} is a rollback event and cannot be used as a rollback target",
            snapshot_id
        );
    }

    let snapshots = list_snapshots(home)?;
    if snapshots.is_empty() {
        anyhow::bail!("no deployment snapshots found");
    }

    let mut parents: std::collections::HashMap<String, Option<String>> =
        std::collections::HashMap::new();
    let mut head: Option<String> = None;
    for s in &snapshots {
        match s.kind.as_str() {
            "deploy" | "bootstrap" => {
                parents.insert(s.id.clone(), head.clone());
                head = Some(s.id.clone());
            }
            "rollback" => {
                if let Some(to) = &s.rolled_back_to {
                    head = Some(to.clone());
                }
            }
            _ => {}
        }
    }

    let Some(current_head) = head else {
        anyhow::bail!("no deployment snapshots found");
    };

    let mut applied = Vec::new();
    let target_state_root = DeploymentSnapshot::state_root(home, snapshot_id);
    if target_state_root.exists() {
        let current_snapshot_path = DeploymentSnapshot::path(home, &current_head);
        let current_snapshot =
            DeploymentSnapshot::load(&current_snapshot_path).with_context(|| {
                format!("load current snapshot {}", current_snapshot_path.display())
            })?;

        let mut desired_set = std::collections::HashSet::new();
        for f in &target_snapshot.managed_files {
            desired_set.insert((f.target.clone(), f.path.clone()));
        }

        for f in &target_snapshot.managed_files {
            let abs = PathBuf::from(&f.path);
            let state_path = snapshot_state_path(&target_state_root, &f.target, &abs)?;
            let bytes = std::fs::read(&state_path).with_context(|| {
                format!(
                    "read snapshot state {} for {}",
                    state_path.display(),
                    abs.display()
                )
            })?;
            let actual_sha = sha256_hex(&bytes);
            if actual_sha != f.sha256 {
                anyhow::bail!(
                    "snapshot state hash mismatch for {}: expected {}, got {}",
                    abs.display(),
                    f.sha256,
                    actual_sha
                );
            }

            let before_sha256 = std::fs::read(&abs).ok().map(|b| sha256_hex(&b));
            write_atomic(&abs, &bytes)?;
            applied.push(AppliedChange {
                target: f.target.clone(),
                op: "rollback_restore".to_string(),
                path: f.path.clone(),
                backup_path: Some(state_path.to_string_lossy().to_string()),
                before_sha256,
                after_sha256: Some(actual_sha),
            });
        }

        for c in &target_snapshot.changes {
            let abs = PathBuf::from(&c.path);
            if !crate::target_manifest::is_target_manifest_path(&abs) {
                continue;
            }
            if c.op != "create" && c.op != "update" {
                continue;
            }
            let state_path = snapshot_state_path(&target_state_root, &c.target, &abs)?;
            let bytes = std::fs::read(&state_path).with_context(|| {
                format!(
                    "read snapshot state {} for {}",
                    state_path.display(),
                    abs.display()
                )
            })?;
            let before_sha256 = std::fs::read(&abs).ok().map(|b| sha256_hex(&b));
            let after_sha256 = Some(sha256_hex(&bytes));
            write_atomic(&abs, &bytes)?;
            applied.push(AppliedChange {
                target: c.target.clone(),
                op: "rollback_restore".to_string(),
                path: c.path.clone(),
                backup_path: Some(state_path.to_string_lossy().to_string()),
                before_sha256,
                after_sha256,
            });
        }

        for f in &current_snapshot.managed_files {
            if desired_set.contains(&(f.target.clone(), f.path.clone())) {
                continue;
            }
            let abs = PathBuf::from(&f.path);
            let before_sha256 = std::fs::read(&abs).ok().map(|b| sha256_hex(&b));
            if abs.exists() {
                std::fs::remove_file(&abs).ok();
            }
            applied.push(AppliedChange {
                target: f.target.clone(),
                op: "rollback_delete".to_string(),
                path: f.path.clone(),
                backup_path: None,
                before_sha256,
                after_sha256: None,
            });
        }
    } else if current_head != snapshot_id {
        let mut cursor = current_head.clone();
        loop {
            if cursor == snapshot_id {
                break;
            }

            let snapshot_path = DeploymentSnapshot::path(home, &cursor);
            let snapshot = DeploymentSnapshot::load(&snapshot_path)
                .with_context(|| format!("load snapshot {}", snapshot_path.display()))?;

            for c in &snapshot.changes {
                let path = PathBuf::from(&c.path);
                match (&c.op[..], &c.backup_path) {
                    ("create", None) => {
                        if path.exists() {
                            std::fs::remove_file(&path).ok();
                        }
                        applied.push(AppliedChange {
                            target: c.target.clone(),
                            op: "rollback_delete".to_string(),
                            path: c.path.clone(),
                            backup_path: None,
                            before_sha256: None,
                            after_sha256: None,
                        });
                    }
                    ("update" | "delete", Some(backup)) => {
                        let backup_path = PathBuf::from(backup);
                        if let Some(parent) = path.parent() {
                            std::fs::create_dir_all(parent).ok();
                        }
                        std::fs::copy(&backup_path, &path).with_context(|| {
                            format!("restore {} -> {}", backup_path.display(), path.display())
                        })?;
                        applied.push(AppliedChange {
                            target: c.target.clone(),
                            op: "rollback_restore".to_string(),
                            path: c.path.clone(),
                            backup_path: c.backup_path.clone(),
                            before_sha256: None,
                            after_sha256: None,
                        });
                    }
                    _ => {}
                }
            }

            cursor = parents.get(&cursor).cloned().flatten().ok_or_else(|| {
                anyhow::anyhow!(
                    "snapshot {} is not reachable from current deployment state {}",
                    snapshot_id,
                    current_head
                )
            })?;
        }
    }

    std::fs::create_dir_all(&home.snapshots_dir).context("create snapshots dir")?;

    let now = time::OffsetDateTime::now_utc();
    let id = now.unix_timestamp_nanos().to_string();
    let created_at = now
        .format(&time::format_description::well_known::Rfc3339)
        .context("format timestamp")?;

    let event = DeploymentSnapshot {
        kind: "rollback".to_string(),
        id: id.clone(),
        created_at,
        targets: target_snapshot.targets.clone(),
        managed_files: target_snapshot.managed_files.clone(),
        changes: applied,
        rolled_back_to: Some(snapshot_id.to_string()),
        lockfile_sha256: target_snapshot.lockfile_sha256.clone(),
        backup_root: String::new(),
    };

    let event_path = DeploymentSnapshot::path(home, &id);
    event.save(&event_path)?;

    Ok(event)
}

fn snapshot_state_path(state_root: &Path, target: &str, path: &Path) -> anyhow::Result<PathBuf> {
    let target_dir = state_root.join(sanitize_module_id(target));
    let mut normalized = path.to_string_lossy().to_string();
    normalized = normalized.replace('\\', "/");
    let key = sha256_hex(normalized.as_bytes());
    Ok(target_dir.join(key.chars().take(16).collect::<String>()))
}

fn store_snapshot_state_files(state_root: &Path, desired: &DesiredState) -> anyhow::Result<()> {
    for (tp, desired_file) in desired {
        let state_path = snapshot_state_path(state_root, &tp.target, &tp.path)?;
        write_atomic(&state_path, &desired_file.bytes)?;
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
