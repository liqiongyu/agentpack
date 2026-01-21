use std::path::PathBuf;

use anyhow::Context as _;

use crate::config::{Module, ModuleType};
use crate::deploy::DesiredState;
use crate::engine::Engine;
use crate::fs::list_files;
use crate::store::sanitize_module_id;
use crate::user_error::UserError;

use super::TargetRoot;
use super::util::{
    expand_tilde, first_file, get_bool, insert_file, module_name_from_id, scope_flags,
};

fn export_root_from_options(
    engine: &Engine,
    opts: &std::collections::BTreeMap<String, serde_yaml::Value>,
) -> anyhow::Result<PathBuf> {
    let Some(serde_yaml::Value::String(root)) = opts.get("root") else {
        return Err(anyhow::Error::new(
            UserError::new(
                "E_CONFIG_INVALID",
                "export_dir target requires options.root",
            )
            .with_details(serde_json::json!({
                "target": "export_dir",
                "option": "root",
            })),
        ));
    };
    let root = root.trim();
    if root.is_empty() {
        return Err(anyhow::Error::new(
            UserError::new(
                "E_CONFIG_INVALID",
                "export_dir target requires non-empty options.root",
            )
            .with_details(serde_json::json!({
                "target": "export_dir",
                "option": "root",
            })),
        ));
    }

    let root = expand_tilde(root)?;
    if root.is_absolute() {
        return Ok(root);
    }
    Ok(engine.project.project_root.join(root))
}

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
        .get("export_dir")
        .context("missing export_dir target config")?;
    let opts = &target_cfg.options;

    let base_root = export_root_from_options(engine, opts)?;
    let scan_extras = get_bool(opts, "scan_extras", true);

    let (allow_user, allow_project) = scope_flags(&target_cfg.scope);
    let user_root = if allow_user && allow_project {
        base_root.join("user")
    } else {
        base_root.clone()
    };
    let project_root = if allow_user && allow_project {
        base_root.join("project")
    } else {
        base_root.clone()
    };

    if allow_user {
        roots.push(TargetRoot {
            target: "export_dir".to_string(),
            root: user_root.clone(),
            scan_extras,
        });
    }
    if allow_project {
        roots.push(TargetRoot {
            target: "export_dir".to_string(),
            root: project_root.clone(),
            scan_extras,
        });
    }

    let mut instructions_parts: Vec<(String, String)> = Vec::new();
    for m in modules
        .iter()
        .filter(|m| matches!(m.module_type, ModuleType::Instructions))
        .filter(|m| m.targets.is_empty() || m.targets.iter().any(|t| t == "export_dir"))
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

    if !instructions_parts.is_empty() {
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
        let bytes = combined.into_bytes();

        if allow_user {
            insert_file(
                desired,
                "export_dir",
                user_root.join("AGENTS.md"),
                bytes.clone(),
                module_ids.clone(),
            )?;
        }
        if allow_project {
            insert_file(
                desired,
                "export_dir",
                project_root.join("AGENTS.md"),
                bytes,
                module_ids,
            )?;
        }
    }

    for m in modules
        .iter()
        .filter(|m| matches!(m.module_type, ModuleType::Prompt))
        .filter(|m| m.targets.is_empty() || m.targets.iter().any(|t| t == "export_dir"))
    {
        let (_tmp, materialized) = engine.materialize_module(m, warnings)?;
        let prompt_file = first_file(&materialized)?;
        let name = prompt_file
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("prompt.md");
        let bytes = std::fs::read(&prompt_file)?;

        if allow_user {
            insert_file(
                desired,
                "export_dir",
                user_root.join("prompts").join(name),
                bytes.clone(),
                vec![m.id.clone()],
            )?;
        }
        if allow_project {
            insert_file(
                desired,
                "export_dir",
                project_root.join("prompts").join(name),
                bytes,
                vec![m.id.clone()],
            )?;
        }
    }

    for m in modules
        .iter()
        .filter(|m| matches!(m.module_type, ModuleType::Skill))
        .filter(|m| m.targets.is_empty() || m.targets.iter().any(|t| t == "export_dir"))
    {
        let (_tmp, materialized) = engine.materialize_module(m, warnings)?;
        let skill_name = module_name_from_id(&m.id).unwrap_or_else(|| sanitize_module_id(&m.id));

        let files = list_files(&materialized)?;
        for f in files {
            let rel = f
                .strip_prefix(&materialized)
                .with_context(|| format!("compute relpath for {}", f.display()))?
                .to_string_lossy()
                .replace('\\', "/");
            let bytes = std::fs::read(&f)?;

            if allow_user {
                let dst = user_root.join("skills").join(&skill_name).join(&rel);
                insert_file(
                    desired,
                    "export_dir",
                    dst,
                    bytes.clone(),
                    vec![m.id.clone()],
                )?;
            }
            if allow_project {
                let dst = project_root.join("skills").join(&skill_name).join(&rel);
                insert_file(desired, "export_dir", dst, bytes, vec![m.id.clone()])?;
            }
        }
    }

    for m in modules
        .iter()
        .filter(|m| matches!(m.module_type, ModuleType::Command))
        .filter(|m| m.targets.is_empty() || m.targets.iter().any(|t| t == "export_dir"))
    {
        let (_tmp, materialized) = engine.materialize_module(m, warnings)?;
        let command_file = first_file(&materialized)?;
        let name = command_file
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("command.md");
        let bytes = std::fs::read(&command_file)?;

        if allow_user {
            insert_file(
                desired,
                "export_dir",
                user_root.join("commands").join(name),
                bytes.clone(),
                vec![m.id.clone()],
            )?;
        }
        if allow_project {
            insert_file(
                desired,
                "export_dir",
                project_root.join("commands").join(name),
                bytes,
                vec![m.id.clone()],
            )?;
        }
    }

    Ok(())
}
