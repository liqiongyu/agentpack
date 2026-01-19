use anyhow::Context as _;

use crate::config::{Module, ModuleType};
use crate::deploy::DesiredState;
use crate::engine::Engine;

use super::TargetRoot;
use super::util::{get_bool, insert_file, scope_flags};

pub(crate) fn render(
    engine: &Engine,
    modules: &[&Module],
    desired: &mut DesiredState,
    warnings: &mut Vec<String>,
    roots: &mut Vec<TargetRoot>,
) -> anyhow::Result<()> {
    let target_cfg = engine
        .manifest
        .targets
        .get("jetbrains")
        .context("missing jetbrains target config")?;
    let opts = &target_cfg.options;

    let (_allow_user, allow_project) = scope_flags(&target_cfg.scope);
    let write_guidelines = allow_project && get_bool(opts, "write_guidelines", true);

    let junie_dir = engine.project.project_root.join(".junie");
    if write_guidelines {
        roots.push(TargetRoot {
            target: "jetbrains".to_string(),
            root: junie_dir.clone(),
            scan_extras: true,
        });
    }

    let mut instructions_parts: Vec<(String, String)> = Vec::new();
    for m in modules
        .iter()
        .filter(|m| matches!(m.module_type, ModuleType::Instructions))
        .filter(|m| m.targets.is_empty() || m.targets.iter().any(|t| t == "jetbrains"))
    {
        if !write_guidelines {
            continue;
        }

        let (_tmp, materialized) = engine.materialize_module(m, warnings)?;
        let agents_path = materialized.join("AGENTS.md");
        if agents_path.exists() {
            instructions_parts.push((
                m.id.clone(),
                std::fs::read_to_string(&agents_path)
                    .with_context(|| format!("read {}", agents_path.display()))?,
            ));
        }
    }

    if write_guidelines && !instructions_parts.is_empty() {
        let module_ids: Vec<String> = instructions_parts
            .iter()
            .map(|(id, _)| id.clone())
            .collect();
        let add_markers = instructions_parts.len() > 1;
        let combined = if add_markers {
            instructions_parts
                .into_iter()
                .map(|(module_id, text)| crate::markers::format_module_section(&module_id, &text))
                .collect::<Vec<_>>()
                .join("\n\n---\n\n")
        } else {
            instructions_parts
                .into_iter()
                .map(|(_, text)| text)
                .collect::<Vec<_>>()
                .join("\n\n---\n\n")
        };

        insert_file(
            desired,
            "jetbrains",
            junie_dir.join("guidelines.md"),
            combined.into_bytes(),
            module_ids,
        )?;
    }

    Ok(())
}
