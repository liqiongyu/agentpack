use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::Context as _;

use crate::fs::write_atomic;
use crate::hash::sha256_hex;
use crate::user_error::UserError;

use super::super::layout::{delete_overlay_file, join_posix, validate_posix_relpath};
use super::super::rebase::{OverlayRebaseOptions, OverlayRebaseReport, merge_three_way_git};

pub(super) fn rebase_overlay_patch_files(
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
    super::validate_patch_text_matches_file(
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
        .arg("-c")
        .arg("core.autocrlf=false")
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
        .arg("-c")
        .arg("core.autocrlf=false")
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
