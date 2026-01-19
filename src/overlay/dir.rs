use std::collections::BTreeMap;
use std::path::Path;

use anyhow::Context as _;

use crate::fs::{list_files, write_atomic};
use crate::hash::sha256_hex;
use crate::user_error::UserError;

use super::layout::delete_overlay_file;
use super::rebase::{OverlayRebaseOptions, OverlayRebaseReport, merge_three_way_git};

pub(super) fn rebase_overlay_dir_files(
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
