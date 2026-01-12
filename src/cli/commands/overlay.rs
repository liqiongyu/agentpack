use anyhow::Context as _;

use crate::engine::Engine;
use crate::output::{JsonEnvelope, print_json};
use crate::overlay::ensure_overlay_skeleton;

use super::super::args::{OverlayCommands, OverlayScope};
use super::Ctx;

pub(crate) fn run(ctx: &Ctx<'_>, command: &OverlayCommands) -> anyhow::Result<()> {
    match command {
        OverlayCommands::Edit {
            module_id,
            scope,
            project,
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

            let skeleton = ensure_overlay_skeleton(
                &engine.home,
                &engine.repo,
                &engine.manifest,
                module_id_str,
                &overlay_dir,
            )
            .context("ensure overlay")?;

            if let Ok(editor) = std::env::var("EDITOR") {
                if !editor.trim().is_empty() {
                    let mut cmd = std::process::Command::new(editor);
                    let status = cmd.arg(&skeleton.dir).status().context("launch editor")?;
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
                        "overlay_dir": skeleton.dir.clone(),
                        "overlay_dir_posix": crate::paths::path_to_posix_string(&skeleton.dir),
                        "created": skeleton.created,
                        "project": effective_scope == OverlayScope::Project,
                        "machine_id": if matches!(effective_scope, OverlayScope::Machine) { Some(engine.machine_id.clone()) } else { None },
                        "project_id": if matches!(effective_scope, OverlayScope::Project) { Some(engine.project.project_id.clone()) } else { None },
                    }),
                );
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
                );
                print_json(&envelope)?;
            } else {
                println!("{}", overlay_dir.display());
            }
        }
    }

    Ok(())
}
