mod dir;
mod layout;
mod patch;
mod rebase;

use std::path::{Path, PathBuf};

use anyhow::Context as _;

use crate::fs::{copy_tree, list_files};
use crate::user_error::UserError;

pub use layout::{
    ensure_overlay_skeleton, ensure_overlay_skeleton_sparse, materialize_overlay_from_upstream,
    resolve_upstream_module_root,
};
pub use patch::ensure_patch_overlay_layout;
pub use rebase::{
    OverlayRebaseOptions, OverlayRebaseReport, OverlayRebaseSummary, overlay_drift_warnings,
    rebase_overlay,
};

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

        let meta = layout::read_overlay_meta(overlay.dir)?;
        let override_files = list_files(overlay.dir)
            .with_context(|| format!("list overlay files {}", overlay.dir.display()))?;
        let patch_files = patch::list_patch_files(overlay.dir)
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
                    "override_files": override_files.iter().map(|p| layout::path_relative_posix(overlay.dir, p)).collect::<Vec<_>>(),
                    "patch_files": patch_files.iter().map(|p| layout::path_relative_posix(overlay.dir, p)).collect::<Vec<_>>(),
                    "hint": "use a single overlay kind per overlay directory (dir OR patch)",
                })),
            ));
        }

        match meta.overlay_kind {
            layout::OverlayKind::Dir => {
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
            layout::OverlayKind::Patch => {
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

                patch::apply_patch_overlays(
                    module_id,
                    overlay.scope,
                    overlay.dir,
                    out_dir,
                    &patch_files,
                )?;
            }
        }
    }
    Ok(())
}
