use std::collections::BTreeMap;
use std::io::Write as _;
use std::path::Path;
use std::process::Command;

use anyhow::Context as _;
use serde::Serialize;
use tempfile::NamedTempFile;

use crate::config::Manifest;
use crate::fs::list_files;
use crate::lockfile::hash_tree;
use crate::paths::{AgentpackHome, RepoPaths};
use crate::user_error::UserError;

use super::layout::{
    OverlayBaseline, OverlayKind, overlay_baseline_path, path_relative_posix, read_overlay_meta,
    resolve_upstream_module_root, write_overlay_baseline,
};

mod base;

pub fn overlay_drift_warnings(
    module_id: &str,
    overlay_kind: &str,
    upstream_root: &Path,
    overlay_dir: &Path,
) -> anyhow::Result<Vec<String>> {
    if !overlay_dir.exists() {
        return Ok(Vec::new());
    }

    let baseline_path = overlay_baseline_path(overlay_dir);
    if !baseline_path.exists() {
        return Ok(Vec::new());
    }

    let raw = std::fs::read_to_string(&baseline_path)
        .with_context(|| format!("read {}", baseline_path.display()))?;
    let baseline: OverlayBaseline = serde_json::from_str(&raw).context("parse overlay baseline")?;
    let baseline_hash = baseline.upstream_sha256.clone();

    let (current_manifest, current_hash) = hash_tree(upstream_root)
        .with_context(|| format!("hash upstream {}", upstream_root.display()))?;

    let baseline_map: BTreeMap<String, String> = baseline
        .file_manifest
        .iter()
        .map(|f| (f.path.clone(), f.sha256.clone()))
        .collect();
    let current_map: BTreeMap<String, String> = current_manifest
        .into_iter()
        .map(|f| (f.path, f.sha256))
        .collect();

    let mut overlay_files = list_files(overlay_dir)?;
    overlay_files.sort();

    let mut warnings = Vec::new();
    for file in overlay_files {
        let rel = file
            .strip_prefix(overlay_dir)
            .unwrap_or(&file)
            .to_string_lossy()
            .replace('\\', "/");

        let Some(baseline_sha) = baseline_map.get(&rel) else {
            continue;
        };

        match current_map.get(&rel) {
            Some(current_sha) if current_sha != baseline_sha => warnings.push(format!(
                "overlay drift ({overlay_kind}) module {module_id}: upstream changed for {rel} ({baseline_sha} -> {current_sha})"
            )),
            None => warnings.push(format!(
                "overlay drift ({overlay_kind}) module {module_id}: upstream removed {rel} (baseline {baseline_sha})"
            )),
            _ => {}
        }
    }

    if warnings.is_empty() && baseline_hash != current_hash {
        warnings.push(format!(
            "overlay drift ({overlay_kind}) module {module_id}: upstream changed ({baseline_hash} -> {current_hash})"
        ));
    }

    Ok(warnings)
}

#[derive(Debug, Clone, Copy)]
pub struct OverlayRebaseOptions {
    pub dry_run: bool,
    pub sparsify: bool,
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct OverlayRebaseSummary {
    pub processed_files: u64,
    pub updated_files: u64,
    pub deleted_files: u64,
    pub skipped_files: u64,
    pub conflict_files: u64,
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct OverlayRebaseReport {
    pub updated: Vec<String>,
    pub deleted: Vec<String>,
    pub skipped: Vec<String>,
    pub conflicts: Vec<String>,
    pub summary: OverlayRebaseSummary,
}

pub fn rebase_overlay(
    home: &AgentpackHome,
    repo: &RepoPaths,
    manifest: &Manifest,
    module_id: &str,
    overlay_dir: &Path,
    options: OverlayRebaseOptions,
) -> anyhow::Result<OverlayRebaseReport> {
    if !overlay_dir.exists() {
        return Err(anyhow::Error::new(
            UserError::new(
                "E_OVERLAY_NOT_FOUND",
                format!("overlay does not exist: {}", overlay_dir.display()),
            )
            .with_details(serde_json::json!({
                "overlay_dir": overlay_dir.to_string_lossy(),
                "hint": format!("run `agentpack overlay edit {}` to create it", module_id),
            })),
        ));
    }

    let baseline_path = overlay_baseline_path(overlay_dir);
    if !baseline_path.exists() {
        return Err(anyhow::Error::new(
            UserError::new(
                "E_OVERLAY_BASELINE_MISSING",
                format!("overlay baseline is missing: {}", baseline_path.display()),
            )
            .with_details(serde_json::json!({
                "overlay_dir": overlay_dir.to_string_lossy(),
                "baseline_path": baseline_path.to_string_lossy(),
                "hint": format!("run `agentpack overlay edit {}` to recreate metadata", module_id),
            })),
        ));
    }

    let raw = std::fs::read_to_string(&baseline_path)
        .with_context(|| format!("read {}", baseline_path.display()))?;
    let baseline: OverlayBaseline = serde_json::from_str(&raw).context("parse overlay baseline")?;

    let baseline_map: BTreeMap<String, String> = baseline
        .file_manifest
        .iter()
        .map(|f| (f.path.clone(), f.sha256.clone()))
        .collect();

    let meta = read_overlay_meta(overlay_dir)?;
    let override_files = list_files(overlay_dir)
        .with_context(|| format!("list overlay files {}", overlay_dir.display()))?;
    let patch_files = super::patch::list_patch_files(overlay_dir)
        .with_context(|| format!("list patch files {}", overlay_dir.display()))?;

    let has_overrides = !override_files.is_empty();
    let has_patches = !patch_files.is_empty();
    if has_overrides && has_patches {
        return Err(anyhow::Error::new(
            UserError::new(
                "E_CONFIG_INVALID",
                format!(
                    "overlay kind conflict for module {module_id}: cannot mix directory override files and patch artifacts"
                ),
            )
            .with_details(serde_json::json!( {
                "module_id": module_id,
                "overlay_dir": overlay_dir.to_string_lossy(),
                "override_files": override_files.iter().map(|p| path_relative_posix(overlay_dir, p)).collect::<Vec<_>>(),
                "patch_files": patch_files.iter().map(|p| path_relative_posix(overlay_dir, p)).collect::<Vec<_>>(),
                "hint": "use a single overlay kind per overlay directory (dir OR patch)",
            })),
        ));
    }

    let module = manifest
        .modules
        .iter()
        .find(|m| m.id == module_id)
        .with_context(|| format!("module not found: {module_id}"))?;

    let upstream_root = resolve_upstream_module_root(home, repo, module)?;

    let repo_root = repo
        .manifest_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| repo.repo_dir.clone());

    let mut report = match meta.overlay_kind {
        OverlayKind::Dir => {
            if has_patches {
                return Err(anyhow::Error::new(
                    UserError::new(
                        "E_CONFIG_INVALID",
                        format!("overlay_kind=dir but patch artifacts exist for module {module_id}"),
                    )
                    .with_details(serde_json::json!({
                        "module_id": module_id,
                        "overlay_dir": overlay_dir.to_string_lossy(),
                        "hint": "set overlay_kind=patch (in .agentpack/overlay.json) or remove .agentpack/patches",
                    })),
                ));
            }

            match base::resolve_rebase_base(home, module, &baseline, &repo_root)? {
                base::RebaseBase::Dir(base_root) => super::dir::rebase_overlay_dir_files(
                    overlay_dir,
                    &baseline_map,
                    |rel| base::read_optional_file_bytes(&base_root, rel),
                    |rel| base::read_optional_file_bytes(&upstream_root, rel),
                    options,
                )?,
                base::RebaseBase::RepoGit {
                    repo_git_rev,
                    repo_rel_path,
                } => {
                    let repo_git_rev = repo_git_rev.clone();
                    let repo_rel_path = repo_rel_path.clone();
                    super::dir::rebase_overlay_dir_files(
                        overlay_dir,
                        &baseline_map,
                        |rel| {
                            base::git_show_optional_bytes(
                                &repo_root,
                                &repo_git_rev,
                                &repo_rel_path,
                                rel,
                            )
                        },
                        |rel| base::read_optional_file_bytes(&upstream_root, rel),
                        options,
                    )?
                }
            }
        }
        OverlayKind::Patch => {
            if has_overrides {
                return Err(anyhow::Error::new(
                    UserError::new(
                        "E_CONFIG_INVALID",
                        format!(
                            "overlay_kind=patch but directory override files exist for module {module_id}"
                        ),
                    )
                    .with_details(serde_json::json!({
                        "module_id": module_id,
                        "overlay_dir": overlay_dir.to_string_lossy(),
                        "hint": "move edits into .agentpack/patches/*.patch or set overlay_kind=dir",
                    })),
                ));
            }

            match base::resolve_rebase_base(home, module, &baseline, &repo_root)? {
                base::RebaseBase::Dir(base_root) => super::patch::rebase_overlay_patch_files(
                    module_id,
                    overlay_dir,
                    &patch_files,
                    &baseline_map,
                    |rel| base::read_optional_file_bytes(&base_root, rel),
                    |rel| base::read_optional_file_bytes(&upstream_root, rel),
                    options,
                )?,
                base::RebaseBase::RepoGit {
                    repo_git_rev,
                    repo_rel_path,
                } => {
                    let repo_git_rev = repo_git_rev.clone();
                    let repo_rel_path = repo_rel_path.clone();
                    super::patch::rebase_overlay_patch_files(
                        module_id,
                        overlay_dir,
                        &patch_files,
                        &baseline_map,
                        |rel| {
                            base::git_show_optional_bytes(
                                &repo_root,
                                &repo_git_rev,
                                &repo_rel_path,
                                rel,
                            )
                        },
                        |rel| base::read_optional_file_bytes(&upstream_root, rel),
                        options,
                    )?
                }
            }
        }
    };

    // Only rewrite the baseline when we could fully reason about base for all baseline-known files.
    if !options.dry_run {
        write_overlay_baseline(home, repo, module, &upstream_root, overlay_dir)?;
    }

    report.updated.sort();
    report.deleted.sort();
    report.skipped.sort();
    report.conflicts.sort();

    Ok(report)
}

#[derive(Debug)]
pub(super) struct MergeOutcome {
    pub(super) merged: Vec<u8>,
    pub(super) conflicted: bool,
}

pub(super) fn merge_three_way_git(
    base: &[u8],
    ours: &[u8],
    theirs: &[u8],
) -> anyhow::Result<MergeOutcome> {
    let mut ours_file = NamedTempFile::new().context("create temp ours")?;
    ours_file.write_all(ours).context("write ours")?;

    let mut base_file = NamedTempFile::new().context("create temp base")?;
    base_file.write_all(base).context("write base")?;

    let mut theirs_file = NamedTempFile::new().context("create temp theirs")?;
    theirs_file.write_all(theirs).context("write theirs")?;

    let out = Command::new("git")
        .arg("merge-file")
        .arg("-p")
        .arg(ours_file.path())
        .arg(base_file.path())
        .arg(theirs_file.path())
        .output()
        .context("git merge-file")?;

    match out.status.code() {
        Some(0) => Ok(MergeOutcome {
            merged: out.stdout,
            conflicted: false,
        }),
        Some(1) => Ok(MergeOutcome {
            merged: out.stdout,
            conflicted: true,
        }),
        _ => anyhow::bail!(
            "git merge-file failed: {}",
            String::from_utf8_lossy(&out.stderr)
        ),
    }
}
