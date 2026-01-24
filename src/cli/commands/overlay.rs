use anyhow::Context as _;

use crate::engine::Engine;
use crate::output::{JsonEnvelope, print_json};
use crate::overlay::{
    OverlayRebaseOptions, ensure_overlay_skeleton, ensure_overlay_skeleton_sparse,
    ensure_patch_overlay_layout, materialize_overlay_from_upstream, rebase_overlay,
};
use crate::user_error::UserError;

use super::super::args::{OverlayCommands, OverlayScope};
use super::Ctx;

pub(crate) fn run(ctx: &Ctx<'_>, command: &OverlayCommands) -> anyhow::Result<()> {
    match command {
        OverlayCommands::Edit {
            module_id,
            scope,
            kind,
            project,
            sparse,
            materialize,
        } => {
            super::super::util::require_yes_for_json_mutation(ctx.cli, "overlay edit")?;
            let engine = Engine::load(ctx.cli.repo.as_deref(), ctx.cli.machine.as_deref())?;
            let mut warnings: Vec<String> = Vec::new();
            let module_id_str = module_id.as_str();

            let mut effective_scope = *scope;
            if *project {
                if *scope != OverlayScope::Global {
                    warnings.push(
                        "--project is deprecated; ignoring --scope and using project scope"
                            .to_string(),
                    );
                }
                effective_scope = OverlayScope::Project;
            }

            let overlay_dir =
                super::super::util::overlay_dir_for_scope(&engine, module_id_str, effective_scope);

            let skeleton = match kind {
                super::super::args::OverlayEditKind::Patch => {
                    if *materialize {
                        return Err(anyhow::Error::new(
                            UserError::new(
                                "E_CONFIG_INVALID",
                                "`overlay edit --kind patch` is not compatible with --materialize"
                                    .to_string(),
                            )
                            .with_details(serde_json::json!({
                                "module_id": module_id,
                                "scope": effective_scope,
                                "hint": "drop --materialize (patch overlays do not copy upstream files)",
                            })),
                        ));
                    }
                    ensure_overlay_skeleton_sparse(
                        &engine.home,
                        &engine.repo,
                        &engine.manifest,
                        module_id_str,
                        &overlay_dir,
                    )
                    .context("ensure overlay")?
                }
                super::super::args::OverlayEditKind::Dir => {
                    if *sparse || *materialize {
                        ensure_overlay_skeleton_sparse(
                            &engine.home,
                            &engine.repo,
                            &engine.manifest,
                            module_id_str,
                            &overlay_dir,
                        )
                        .context("ensure overlay")?
                    } else {
                        ensure_overlay_skeleton(
                            &engine.home,
                            &engine.repo,
                            &engine.manifest,
                            module_id_str,
                            &overlay_dir,
                        )
                        .context("ensure overlay")?
                    }
                }
            };

            let mut did_materialize = false;
            if *materialize && matches!(kind, super::super::args::OverlayEditKind::Dir) {
                materialize_overlay_from_upstream(
                    &engine.home,
                    &engine.repo,
                    &engine.manifest,
                    module_id_str,
                    &overlay_dir,
                )
                .context("materialize overlay")?;
                did_materialize = true;
            }

            let patches_dir = if matches!(kind, super::super::args::OverlayEditKind::Patch) {
                Some(
                    ensure_patch_overlay_layout(module_id_str, &overlay_dir)
                        .context("ensure patch overlay layout")?,
                )
            } else {
                None
            };

            if let Ok(editor) = std::env::var("EDITOR") {
                if !editor.trim().is_empty() {
                    let mut cmd = std::process::Command::new(editor);
                    let editor_dir = patches_dir.as_ref().unwrap_or(&skeleton.dir);
                    let status = cmd.arg(editor_dir).status().context("launch editor")?;
                    if !status.success() {
                        anyhow::bail!("editor exited with status: {status}");
                    }
                }
            }

            if ctx.cli.json {
                let mut envelope = JsonEnvelope::ok(
                    "overlay.edit",
                    serde_json::json!({
                        "module_id": module_id,
                        "scope": effective_scope,
                        "overlay_kind": kind,
                        "overlay_dir": skeleton.dir.clone(),
                        "overlay_dir_posix": crate::paths::path_to_posix_string(&skeleton.dir),
                        "created": skeleton.created,
                        "sparse": sparse,
                        "materialized": did_materialize,
                        "patches_dir": patches_dir,
                        "project": effective_scope == OverlayScope::Project,
                        "machine_id": if matches!(effective_scope, OverlayScope::Machine) { Some(engine.machine_id.clone()) } else { None },
                        "project_id": if matches!(effective_scope, OverlayScope::Project) { Some(engine.project.project_id.clone()) } else { None },
                    }),
                )
                .with_command_meta(ctx.cli.command_id(), ctx.cli.command_path());
                envelope.warnings = warnings;
                print_json(&envelope)?;
            } else {
                for w in warnings {
                    eprintln!("Warning: {w}");
                }
                let status = if skeleton.created {
                    "Created"
                } else {
                    "Overlay already exists at"
                };
                println!("{status} {}", skeleton.dir.display());
                if *sparse {
                    println!("Note: created sparse overlay (no upstream files copied)");
                }
                if did_materialize {
                    println!("Note: materialized upstream files into overlay (missing-only)");
                }
                if let Some(dir) = patches_dir {
                    println!(
                        "Note: created patch overlay; edit patch files under {}",
                        dir.display()
                    );
                }
            }
        }
        OverlayCommands::Rebase {
            module_id,
            scope,
            sparsify,
        } => {
            if ctx.cli.json && !ctx.cli.yes && !ctx.cli.dry_run {
                return Err(UserError::confirm_required("overlay rebase"));
            }

            let engine = Engine::load(ctx.cli.repo.as_deref(), ctx.cli.machine.as_deref())?;
            let module_id_str = module_id.as_str();

            let overlay_dir =
                super::super::util::overlay_dir_for_scope(&engine, module_id_str, *scope);

            let report = rebase_overlay(
                &engine.home,
                &engine.repo,
                &engine.manifest,
                module_id_str,
                &overlay_dir,
                OverlayRebaseOptions {
                    dry_run: ctx.cli.dry_run,
                    sparsify: *sparsify,
                },
            )
            .context("rebase overlay")?;

            if !report.conflicts.is_empty() {
                return Err(anyhow::Error::new(
                    UserError::new(
                        "E_OVERLAY_REBASE_CONFLICT",
                        format!(
                            "overlay rebase produced conflicts in {} file(s)",
                            report.conflicts.len()
                        ),
                    )
                    .with_details(serde_json::json!({
                        "module_id": module_id,
                        "scope": scope,
                        "overlay_dir": overlay_dir,
                        "overlay_dir_posix": crate::paths::path_to_posix_string(&overlay_dir),
                        "dry_run": ctx.cli.dry_run,
                        "sparsify": sparsify,
                        "conflicts": report.conflicts,
                        "summary": report.summary,
                        "reason_code": "overlay_rebase_conflict",
                        "next_actions": ["resolve_overlay_conflicts", "retry_overlay_rebase"],
                    })),
                ));
            }

            if ctx.cli.json {
                let envelope = JsonEnvelope::ok(
                    "overlay.rebase",
                    serde_json::json!({
                        "module_id": module_id,
                        "scope": scope,
                        "overlay_dir": overlay_dir,
                        "overlay_dir_posix": crate::paths::path_to_posix_string(&overlay_dir),
                        "dry_run": ctx.cli.dry_run,
                        "sparsify": sparsify,
                        "report": report,
                    }),
                )
                .with_command_meta(ctx.cli.command_id(), ctx.cli.command_path());
                print_json(&envelope)?;
            } else {
                let verb = if ctx.cli.dry_run {
                    "Would rebase"
                } else {
                    "Rebased"
                };
                println!("{verb} {}", overlay_dir.display());
                println!(
                    "Summary: updated={} deleted={} skipped={} conflicts={}",
                    report.summary.updated_files,
                    report.summary.deleted_files,
                    report.summary.skipped_files,
                    report.summary.conflict_files
                );
            }
        }
        OverlayCommands::Path { module_id, scope } => {
            let engine = Engine::load(ctx.cli.repo.as_deref(), ctx.cli.machine.as_deref())?;
            let module_id_str = module_id.as_str();

            let overlay_dir =
                super::super::util::overlay_dir_for_scope(&engine, module_id_str, *scope);

            if ctx.cli.json {
                let envelope = JsonEnvelope::ok(
                    "overlay.path",
                    serde_json::json!({
                        "module_id": module_id,
                        "scope": scope,
                        "overlay_dir": overlay_dir,
                        "overlay_dir_posix": crate::paths::path_to_posix_string(&overlay_dir),
                    }),
                )
                .with_command_meta(ctx.cli.command_id(), ctx.cli.command_path());
                print_json(&envelope)?;
            } else {
                println!("{}", overlay_dir.display());
            }
        }
    }

    Ok(())
}
