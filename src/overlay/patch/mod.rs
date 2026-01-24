use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::Context as _;
use walkdir::WalkDir;

use crate::fs::list_files;
use crate::user_error::UserError;

use super::layout::{
    OverlayKind, join_posix, path_relative_posix, read_overlay_meta, validate_posix_relpath,
    write_overlay_meta,
};

mod rebase;

pub fn ensure_patch_overlay_layout(module_id: &str, overlay_dir: &Path) -> anyhow::Result<PathBuf> {
    let override_files = list_files(overlay_dir)?;
    if !override_files.is_empty() {
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
                "override_files": override_files.iter().map(|p| path_relative_posix(overlay_dir, p)).collect::<Vec<_>>(),
                "hint": "move edits into .agentpack/patches/*.patch or use overlay_kind=dir",
            })),
        ));
    }

    let meta = read_overlay_meta(overlay_dir)?;
    if meta.overlay_kind != OverlayKind::Patch {
        write_overlay_meta(overlay_dir, OverlayKind::Patch)?;
    }

    let patches_dir = overlay_dir.join(".agentpack").join("patches");
    std::fs::create_dir_all(&patches_dir)
        .with_context(|| format!("create {}", patches_dir.display()))?;
    Ok(patches_dir)
}

pub(super) fn list_patch_files(overlay_dir: &Path) -> anyhow::Result<Vec<PathBuf>> {
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

pub(super) fn apply_patch_overlays(
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
                        "invalid patch relpath for module {module_id} ({scope}): {rel_target}"
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
                        "patch target is missing for module {module_id} ({scope}): {rel_target}"
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
                        "patch overlays only support UTF-8 text files: {rel_target} (module {module_id}, {scope})"
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
            .arg("-c")
            .arg("core.autocrlf=false")
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
                        "failed to apply patch overlay for module {module_id} ({scope}): {rel_target}"
                    ),
                )
                .with_details(serde_json::json!({
                    "module_id": module_id,
                    "scope": scope,
                    "overlay_dir": overlay_dir.to_string_lossy(),
                    "patch_file": patch_file.to_string_lossy(),
                    "relpath": rel_target,
                    "stderr": String::from_utf8_lossy(&out.stderr),
                    "reason_code": "overlay_patch_apply_failed",
                    "next_actions": ["regenerate_patch", "switch_to_dir_overlay", "retry_command"],
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
                    "patch overlays do not support binary patches (module {module_id}, {scope})"
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
                    "invalid patch file (expected a single unified diff) for module {module_id} ({scope})"
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
                    "patch overlays do not currently support file create/delete (module {module_id}, {scope})"
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
                    "patch file paths do not match filename-derived relpath for module {module_id} ({scope})"
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

pub(super) fn rebase_overlay_patch_files(
    module_id: &str,
    overlay_dir: &Path,
    patch_files: &[PathBuf],
    baseline_map: &std::collections::BTreeMap<String, String>,
    read_base: impl Fn(&str) -> anyhow::Result<Option<Vec<u8>>>,
    read_upstream: impl Fn(&str) -> anyhow::Result<Option<Vec<u8>>>,
    options: super::rebase::OverlayRebaseOptions,
) -> anyhow::Result<super::rebase::OverlayRebaseReport> {
    rebase::rebase_overlay_patch_files(
        module_id,
        overlay_dir,
        patch_files,
        baseline_map,
        read_base,
        read_upstream,
        options,
    )
}
