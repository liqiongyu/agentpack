use std::collections::BTreeMap;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::Context as _;
use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;
use walkdir::WalkDir;

use crate::config::GitSource;
use crate::config::{Manifest, Module, SourceKind};
use crate::fs::{copy_tree, copy_tree_missing_only, list_files, write_atomic};
use crate::hash::sha256_hex;
use crate::lockfile::{FileEntry, Lockfile, hash_tree};
use crate::paths::{AgentpackHome, RepoPaths};
use crate::store::Store;
use crate::user_error::UserError;

#[derive(Debug, Clone)]
pub struct OverlaySkeleton {
    pub dir: PathBuf,
    pub created: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct OverlayLayer<'a> {
    pub scope: &'a str,
    pub dir: &'a Path,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum OverlayKind {
    Dir,
    Patch,
}

fn default_overlay_kind() -> OverlayKind {
    OverlayKind::Dir
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OverlayMeta {
    #[serde(default = "default_overlay_kind")]
    overlay_kind: OverlayKind,
}

#[derive(Debug, Serialize, Deserialize)]
struct OverlayBaseline {
    version: u32,
    created_at: String,
    upstream_sha256: String,
    file_manifest: Vec<FileEntry>,
    #[serde(default)]
    upstream: Option<BaselineUpstream>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum BaselineUpstream {
    Git {
        url: String,
        commit: String,
        #[serde(default)]
        subdir: String,
    },
    LocalPath {
        /// The upstream module root path, relative to the config repo root (POSIX style).
        repo_rel_path: String,
        #[serde(default)]
        repo_git_rev: Option<String>,
        #[serde(default)]
        repo_dirty: Option<bool>,
    },
}

pub fn ensure_overlay_skeleton(
    home: &AgentpackHome,
    repo: &RepoPaths,
    manifest: &Manifest,
    module_id: &str,
    overlay_dir: &Path,
) -> anyhow::Result<OverlaySkeleton> {
    ensure_overlay_skeleton_impl(home, repo, manifest, module_id, overlay_dir, true)
}

pub fn ensure_overlay_skeleton_sparse(
    home: &AgentpackHome,
    repo: &RepoPaths,
    manifest: &Manifest,
    module_id: &str,
    overlay_dir: &Path,
) -> anyhow::Result<OverlaySkeleton> {
    ensure_overlay_skeleton_impl(home, repo, manifest, module_id, overlay_dir, false)
}

pub fn materialize_overlay_from_upstream(
    home: &AgentpackHome,
    repo: &RepoPaths,
    manifest: &Manifest,
    module_id: &str,
    overlay_dir: &Path,
) -> anyhow::Result<()> {
    let module = manifest
        .modules
        .iter()
        .find(|m| m.id == module_id)
        .with_context(|| format!("module not found: {module_id}"))?;

    let upstream_root = resolve_upstream_module_root(home, repo, module)?;

    std::fs::create_dir_all(overlay_dir).context("create overlay dir")?;
    copy_tree_missing_only(&upstream_root, overlay_dir).with_context(|| {
        format!(
            "materialize upstream {} -> {}",
            upstream_root.display(),
            overlay_dir.display()
        )
    })?;

    if !overlay_baseline_path(overlay_dir).exists() {
        write_overlay_baseline(home, repo, module, &upstream_root, overlay_dir)?;
    }
    if !overlay_module_id_path(overlay_dir).exists() {
        write_overlay_module_id(module_id, overlay_dir)?;
    }
    if !overlay_meta_path(overlay_dir).exists() {
        write_overlay_meta_default_dir(overlay_dir)?;
    }

    Ok(())
}

fn ensure_overlay_skeleton_impl(
    home: &AgentpackHome,
    repo: &RepoPaths,
    manifest: &Manifest,
    module_id: &str,
    overlay_dir: &Path,
    copy_upstream: bool,
) -> anyhow::Result<OverlaySkeleton> {
    let module = manifest
        .modules
        .iter()
        .find(|m| m.id == module_id)
        .with_context(|| format!("module not found: {module_id}"))?;

    let upstream_root = resolve_upstream_module_root(home, repo, module)?;

    let created = !overlay_dir.exists();
    if created {
        std::fs::create_dir_all(overlay_dir).context("create overlay dir")?;
        if copy_upstream {
            copy_tree(&upstream_root, overlay_dir).with_context(|| {
                format!(
                    "copy upstream {} -> {}",
                    upstream_root.display(),
                    overlay_dir.display()
                )
            })?;
        }
    }

    if !overlay_baseline_path(overlay_dir).exists() {
        write_overlay_baseline(home, repo, module, &upstream_root, overlay_dir)?;
    }
    if !overlay_module_id_path(overlay_dir).exists() {
        write_overlay_module_id(module_id, overlay_dir)?;
    }
    if !overlay_meta_path(overlay_dir).exists() {
        write_overlay_meta_default_dir(overlay_dir)?;
    }

    Ok(OverlaySkeleton {
        dir: overlay_dir.to_path_buf(),
        created,
    })
}

pub fn compose_module_tree(
    module_id: &str,
    upstream_root: &Path,
    overlays: &[OverlayLayer<'_>],
    out_dir: &Path,
) -> anyhow::Result<()> {
    std::fs::create_dir_all(out_dir).context("create module out dir")?;
    copy_tree(upstream_root, out_dir).context("copy upstream")?;
    for overlay in overlays {
        if !overlay.dir.exists() {
            continue;
        }

        let meta = read_overlay_meta(overlay.dir)?;
        let override_files = list_files(overlay.dir)
            .with_context(|| format!("list overlay files {}", overlay.dir.display()))?;
        let patch_files = list_patch_files(overlay.dir)
            .with_context(|| format!("list patch files {}", overlay.dir.display()))?;

        let has_overrides = !override_files.is_empty();
        let has_patches = !patch_files.is_empty();

        if has_overrides && has_patches {
            return Err(anyhow::Error::new(
                UserError::new(
                    "E_CONFIG_INVALID",
                    format!(
                        "overlay kind conflict for module {module_id} ({scope}): cannot mix directory override files and patch artifacts",
                        scope = overlay.scope
                    ),
                )
                .with_details(serde_json::json!({
                    "module_id": module_id,
                    "scope": overlay.scope,
                    "overlay_dir": overlay.dir.to_string_lossy(),
                    "override_files": override_files.iter().map(|p| path_relative_posix(overlay.dir, p)).collect::<Vec<_>>(),
                    "patch_files": patch_files.iter().map(|p| path_relative_posix(overlay.dir, p)).collect::<Vec<_>>(),
                    "hint": "use a single overlay kind per overlay directory (dir OR patch)",
                })),
            ));
        }

        match meta.overlay_kind {
            OverlayKind::Dir => {
                if has_patches {
                    return Err(anyhow::Error::new(
                        UserError::new(
                            "E_CONFIG_INVALID",
                            format!(
                                "overlay_kind=dir but patch artifacts exist for module {module_id} ({scope})",
                                scope = overlay.scope
                            ),
                        )
                        .with_details(serde_json::json!({
                            "module_id": module_id,
                            "scope": overlay.scope,
                            "overlay_dir": overlay.dir.to_string_lossy(),
                            "hint": "set overlay_kind=patch (in .agentpack/overlay.json) or remove .agentpack/patches",
                        })),
                    ));
                }

                copy_tree(overlay.dir, out_dir)
                    .with_context(|| format!("apply overlay {}", overlay.dir.display()))?;
            }
            OverlayKind::Patch => {
                if has_overrides {
                    return Err(anyhow::Error::new(
                        UserError::new(
                            "E_CONFIG_INVALID",
                            format!(
                                "overlay_kind=patch but directory override files exist for module {module_id} ({scope})",
                                scope = overlay.scope
                            ),
                        )
                        .with_details(serde_json::json!({
                            "module_id": module_id,
                            "scope": overlay.scope,
                            "overlay_dir": overlay.dir.to_string_lossy(),
                            "hint": "move edits into .agentpack/patches/*.patch or set overlay_kind=dir",
                        })),
                    ));
                }

                apply_patch_overlays(module_id, overlay.scope, overlay.dir, out_dir, &patch_files)?;
            }
        }
    }
    Ok(())
}

fn list_patch_files(overlay_dir: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let patches_dir = overlay_dir.join(".agentpack").join("patches");
    if !patches_dir.exists() {
        return Ok(Vec::new());
    }

    let mut out = Vec::new();
    for entry in WalkDir::new(&patches_dir).follow_links(false) {
        let entry = entry?;
        if entry.file_type().is_dir() {
            continue;
        }
        if entry
            .path()
            .extension()
            .and_then(|s| s.to_str())
            .is_some_and(|s| s.eq_ignore_ascii_case("patch"))
        {
            out.push(entry.into_path());
        }
    }

    out.sort();
    Ok(out)
}

fn apply_patch_overlays(
    module_id: &str,
    scope: &str,
    overlay_dir: &Path,
    out_dir: &Path,
    patch_files: &[PathBuf],
) -> anyhow::Result<()> {
    let patches_root = overlay_dir.join(".agentpack").join("patches");

    for patch_file in patch_files {
        let rel_patch = patch_file.strip_prefix(&patches_root).unwrap_or(patch_file);
        let rel_patch_posix = rel_patch.to_string_lossy().replace('\\', "/");
        let Some(rel_target) = rel_patch_posix.strip_suffix(".patch") else {
            continue;
        };

        if !validate_posix_relpath(rel_target) {
            return Err(anyhow::Error::new(
                UserError::new(
                    "E_CONFIG_INVALID",
                    format!(
                        "invalid patch relpath for module {module_id} ({scope}): {rel_target}",
                        scope = scope
                    ),
                )
                .with_details(serde_json::json!({
                    "module_id": module_id,
                    "scope": scope,
                    "overlay_dir": overlay_dir.to_string_lossy(),
                    "patch_file": patch_file.to_string_lossy(),
                    "relpath": rel_target,
                    "hint": "patch filenames must map to a safe relpath under the upstream module root",
                })),
            ));
        }

        let target_path = join_posix(out_dir, rel_target);
        let target_bytes = std::fs::read(&target_path).map_err(|err| {
            anyhow::Error::new(
                UserError::new(
                    "E_CONFIG_INVALID",
                    format!(
                        "patch target is missing for module {module_id} ({scope}): {rel_target}",
                        scope = scope
                    ),
                )
                .with_details(serde_json::json!({
                    "module_id": module_id,
                    "scope": scope,
                    "overlay_dir": overlay_dir.to_string_lossy(),
                    "patch_file": patch_file.to_string_lossy(),
                    "relpath": rel_target,
                    "target_path": target_path.to_string_lossy(),
                    "cause": err.to_string(),
                    "hint": "patch overlays currently only support patching existing upstream files",
                })),
            )
        })?;

        if std::str::from_utf8(&target_bytes).is_err() {
            return Err(anyhow::Error::new(
                UserError::new(
                    "E_CONFIG_INVALID",
                    format!(
                        "patch overlays only support UTF-8 text files: {rel_target} (module {module_id}, {scope})",
                        scope = scope
                    ),
                )
                .with_details(serde_json::json!({
                    "module_id": module_id,
                    "scope": scope,
                    "relpath": rel_target,
                    "target_path": target_path.to_string_lossy(),
                    "hint": "use a directory overlay for binary/non-UTF8 files",
                })),
            ));
        }

        let patch_bytes = std::fs::read(patch_file)
            .with_context(|| format!("read patch {}", patch_file.display()))?;
        let patch_text = std::str::from_utf8(&patch_bytes).map_err(|err| {
            anyhow::Error::new(
                UserError::new(
                    "E_CONFIG_INVALID",
                    format!(
                        "patch file is not UTF-8 for module {module_id} ({scope}): {}",
                        patch_file.display(),
                        scope = scope
                    ),
                )
                .with_details(serde_json::json!({
                    "module_id": module_id,
                    "scope": scope,
                    "patch_file": patch_file.to_string_lossy(),
                    "error": err.to_string(),
                })),
            )
        })?;

        validate_patch_text_matches_file(
            module_id,
            scope,
            overlay_dir,
            patch_file,
            patch_text,
            rel_target,
        )?;

        let out = Command::new("git")
            .arg("apply")
            .arg("--whitespace=nowarn")
            .arg(patch_file)
            .current_dir(out_dir)
            .output()
            .context("git apply")?;

        if !out.status.success() {
            return Err(anyhow::Error::new(
                UserError::new(
                    "E_OVERLAY_PATCH_APPLY_FAILED",
                    format!(
                        "failed to apply patch overlay for module {module_id} ({scope}): {rel_target}",
                        scope = scope
                    ),
                )
                .with_details(serde_json::json!({
                    "module_id": module_id,
                    "scope": scope,
                    "overlay_dir": overlay_dir.to_string_lossy(),
                    "patch_file": patch_file.to_string_lossy(),
                    "relpath": rel_target,
                    "stderr": String::from_utf8_lossy(&out.stderr),
                    "hint": "regenerate the patch against the current upstream (or lower overlays) content",
                })),
            ));
        }
    }

    Ok(())
}

fn validate_patch_text_matches_file(
    module_id: &str,
    scope: &str,
    overlay_dir: &Path,
    patch_file: &Path,
    patch_text: &str,
    expected_relpath: &str,
) -> anyhow::Result<()> {
    if patch_text.contains("GIT binary patch") {
        return Err(anyhow::Error::new(
            UserError::new(
                "E_CONFIG_INVALID",
                format!(
                    "patch overlays do not support binary patches (module {module_id}, {scope})",
                    scope = scope
                ),
            )
            .with_details(serde_json::json!({
                "module_id": module_id,
                "scope": scope,
                "overlay_dir": overlay_dir.to_string_lossy(),
                "patch_file": patch_file.to_string_lossy(),
                "relpath": expected_relpath,
                "hint": "use a directory overlay for binary files",
            })),
        ));
    }

    let mut old_lines = Vec::new();
    let mut new_lines = Vec::new();
    for line in patch_text.lines() {
        if let Some(rest) = line.strip_prefix("--- ") {
            old_lines.push(rest);
        } else if let Some(rest) = line.strip_prefix("+++ ") {
            new_lines.push(rest);
        }
    }

    if old_lines.len() != 1 || new_lines.len() != 1 {
        return Err(anyhow::Error::new(
            UserError::new(
                "E_CONFIG_INVALID",
                format!(
                    "invalid patch file (expected a single unified diff) for module {module_id} ({scope})",
                    scope = scope
                ),
            )
            .with_details(serde_json::json!({
                "module_id": module_id,
                "scope": scope,
                "overlay_dir": overlay_dir.to_string_lossy(),
                "patch_file": patch_file.to_string_lossy(),
                "relpath": expected_relpath,
                "hint": "generate patches using `git diff --no-index -- <upstream-file> <edited-file>` and store the output in .agentpack/patches/<relpath>.patch",
            })),
        ));
    }

    let old_path = parse_patch_header_path(old_lines[0]);
    let new_path = parse_patch_header_path(new_lines[0]);
    if old_path == "/dev/null" || new_path == "/dev/null" {
        return Err(anyhow::Error::new(
            UserError::new(
                "E_CONFIG_INVALID",
                format!(
                    "patch overlays do not currently support file create/delete (module {module_id}, {scope})",
                    scope = scope
                ),
            )
            .with_details(serde_json::json!({
                "module_id": module_id,
                "scope": scope,
                "overlay_dir": overlay_dir.to_string_lossy(),
                "patch_file": patch_file.to_string_lossy(),
                "relpath": expected_relpath,
                "hint": "use a directory overlay for create/delete operations",
            })),
        ));
    }

    let old_norm = strip_ab_prefix(old_path);
    let new_norm = strip_ab_prefix(new_path);
    if old_norm != expected_relpath || new_norm != expected_relpath {
        return Err(anyhow::Error::new(
            UserError::new(
                "E_CONFIG_INVALID",
                format!(
                    "patch file paths do not match filename-derived relpath for module {module_id} ({scope})",
                    scope = scope
                ),
            )
            .with_details(serde_json::json!({
                "module_id": module_id,
                "scope": scope,
                "overlay_dir": overlay_dir.to_string_lossy(),
                "patch_file": patch_file.to_string_lossy(),
                "expected_relpath": expected_relpath,
                "patch_old_path": old_norm,
                "patch_new_path": new_norm,
                "hint": "ensure the patch header uses the same path as the patch file name (e.g. --- a/<relpath> / +++ b/<relpath>)",
            })),
        ));
    }

    Ok(())
}

fn parse_patch_header_path(value: &str) -> &str {
    // diff -u may append timestamps after a tab; git diff usually does not.
    value.split_whitespace().next().unwrap_or("")
}

fn strip_ab_prefix(path: &str) -> String {
    path.strip_prefix("a/")
        .or_else(|| path.strip_prefix("b/"))
        .unwrap_or(path)
        .to_string()
}

fn validate_posix_relpath(relpath: &str) -> bool {
    if relpath.is_empty() || relpath.starts_with('/') {
        return false;
    }
    relpath
        .split('/')
        .all(|seg| !seg.is_empty() && seg != "." && seg != "..")
}

fn join_posix(root: &Path, rel_posix: &str) -> PathBuf {
    let mut out = root.to_path_buf();
    for part in rel_posix.split('/') {
        out.push(part);
    }
    out
}

fn path_relative_posix(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

pub fn resolve_upstream_module_root(
    home: &AgentpackHome,
    repo: &RepoPaths,
    module: &Module,
) -> anyhow::Result<PathBuf> {
    let repo_root = repo
        .manifest_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| repo.repo_dir.clone());

    match module.source.kind() {
        SourceKind::LocalPath => {
            let lp = module
                .source
                .local_path
                .as_ref()
                .context("missing local_path")?;
            Ok(repo_root.join(&lp.path))
        }
        SourceKind::Git => {
            // Prefer lockfile resolution for reproducibility.
            let lock = Lockfile::load(&repo.lockfile_path).ok();
            if let Some(lock) = lock {
                if let Some(lm) = lock.modules.iter().find(|m| m.id == module.id) {
                    if let Some(gs) = &lm.resolved_source.git {
                        let store = Store::new(home);
                        let checkout_dir = ensure_locked_git_checkout(&store, &module.id, gs)?;
                        let root = Store::module_root_in_checkout(&checkout_dir, &gs.subdir);
                        return Ok(root);
                    }
                }
            }

            // Fallback to manifest ref resolution.
            let src = module.source.git.as_ref().context("missing git source")?;
            let store = Store::new(home);
            let commit = store.resolve_git_commit(src)?;
            let checkout_dir = store.ensure_git_checkout(&module.id, src, &commit)?;
            Ok(Store::module_root_in_checkout(&checkout_dir, &src.subdir))
        }
        SourceKind::Invalid => anyhow::bail!("invalid source for module {}", module.id),
    }
}

fn ensure_locked_git_checkout(
    store: &Store,
    module_id: &str,
    locked: &crate::lockfile::ResolvedGitSource,
) -> anyhow::Result<PathBuf> {
    // Lockfile stores the exact commit; use the commit itself as the ref name to avoid
    // requiring the original branch/tag in order to populate the checkout.
    let src = GitSource {
        url: locked.url.clone(),
        ref_name: locked.commit.clone(),
        subdir: locked.subdir.clone(),
        shallow: false,
    };
    store.ensure_git_checkout(module_id, &src, &locked.commit)
}

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
            "overlay drift ({overlay_kind}) module {module_id}: upstream changed ({baseline} -> {current})",
            baseline = baseline_hash,
            current = current_hash
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
    let patch_files = list_patch_files(overlay_dir)
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
                        format!(
                            "overlay_kind=dir but patch artifacts exist for module {module_id}"
                        ),
                    )
                    .with_details(serde_json::json!({
                        "module_id": module_id,
                        "overlay_dir": overlay_dir.to_string_lossy(),
                        "hint": "set overlay_kind=patch (in .agentpack/overlay.json) or remove .agentpack/patches",
                    })),
                ));
            }

            match resolve_rebase_base(home, module, &baseline, &repo_root)? {
                RebaseBase::Dir(base_root) => rebase_overlay_dir_files(
                    overlay_dir,
                    &baseline_map,
                    |rel| read_optional_file_bytes(&base_root, rel),
                    |rel| read_optional_file_bytes(&upstream_root, rel),
                    options,
                )?,
                RebaseBase::RepoGit {
                    repo_git_rev,
                    repo_rel_path,
                } => {
                    let repo_git_rev = repo_git_rev.clone();
                    let repo_rel_path = repo_rel_path.clone();
                    rebase_overlay_dir_files(
                        overlay_dir,
                        &baseline_map,
                        |rel| {
                            git_show_optional_bytes(&repo_root, &repo_git_rev, &repo_rel_path, rel)
                        },
                        |rel| read_optional_file_bytes(&upstream_root, rel),
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

            match resolve_rebase_base(home, module, &baseline, &repo_root)? {
                RebaseBase::Dir(base_root) => rebase_overlay_patch_files(
                    module_id,
                    overlay_dir,
                    &patch_files,
                    &baseline_map,
                    |rel| read_optional_file_bytes(&base_root, rel),
                    |rel| read_optional_file_bytes(&upstream_root, rel),
                    options,
                )?,
                RebaseBase::RepoGit {
                    repo_git_rev,
                    repo_rel_path,
                } => {
                    let repo_git_rev = repo_git_rev.clone();
                    let repo_rel_path = repo_rel_path.clone();
                    rebase_overlay_patch_files(
                        module_id,
                        overlay_dir,
                        &patch_files,
                        &baseline_map,
                        |rel| {
                            git_show_optional_bytes(&repo_root, &repo_git_rev, &repo_rel_path, rel)
                        },
                        |rel| read_optional_file_bytes(&upstream_root, rel),
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

enum RebaseBase {
    Dir(PathBuf),
    RepoGit {
        repo_git_rev: String,
        repo_rel_path: String,
    },
}

fn resolve_rebase_base(
    home: &AgentpackHome,
    module: &Module,
    baseline: &OverlayBaseline,
    repo_root: &Path,
) -> anyhow::Result<RebaseBase> {
    if let Some(upstream) = baseline.upstream.clone() {
        match upstream {
            BaselineUpstream::Git {
                url,
                commit,
                subdir,
            } => {
                let store = Store::new(home);
                let src = GitSource {
                    url,
                    ref_name: commit.clone(),
                    subdir: subdir.clone(),
                    shallow: false,
                };
                let checkout_dir = store.ensure_git_checkout(&module.id, &src, &commit)?;
                return Ok(RebaseBase::Dir(Store::module_root_in_checkout(
                    &checkout_dir,
                    &subdir,
                )));
            }
            BaselineUpstream::LocalPath {
                repo_rel_path,
                repo_git_rev,
                repo_dirty: _,
            } => {
                let Some(repo_git_rev) = repo_git_rev else {
                    return Err(anyhow::Error::new(
                        UserError::new(
                            "E_OVERLAY_BASELINE_UNSUPPORTED",
                            "overlay baseline is missing repo git revision; cannot locate merge base"
                                .to_string(),
                        )
                        .with_details(serde_json::json!({
                            "repo_rel_path": repo_rel_path,
                            "hint": "ensure the config repo is a git repo and recreate the overlay baseline",
                        })),
                    ));
                };
                return Ok(RebaseBase::RepoGit {
                    repo_git_rev,
                    repo_rel_path,
                });
            }
        }
    }

    // Backwards-compatibility: older baselines don't include upstream identity.
    if module.source.kind() == SourceKind::Git {
        let subdir = module
            .source
            .git
            .as_ref()
            .map(|g| g.subdir.clone())
            .unwrap_or_default();
        if let Some((_, root)) =
            find_git_checkout_matching_hash(home, &module.id, &subdir, &baseline.upstream_sha256)?
        {
            return Ok(RebaseBase::Dir(root));
        }
    }

    if module.source.kind() == SourceKind::LocalPath && repo_root.join(".git").exists() {
        return Err(anyhow::Error::new(
            UserError::new(
                "E_OVERLAY_BASELINE_UNSUPPORTED",
                "overlay baseline is missing upstream identity; cannot locate merge base"
                    .to_string(),
            )
            .with_details(serde_json::json!({
                "hint": "recreate the overlay baseline (agentpack overlay edit) after committing the repo state",
            })),
        ));
    }

    Err(anyhow::Error::new(UserError::new(
        "E_OVERLAY_BASELINE_UNSUPPORTED",
        "overlay baseline is missing upstream identity; cannot locate merge base".to_string(),
    )))
}

fn find_git_checkout_matching_hash(
    home: &AgentpackHome,
    module_id: &str,
    subdir: &str,
    baseline_hash: &str,
) -> anyhow::Result<Option<(String, PathBuf)>> {
    let dir = home
        .cache_dir
        .join("git")
        .join(crate::ids::module_fs_key(module_id));
    let entries = match std::fs::read_dir(&dir) {
        Ok(e) => e,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(err) => return Err(err).with_context(|| format!("read_dir {}", dir.display())),
    };

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let commit = entry.file_name().to_string_lossy().to_string();
        let root = Store::module_root_in_checkout(&path, subdir);
        if !root.exists() {
            continue;
        }
        let (_, hash) = hash_tree(&root).with_context(|| format!("hash {}", root.display()))?;
        if hash == baseline_hash {
            return Ok(Some((commit, root)));
        }
    }

    Ok(None)
}

fn read_optional_file_bytes(root: &Path, rel_posix: &str) -> anyhow::Result<Option<Vec<u8>>> {
    let path = root.join(rel_posix);
    match std::fs::read(&path) {
        Ok(b) => Ok(Some(b)),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(err).with_context(|| format!("read {}", path.display())),
    }
}

fn git_show_optional_bytes(
    repo_root: &Path,
    repo_git_rev: &str,
    repo_rel_root_posix: &str,
    rel_posix: &str,
) -> anyhow::Result<Option<Vec<u8>>> {
    let mut full = repo_rel_root_posix.trim_end_matches('/').to_string();
    if !full.is_empty() {
        full.push('/');
    }
    full.push_str(rel_posix);
    let spec = format!("{repo_git_rev}:{full}");

    let out = Command::new("git")
        .current_dir(repo_root)
        .arg("show")
        .arg(spec)
        .output()
        .context("git show")?;

    if out.status.success() {
        Ok(Some(out.stdout))
    } else {
        Ok(None)
    }
}

struct MergeOutcome {
    merged: Vec<u8>,
    conflicted: bool,
}

fn merge_three_way_git(base: &[u8], ours: &[u8], theirs: &[u8]) -> anyhow::Result<MergeOutcome> {
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

fn rebase_overlay_patch_files(
    module_id: &str,
    overlay_dir: &Path,
    patch_files: &[PathBuf],
    baseline_map: &BTreeMap<String, String>,
    read_base: impl Fn(&str) -> anyhow::Result<Option<Vec<u8>>>,
    read_upstream: impl Fn(&str) -> anyhow::Result<Option<Vec<u8>>>,
    options: OverlayRebaseOptions,
) -> anyhow::Result<OverlayRebaseReport> {
    let patches_root = overlay_dir.join(".agentpack").join("patches");

    let mut files = patch_files.to_vec();
    files.sort();

    let mut report = OverlayRebaseReport::default();
    for patch_file in files {
        report.summary.processed_files += 1;

        let rel_patch = patch_file
            .strip_prefix(&patches_root)
            .unwrap_or(&patch_file);
        let rel_patch_posix = rel_patch.to_string_lossy().replace('\\', "/");
        let Some(rel_target) = rel_patch_posix.strip_suffix(".patch") else {
            report.summary.skipped_files += 1;
            report.skipped.push(rel_patch_posix);
            continue;
        };

        if !validate_posix_relpath(rel_target) {
            return Err(anyhow::Error::new(
                UserError::new(
                    "E_CONFIG_INVALID",
                    format!("invalid patch relpath for module {module_id}: {rel_target}"),
                )
                .with_details(serde_json::json!({
                    "module_id": module_id,
                    "overlay_dir": overlay_dir.to_string_lossy(),
                    "patch_file": patch_file.to_string_lossy(),
                    "relpath": rel_target,
                    "hint": "patch filenames must map to a safe relpath under the upstream module root",
                })),
            ));
        }

        if !baseline_map.contains_key(rel_target) {
            report.summary.skipped_files += 1;
            report.skipped.push(rel_target.to_string());
            continue;
        }

        let base =
            read_base(rel_target)?.with_context(|| format!("missing base for {rel_target}"))?;

        let expected_sha = baseline_map
            .get(rel_target)
            .expect("baseline_map contains rel_target");
        let got_sha = sha256_hex(&base);
        if got_sha != *expected_sha {
            return Err(anyhow::Error::new(
                UserError::new(
                    "E_OVERLAY_BASELINE_UNSUPPORTED",
                    format!("overlay baseline does not match merge base for {rel_target}"),
                )
                .with_details(serde_json::json!({
                    "path": rel_target,
                    "expected_sha256": expected_sha,
                    "base_sha256": got_sha,
                    "hint": "recreate the overlay baseline after committing upstream changes",
                })),
            ));
        }

        let ours =
            apply_patch_to_base_for_rebase(module_id, overlay_dir, &patch_file, rel_target, &base)?;

        let upstream = read_upstream(rel_target)?;
        let Some(upstream) = upstream else {
            report.summary.conflict_files += 1;
            report.conflicts.push(rel_target.to_string());
            if !options.dry_run {
                let conflict = conflict_markers_for_deleted_upstream(&ours)?;
                write_patch_conflict_artifact(overlay_dir, rel_target, &conflict)?;
            }
            continue;
        };

        if std::str::from_utf8(&base).is_err()
            || std::str::from_utf8(&ours).is_err()
            || std::str::from_utf8(&upstream).is_err()
        {
            return Err(anyhow::Error::new(
                UserError::new(
                    "E_CONFIG_INVALID",
                    format!("patch overlays only support UTF-8 text files: {rel_target}"),
                )
                .with_details(serde_json::json!({
                    "module_id": module_id,
                    "overlay_dir": overlay_dir.to_string_lossy(),
                    "relpath": rel_target,
                    "hint": "use a directory overlay for binary/non-UTF8 files",
                })),
            ));
        }

        let merged = merge_three_way_git(&base, &ours, &upstream)?;
        if merged.conflicted {
            report.summary.conflict_files += 1;
            report.conflicts.push(rel_target.to_string());
            if !options.dry_run {
                write_patch_conflict_artifact(overlay_dir, rel_target, &merged.merged)?;
                // Keep the patch overlay consistent with the updated upstream by rewriting the
                // patch to produce the conflict-marked merged content.
                if let Some(bytes) =
                    compute_patch_from_upstream(rel_target, &upstream, &merged.merged)?
                {
                    write_atomic(&patch_file, &bytes)
                        .with_context(|| format!("write {}", patch_file.display()))?;
                }
            }
            continue;
        }

        let new_patch = compute_patch_from_upstream(rel_target, &upstream, &merged.merged)?;
        match new_patch {
            None => {
                if !options.dry_run {
                    delete_overlay_file(overlay_dir, &patch_file, false)?;
                }
                report.summary.deleted_files += 1;
                report.deleted.push(rel_target.to_string());
            }
            Some(bytes) => {
                if !options.dry_run {
                    write_atomic(&patch_file, &bytes)
                        .with_context(|| format!("write {}", patch_file.display()))?;
                }
                report.summary.updated_files += 1;
                report.updated.push(rel_target.to_string());
            }
        }
    }

    Ok(report)
}

fn apply_patch_to_base_for_rebase(
    module_id: &str,
    overlay_dir: &Path,
    patch_file: &Path,
    rel_target: &str,
    base: &[u8],
) -> anyhow::Result<Vec<u8>> {
    if std::str::from_utf8(base).is_err() {
        return Err(anyhow::Error::new(
            UserError::new(
                "E_CONFIG_INVALID",
                format!("patch overlays only support UTF-8 text files: {rel_target}"),
            )
            .with_details(serde_json::json!({
                "module_id": module_id,
                "overlay_dir": overlay_dir.to_string_lossy(),
                "patch_file": patch_file.to_string_lossy(),
                "relpath": rel_target,
                "hint": "use a directory overlay for binary/non-UTF8 files",
            })),
        ));
    }

    let patch_bytes =
        std::fs::read(patch_file).with_context(|| format!("read {}", patch_file.display()))?;
    let patch_text = std::str::from_utf8(&patch_bytes).map_err(|err| {
        anyhow::Error::new(
            UserError::new(
                "E_CONFIG_INVALID",
                format!(
                    "patch file is not UTF-8 for module {module_id}: {}",
                    patch_file.display()
                ),
            )
            .with_details(serde_json::json!({
                "module_id": module_id,
                "overlay_dir": overlay_dir.to_string_lossy(),
                "patch_file": patch_file.to_string_lossy(),
                "error": err.to_string(),
            })),
        )
    })?;
    validate_patch_text_matches_file(
        module_id,
        "rebase",
        overlay_dir,
        patch_file,
        patch_text,
        rel_target,
    )?;

    let td = tempfile::tempdir().context("create tempdir")?;
    let target_path = join_posix(td.path(), rel_target);
    write_atomic(&target_path, base).with_context(|| format!("write {}", target_path.display()))?;

    let out = Command::new("git")
        .arg("apply")
        .arg("--whitespace=nowarn")
        .arg(patch_file)
        .current_dir(td.path())
        .output()
        .context("git apply")?;

    if !out.status.success() {
        return Err(anyhow::Error::new(
            UserError::new(
                "E_CONFIG_INVALID",
                format!("patch does not apply to baseline for {rel_target} (module {module_id})"),
            )
            .with_details(serde_json::json!({
                "module_id": module_id,
                "overlay_dir": overlay_dir.to_string_lossy(),
                "patch_file": patch_file.to_string_lossy(),
                "relpath": rel_target,
                "stderr": String::from_utf8_lossy(&out.stderr),
                "hint": "regenerate the patch against the baseline content (or recreate the overlay baseline)",
            })),
        ));
    }

    std::fs::read(&target_path).with_context(|| format!("read {}", target_path.display()))
}

fn compute_patch_from_upstream(
    rel_target: &str,
    upstream: &[u8],
    merged: &[u8],
) -> anyhow::Result<Option<Vec<u8>>> {
    let td = tempfile::tempdir().context("create tempdir")?;

    let a_root = td.path().join("a");
    let b_root = td.path().join("b");
    let a_path = join_posix(&a_root, rel_target);
    let b_path = join_posix(&b_root, rel_target);

    write_atomic(&a_path, upstream).with_context(|| format!("write {}", a_path.display()))?;
    write_atomic(&b_path, merged).with_context(|| format!("write {}", b_path.display()))?;

    let a_rel = format!("a/{rel_target}");
    let b_rel = format!("b/{rel_target}");

    let out = Command::new("git")
        .current_dir(td.path())
        .arg("diff")
        .arg("--no-index")
        .arg("--src-prefix=")
        .arg("--dst-prefix=")
        .arg("--")
        .arg(a_rel)
        .arg(b_rel)
        .output()
        .context("git diff --no-index")?;

    match out.status.code() {
        Some(0) => Ok(None),
        Some(1) => Ok(Some(out.stdout)),
        _ => anyhow::bail!(
            "git diff --no-index failed: {}",
            String::from_utf8_lossy(&out.stderr)
        ),
    }
}

fn write_patch_conflict_artifact(
    overlay_dir: &Path,
    rel_target: &str,
    bytes: &[u8],
) -> anyhow::Result<()> {
    let root = overlay_dir.join(".agentpack").join("conflicts");
    let path = join_posix(&root, rel_target);
    write_atomic(&path, bytes).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

fn conflict_markers_for_deleted_upstream(ours: &[u8]) -> anyhow::Result<Vec<u8>> {
    let ours = std::str::from_utf8(ours).context("conflict content must be UTF-8")?;
    Ok(format!("<<<<<<< ours\n{ours}\n=======\n>>>>>>> theirs (deleted upstream)\n").into_bytes())
}

fn rebase_overlay_dir_files(
    overlay_dir: &Path,
    baseline_map: &BTreeMap<String, String>,
    read_base: impl Fn(&str) -> anyhow::Result<Option<Vec<u8>>>,
    read_upstream: impl Fn(&str) -> anyhow::Result<Option<Vec<u8>>>,
    options: OverlayRebaseOptions,
) -> anyhow::Result<OverlayRebaseReport> {
    let mut files = list_files(overlay_dir)?;
    files.sort();

    let mut report = OverlayRebaseReport::default();
    for file in files {
        report.summary.processed_files += 1;
        let rel_path = file.strip_prefix(overlay_dir).unwrap_or(&file);
        let rel_posix = rel_path.to_string_lossy().replace('\\', "/");

        if !baseline_map.contains_key(&rel_posix) {
            report.summary.skipped_files += 1;
            report.skipped.push(rel_posix);
            continue;
        }

        let ours = std::fs::read(&file).with_context(|| format!("read {}", file.display()))?;
        let base =
            read_base(&rel_posix)?.with_context(|| format!("missing base for {rel_posix}"))?;

        let expected_sha = baseline_map
            .get(&rel_posix)
            .expect("baseline_map contains rel_posix");
        let got_sha = sha256_hex(&base);
        if got_sha != *expected_sha {
            return Err(anyhow::Error::new(
                UserError::new(
                    "E_OVERLAY_BASELINE_UNSUPPORTED",
                    format!("overlay baseline does not match merge base for {rel_posix}"),
                )
                .with_details(serde_json::json!({
                    "path": rel_posix,
                    "expected_sha256": expected_sha,
                    "base_sha256": got_sha,
                    "hint": "recreate the overlay baseline after committing upstream changes",
                })),
            ));
        }
        let upstream = read_upstream(&rel_posix)?;

        match upstream {
            None => {
                if ours == base {
                    delete_overlay_file(overlay_dir, &file, options.dry_run)?;
                    report.summary.deleted_files += 1;
                    report.deleted.push(rel_posix);
                    continue;
                }

                report.summary.skipped_files += 1;
                report.skipped.push(rel_posix);
            }
            Some(upstream) => {
                if ours == base {
                    if options.sparsify {
                        delete_overlay_file(overlay_dir, &file, options.dry_run)?;
                        report.summary.deleted_files += 1;
                        report.deleted.push(rel_posix);
                    } else if ours != upstream {
                        if !options.dry_run {
                            write_atomic(&file, &upstream)
                                .with_context(|| format!("write {}", file.display()))?;
                        }
                        report.summary.updated_files += 1;
                        report.updated.push(rel_posix);
                    }
                    continue;
                }

                if upstream == base {
                    continue;
                }

                if ours == upstream {
                    if options.sparsify {
                        delete_overlay_file(overlay_dir, &file, options.dry_run)?;
                        report.summary.deleted_files += 1;
                        report.deleted.push(rel_posix);
                    }
                    continue;
                }

                let merged = merge_three_way_git(&base, &ours, &upstream)?;
                if merged.conflicted {
                    report.summary.conflict_files += 1;
                    report.conflicts.push(rel_posix.clone());
                }

                if options.sparsify && !merged.conflicted && merged.merged == upstream {
                    delete_overlay_file(overlay_dir, &file, options.dry_run)?;
                    report.summary.deleted_files += 1;
                    report.deleted.push(rel_posix);
                } else {
                    if !options.dry_run {
                        write_atomic(&file, &merged.merged)
                            .with_context(|| format!("write {}", file.display()))?;
                    }
                    report.summary.updated_files += 1;
                    report.updated.push(rel_posix);
                }
            }
        }
    }

    Ok(report)
}

fn delete_overlay_file(overlay_dir: &Path, file: &Path, dry_run: bool) -> anyhow::Result<()> {
    if dry_run {
        return Ok(());
    }

    std::fs::remove_file(file).with_context(|| format!("remove {}", file.display()))?;
    prune_empty_parents(
        file.parent().with_context(|| "missing file parent")?,
        overlay_dir,
    )?;
    Ok(())
}

fn prune_empty_parents(mut dir: &Path, stop: &Path) -> anyhow::Result<()> {
    while dir != stop {
        let mut entries =
            std::fs::read_dir(dir).with_context(|| format!("read_dir {}", dir.display()))?;
        if entries.next().is_some() {
            break;
        }
        std::fs::remove_dir(dir).with_context(|| format!("remove_dir {}", dir.display()))?;
        dir = dir.parent().with_context(|| "missing parent")?;
    }
    Ok(())
}

fn overlay_baseline_path(overlay_dir: &Path) -> PathBuf {
    overlay_dir.join(".agentpack").join("baseline.json")
}

fn overlay_module_id_path(overlay_dir: &Path) -> PathBuf {
    overlay_dir.join(".agentpack").join("module_id")
}

fn overlay_meta_path(overlay_dir: &Path) -> PathBuf {
    overlay_dir.join(".agentpack").join("overlay.json")
}

fn write_overlay_module_id(module_id: &str, overlay_dir: &Path) -> anyhow::Result<()> {
    let meta_dir = overlay_dir.join(".agentpack");
    std::fs::create_dir_all(&meta_dir).context("create overlay metadata dir")?;

    let path = overlay_module_id_path(overlay_dir);
    let mut content = module_id.to_string();
    if !content.ends_with('\n') {
        content.push('\n');
    }
    write_atomic(&path, content.as_bytes()).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

fn write_overlay_meta_default_dir(overlay_dir: &Path) -> anyhow::Result<()> {
    let meta_dir = overlay_dir.join(".agentpack");
    std::fs::create_dir_all(&meta_dir).context("create overlay metadata dir")?;

    let path = overlay_meta_path(overlay_dir);
    let meta = OverlayMeta {
        overlay_kind: OverlayKind::Dir,
    };

    let mut out = serde_json::to_string_pretty(&meta).context("serialize overlay meta")?;
    if !out.ends_with('\n') {
        out.push('\n');
    }

    write_atomic(&path, out.as_bytes()).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

fn read_overlay_meta(overlay_dir: &Path) -> anyhow::Result<OverlayMeta> {
    let path = overlay_meta_path(overlay_dir);
    if !path.exists() {
        return Ok(OverlayMeta {
            overlay_kind: OverlayKind::Dir,
        });
    }

    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let meta: OverlayMeta = serde_json::from_str(&raw).map_err(|err| {
        anyhow::Error::new(
            UserError::new(
                "E_CONFIG_INVALID",
                format!(
                    "invalid overlay metadata (expected JSON) at {}: {}",
                    path.display(),
                    err
                ),
            )
            .with_details(serde_json::json!({
                "overlay_dir": overlay_dir.to_string_lossy(),
                "meta_path": path.to_string_lossy(),
                "hint": "delete the file to fall back to overlay_kind=dir, or fix it to include {\"overlay_kind\": \"dir\"|\"patch\"}",
            })),
        )
    })?;

    Ok(meta)
}

fn write_overlay_baseline(
    home: &AgentpackHome,
    repo: &RepoPaths,
    module: &Module,
    upstream_root: &Path,
    overlay_dir: &Path,
) -> anyhow::Result<()> {
    let (file_manifest, module_hash) = hash_tree(upstream_root)
        .with_context(|| format!("hash upstream {}", upstream_root.display()))?;

    let created_at = time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .context("format timestamp")?;

    let upstream = match module.source.kind() {
        SourceKind::Git => write_baseline_upstream_git(home, repo, module)?,
        SourceKind::LocalPath => write_baseline_upstream_local(repo, upstream_root)?,
        SourceKind::Invalid => None,
    };

    let baseline = OverlayBaseline {
        version: 2,
        created_at,
        upstream_sha256: module_hash,
        file_manifest,
        upstream,
    };

    let meta_dir = overlay_dir.join(".agentpack");
    std::fs::create_dir_all(&meta_dir).context("create overlay metadata dir")?;

    let baseline_path = overlay_baseline_path(overlay_dir);
    let mut out = serde_json::to_string_pretty(&baseline).context("serialize baseline")?;
    if !out.ends_with('\n') {
        out.push('\n');
    }
    write_atomic(&baseline_path, out.as_bytes())
        .with_context(|| format!("write {}", baseline_path.display()))?;
    Ok(())
}

fn write_baseline_upstream_git(
    home: &AgentpackHome,
    repo: &RepoPaths,
    module: &Module,
) -> anyhow::Result<Option<BaselineUpstream>> {
    let lock = Lockfile::load(&repo.lockfile_path).ok();
    if let Some(lock) = lock {
        if let Some(lm) = lock.modules.iter().find(|m| m.id == module.id) {
            if let Some(gs) = &lm.resolved_source.git {
                return Ok(Some(BaselineUpstream::Git {
                    url: gs.url.clone(),
                    commit: gs.commit.clone(),
                    subdir: gs.subdir.clone(),
                }));
            }
        }
    }

    let src = module.source.git.as_ref().context("missing git source")?;
    let store = Store::new(home);
    let commit = store.resolve_git_commit(src)?;
    Ok(Some(BaselineUpstream::Git {
        url: src.url.clone(),
        commit,
        subdir: src.subdir.clone(),
    }))
}

fn write_baseline_upstream_local(
    repo: &RepoPaths,
    upstream_root: &Path,
) -> anyhow::Result<Option<BaselineUpstream>> {
    let repo_root = repo
        .manifest_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| repo.repo_dir.clone());

    let repo_rel_path = upstream_root
        .strip_prefix(&repo_root)
        .ok()
        .map(|p| p.to_string_lossy().replace('\\', "/"));
    let Some(repo_rel_path) = repo_rel_path else {
        return Ok(None);
    };

    let repo_git_rev = crate::git::git_in(&repo_root, &["rev-parse", "HEAD"])
        .ok()
        .map(|s| s.trim().to_string());
    let repo_dirty = crate::git::git_in(&repo_root, &["status", "--porcelain"])
        .ok()
        .map(|s| !s.trim().is_empty());

    Ok(Some(BaselineUpstream::LocalPath {
        repo_rel_path,
        repo_git_rev,
        repo_dirty,
    }))
}
