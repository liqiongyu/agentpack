use anyhow::Context as _;

use crate::config::{Module, ModuleType};
use crate::deploy::DesiredState;
use crate::engine::Engine;

use super::TargetRoot;
use super::util::{first_file, get_bool, insert_file, scope_flags};

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
        .get("vscode")
        .context("missing vscode target config")?;
    let opts = &target_cfg.options;

    let (_allow_user, allow_project) = scope_flags(&target_cfg.scope);
    let write_instructions = allow_project && get_bool(opts, "write_instructions", true);
    let write_prompts = allow_project && get_bool(opts, "write_prompts", true);

    let github_dir = engine.project.project_root.join(".github");
    let prompts_dir = github_dir.join("prompts");

    if write_instructions {
        roots.push(TargetRoot {
            target: "vscode".to_string(),
            root: github_dir.clone(),
            scan_extras: false,
        });
    }
    if write_prompts {
        roots.push(TargetRoot {
            target: "vscode".to_string(),
            root: prompts_dir.clone(),
            scan_extras: true,
        });
    }

    let mut instructions_parts: Vec<(String, String)> = Vec::new();
    for m in modules
        .iter()
        .filter(|m| matches!(m.module_type, ModuleType::Instructions))
        .filter(|m| m.targets.is_empty() || m.targets.iter().any(|t| t == "vscode"))
    {
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

    if write_instructions && !instructions_parts.is_empty() {
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
            "vscode",
            github_dir.join("copilot-instructions.md"),
            combined.into_bytes(),
            module_ids,
        )?;
    }

    for m in modules
        .iter()
        .filter(|m| matches!(m.module_type, ModuleType::Prompt))
        .filter(|m| m.targets.is_empty() || m.targets.iter().any(|t| t == "vscode"))
    {
        if !write_prompts {
            continue;
        }
        let (_tmp, materialized) = engine.materialize_module(m, warnings)?;
        let prompt_file = first_file(&materialized)?;
        let name = prompt_file
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("prompt.md");

        let name = if name.ends_with(".prompt.md") {
            name.to_string()
        } else if let Some(stem) = name.strip_suffix(".md") {
            format!("{stem}.prompt.md")
        } else {
            format!("{name}.prompt.md")
        };

        let bytes = std::fs::read(&prompt_file)?;
        insert_file(
            desired,
            "vscode",
            prompts_dir.join(name),
            bytes,
            vec![m.id.clone()],
        )?;
    }

    Ok(())
}
