use anyhow::Context as _;

use crate::deploy::{DesiredState, TargetPath};
use crate::hash::sha256_hex;
use crate::targets::TargetRoot;

#[derive(Clone, Copy, Debug)]
pub(crate) enum ExtraScanHashMode {
    IncludeHashes,
    SkipHashes,
}

#[derive(serde::Serialize)]
pub(crate) struct DriftItem {
    pub(crate) target: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) root: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) root_posix: Option<String>,
    pub(crate) path: String,
    pub(crate) path_posix: String,
    pub(crate) expected: Option<String>,
    pub(crate) actual: Option<String>,
    pub(crate) kind: String,
}

#[derive(Default, serde::Serialize, Clone, Copy)]
pub(crate) struct DriftSummary {
    pub(crate) modified: u64,
    pub(crate) missing: u64,
    pub(crate) extra: u64,
}

pub(crate) struct StatusDriftReport {
    pub(crate) warnings: Vec<String>,
    pub(crate) drift: Vec<DriftItem>,
    pub(crate) summary: DriftSummary,
    pub(crate) any_manifest: bool,
    pub(crate) needs_deploy_apply: bool,
}

fn best_root_idx(roots: &[TargetRoot], target: &str, path: &std::path::Path) -> Option<usize> {
    roots
        .iter()
        .enumerate()
        .filter(|(_, r)| r.target == target)
        .filter(|(_, r)| path.strip_prefix(&r.root).is_ok())
        .max_by_key(|(_, r)| r.root.components().count())
        .map(|(idx, _)| idx)
}

pub(crate) fn status_drift_report(
    desired: &DesiredState,
    roots: &[TargetRoot],
    mut warnings: Vec<String>,
    extra_scan_hash_mode: ExtraScanHashMode,
) -> anyhow::Result<StatusDriftReport> {
    let mut drift = Vec::new();
    let mut summary = DriftSummary::default();

    let mut manifests: Vec<Option<crate::target_manifest::TargetManifest>> =
        vec![None; roots.len()];
    let mut manifest_paths: Vec<Option<std::path::PathBuf>> = vec![None; roots.len()];
    for (idx, root) in roots.iter().enumerate() {
        let preferred = crate::target_manifest::manifest_path_for_target(&root.root, &root.target);
        let legacy = crate::target_manifest::legacy_manifest_path(&root.root);

        let (path, used_legacy) = if preferred.exists() {
            (preferred, false)
        } else if legacy.exists() {
            (legacy, true)
        } else {
            continue;
        };

        if used_legacy {
            warnings.push(format!(
                "target manifest ({}): using legacy manifest filename {} (consider running `agentpack deploy --apply` to migrate)",
                root.target,
                path.display(),
            ));
        }

        let (manifest, manifest_warnings) =
            crate::target_manifest::read_target_manifest_soft(&path, &root.target);
        warnings.extend(manifest_warnings);
        manifests[idx] = manifest;
        if manifests[idx].is_some() {
            manifest_paths[idx] = Some(path);
        }
    }

    let mut needs_deploy_apply = false;

    let any_manifest = manifests.iter().any(Option::is_some);
    if !any_manifest {
        warnings.push(
            "no target manifests found; drift may be inaccurate (run deploy --apply to write manifests)"
                .to_string(),
        );
        needs_deploy_apply = true;

        for (tp, desired_file) in desired {
            let expected = format!("sha256:{}", sha256_hex(&desired_file.bytes));
            let root = best_root_idx(roots, &tp.target, &tp.path)
                .and_then(|idx| roots.get(idx))
                .map(|r| r.root.as_path());
            match std::fs::read(&tp.path) {
                Ok(actual_bytes) => {
                    let actual = format!("sha256:{}", sha256_hex(&actual_bytes));
                    if actual != expected {
                        summary.modified += 1;
                        drift.push(DriftItem {
                            target: tp.target.clone(),
                            root: root.map(|p| p.to_string_lossy().to_string()),
                            root_posix: root.map(crate::paths::path_to_posix_string),
                            path: tp.path.to_string_lossy().to_string(),
                            path_posix: crate::paths::path_to_posix_string(&tp.path),
                            expected: Some(expected),
                            actual: Some(actual),
                            kind: "modified".to_string(),
                        });
                    }
                }
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                    summary.missing += 1;
                    drift.push(DriftItem {
                        target: tp.target.clone(),
                        root: root.map(|p| p.to_string_lossy().to_string()),
                        root_posix: root.map(crate::paths::path_to_posix_string),
                        path: tp.path.to_string_lossy().to_string(),
                        path_posix: crate::paths::path_to_posix_string(&tp.path),
                        expected: Some(expected),
                        actual: None,
                        kind: "missing".to_string(),
                    });
                }
                Err(err) => return Err(err).context("read deployed file"),
            }
        }

        return Ok(StatusDriftReport {
            warnings,
            drift,
            summary,
            any_manifest,
            needs_deploy_apply,
        });
    }

    let mut desired_by_root: Vec<Vec<(&TargetPath, &crate::deploy::DesiredFile)>> =
        vec![Vec::new(); roots.len()];
    for (tp, desired_file) in desired {
        let Some(root_idx) = best_root_idx(roots, &tp.target, &tp.path) else {
            continue;
        };
        desired_by_root[root_idx].push((tp, desired_file));
    }

    for (idx, root) in roots.iter().enumerate() {
        let Some(manifest) = &manifests[idx] else {
            if desired_by_root[idx].is_empty() {
                continue;
            }
            needs_deploy_apply = true;
            warnings.push(format!(
                "no usable target manifest for {} {}; drift may be incomplete (run deploy --apply to write manifests)",
                root.target,
                root.root.display()
            ));
            for (tp, desired_file) in &desired_by_root[idx] {
                let expected = format!("sha256:{}", sha256_hex(&desired_file.bytes));
                match std::fs::read(&tp.path) {
                    Ok(actual_bytes) => {
                        let actual = format!("sha256:{}", sha256_hex(&actual_bytes));
                        if actual != expected {
                            summary.modified += 1;
                            drift.push(DriftItem {
                                target: tp.target.clone(),
                                root: Some(root.root.to_string_lossy().to_string()),
                                root_posix: Some(crate::paths::path_to_posix_string(&root.root)),
                                path: tp.path.to_string_lossy().to_string(),
                                path_posix: crate::paths::path_to_posix_string(&tp.path),
                                expected: Some(expected),
                                actual: Some(actual),
                                kind: "modified".to_string(),
                            });
                        }
                    }
                    Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                        summary.missing += 1;
                        drift.push(DriftItem {
                            target: tp.target.clone(),
                            root: Some(root.root.to_string_lossy().to_string()),
                            root_posix: Some(crate::paths::path_to_posix_string(&root.root)),
                            path: tp.path.to_string_lossy().to_string(),
                            path_posix: crate::paths::path_to_posix_string(&tp.path),
                            expected: Some(expected),
                            actual: None,
                            kind: "missing".to_string(),
                        });
                    }
                    Err(err) => return Err(err).context("read deployed file"),
                }
            }
            continue;
        };

        let mut managed_paths = crate::deploy::ManagedPaths::new();
        for f in &manifest.managed_files {
            let rel_path = std::path::Path::new(&f.path);
            if rel_path.is_absolute()
                || rel_path
                    .components()
                    .any(|c| matches!(c, std::path::Component::ParentDir))
            {
                warnings.push(format!(
                    "target manifest ({}): skipped invalid entry path {:?} in {}",
                    root.target,
                    f.path,
                    manifest_paths[idx]
                        .as_ref()
                        .map(|p| p.display().to_string())
                        .unwrap_or_else(|| "<unknown>".to_string()),
                ));
                continue;
            }

            managed_paths.insert(TargetPath {
                target: root.target.clone(),
                path: root.root.join(&f.path),
            });
        }

        for tp in &managed_paths {
            let expected = desired
                .get(tp)
                .map(|f| format!("sha256:{}", sha256_hex(&f.bytes)));
            match std::fs::read(&tp.path) {
                Ok(actual_bytes) => {
                    let actual = format!("sha256:{}", sha256_hex(&actual_bytes));
                    if let Some(exp) = &expected {
                        if &actual != exp {
                            summary.modified += 1;
                            drift.push(DriftItem {
                                target: tp.target.clone(),
                                root: Some(root.root.to_string_lossy().to_string()),
                                root_posix: Some(crate::paths::path_to_posix_string(&root.root)),
                                path: tp.path.to_string_lossy().to_string(),
                                path_posix: crate::paths::path_to_posix_string(&tp.path),
                                expected: Some(exp.clone()),
                                actual: Some(actual),
                                kind: "modified".to_string(),
                            });
                        }
                    } else {
                        summary.extra += 1;
                        drift.push(DriftItem {
                            target: tp.target.clone(),
                            root: Some(root.root.to_string_lossy().to_string()),
                            root_posix: Some(crate::paths::path_to_posix_string(&root.root)),
                            path: tp.path.to_string_lossy().to_string(),
                            path_posix: crate::paths::path_to_posix_string(&tp.path),
                            expected: None,
                            actual: Some(actual),
                            kind: "extra".to_string(),
                        });
                    }
                }
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                    if let Some(exp) = expected {
                        summary.missing += 1;
                        drift.push(DriftItem {
                            target: tp.target.clone(),
                            root: Some(root.root.to_string_lossy().to_string()),
                            root_posix: Some(crate::paths::path_to_posix_string(&root.root)),
                            path: tp.path.to_string_lossy().to_string(),
                            path_posix: crate::paths::path_to_posix_string(&tp.path),
                            expected: Some(exp),
                            actual: None,
                            kind: "missing".to_string(),
                        });
                    }
                }
                Err(err) => return Err(err).context("read deployed file"),
            }
        }

        if !root.scan_extras {
            continue;
        }
        if !root.root.exists() {
            continue;
        }

        let mut files = crate::fs::list_files(&root.root)?;
        files.sort();
        for path in files {
            if crate::target_manifest::is_target_manifest_path(&path) {
                continue;
            }

            let tp = TargetPath {
                target: root.target.clone(),
                path: path.clone(),
            };
            if managed_paths.contains(&tp) {
                continue;
            }

            summary.extra += 1;

            let actual = match extra_scan_hash_mode {
                ExtraScanHashMode::IncludeHashes => {
                    Some(format!("sha256:{}", sha256_hex(&std::fs::read(&tp.path)?)))
                }
                ExtraScanHashMode::SkipHashes => None,
            };

            drift.push(DriftItem {
                target: tp.target.clone(),
                root: Some(root.root.to_string_lossy().to_string()),
                root_posix: Some(crate::paths::path_to_posix_string(&root.root)),
                path: tp.path.to_string_lossy().to_string(),
                path_posix: crate::paths::path_to_posix_string(&tp.path),
                expected: None,
                actual,
                kind: "extra".to_string(),
            });
        }
    }

    Ok(StatusDriftReport {
        warnings,
        drift,
        summary,
        any_manifest,
        needs_deploy_apply,
    })
}
