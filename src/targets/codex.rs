use anyhow::Context as _;

use crate::config::{Module, ModuleType};
use crate::deploy::DesiredState;
use crate::engine::Engine;
use crate::fs::list_files;
use crate::store::sanitize_module_id;

use super::TargetRoot;
use super::util::{
    codex_home_from_options, first_file, get_bool, insert_file, module_name_from_id, scope_flags,
};

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
        .get("codex")
        .context("missing codex target config")?;
    let opts = &target_cfg.options;

    let codex_home = codex_home_from_options(opts)?;
    let (allow_user, allow_project) = scope_flags(&target_cfg.scope);
    let write_repo_skills = allow_project && get_bool(opts, "write_repo_skills", true);
    let write_user_skills = allow_user && get_bool(opts, "write_user_skills", true);
    let write_user_prompts = allow_user && get_bool(opts, "write_user_prompts", true);
    let write_agents_global = allow_user && get_bool(opts, "write_agents_global", true);
    let write_agents_repo_root = allow_project && get_bool(opts, "write_agents_repo_root", true);

    if write_agents_global {
        roots.push(TargetRoot {
            target: "codex".to_string(),
            root: codex_home.clone(),
            scan_extras: false,
        });
    }
    if write_user_prompts {
        roots.push(TargetRoot {
            target: "codex".to_string(),
            root: codex_home.join("prompts"),
            scan_extras: true,
        });
    }
    if write_user_skills {
        roots.push(TargetRoot {
            target: "codex".to_string(),
            root: codex_home.join("skills"),
            scan_extras: true,
        });
    }
    if write_agents_repo_root {
        roots.push(TargetRoot {
            target: "codex".to_string(),
            root: engine.project.project_root.clone(),
            scan_extras: false,
        });
    }
    if write_repo_skills {
        roots.push(TargetRoot {
            target: "codex".to_string(),
            root: engine.project.project_root.join(".codex/skills"),
            scan_extras: true,
        });
    }

    let mut instructions_parts: Vec<(String, String)> = Vec::new();
    for m in modules
        .iter()
        .filter(|m| matches!(m.module_type, ModuleType::Instructions))
        .filter(|m| m.targets.is_empty() || m.targets.iter().any(|t| t == "codex"))
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

        if write_agents_global {
            insert_file(
                desired,
                "codex",
                codex_home.join("AGENTS.md"),
                bytes.clone(),
                module_ids.clone(),
            )?;
        }
        if write_agents_repo_root {
            insert_file(
                desired,
                "codex",
                engine.project.project_root.join("AGENTS.md"),
                bytes,
                module_ids,
            )?;
        }
    }

    for m in modules
        .iter()
        .filter(|m| matches!(m.module_type, ModuleType::Prompt))
        .filter(|m| m.targets.is_empty() || m.targets.iter().any(|t| t == "codex"))
    {
        if !write_user_prompts {
            continue;
        }
        let (_tmp, materialized) = engine.materialize_module(m, warnings)?;
        let prompt_file = first_file(&materialized)?;
        let name = prompt_file
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("prompt.md");
        let bytes = std::fs::read(&prompt_file)?;
        insert_file(
            desired,
            "codex",
            codex_home.join("prompts").join(name),
            bytes,
            vec![m.id.clone()],
        )?;
    }

    for m in modules
        .iter()
        .filter(|m| matches!(m.module_type, ModuleType::Skill))
        .filter(|m| m.targets.is_empty() || m.targets.iter().any(|t| t == "codex"))
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

            if write_user_skills {
                let dst = codex_home.join("skills").join(&skill_name).join(&rel);
                insert_file(desired, "codex", dst, bytes.clone(), vec![m.id.clone()])?;
            }
            if write_repo_skills {
                let dst = engine
                    .project
                    .project_root
                    .join(".codex/skills")
                    .join(&skill_name)
                    .join(&rel);
                insert_file(desired, "codex", dst, bytes, vec![m.id.clone()])?;
            }
        }
    }

    Ok(())
}
