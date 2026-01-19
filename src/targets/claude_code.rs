use anyhow::Context as _;

use crate::config::{Module, ModuleType};
use crate::deploy::DesiredState;
use crate::engine::Engine;
use crate::fs::list_files;
use crate::store::sanitize_module_id;

use super::TargetRoot;
use super::util::{
    expand_tilde, first_file, get_bool, insert_file, module_name_from_id, scope_flags,
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
        .get("claude_code")
        .context("missing claude_code target config")?;
    let opts = &target_cfg.options;

    let (allow_user, allow_project) = scope_flags(&target_cfg.scope);
    let write_repo_commands = allow_project && get_bool(opts, "write_repo_commands", true);
    let write_user_commands = allow_user && get_bool(opts, "write_user_commands", true);
    let write_repo_skills = allow_project && get_bool(opts, "write_repo_skills", false);
    let write_user_skills = allow_user && get_bool(opts, "write_user_skills", false);

    let user_commands_dir = expand_tilde("~/.claude/commands")?;
    let user_skills_dir = expand_tilde("~/.claude/skills")?;

    if write_user_commands {
        roots.push(TargetRoot {
            target: "claude_code".to_string(),
            root: user_commands_dir.clone(),
            scan_extras: true,
        });
    }
    if write_repo_commands {
        roots.push(TargetRoot {
            target: "claude_code".to_string(),
            root: engine.project.project_root.join(".claude/commands"),
            scan_extras: true,
        });
    }
    if write_user_skills {
        roots.push(TargetRoot {
            target: "claude_code".to_string(),
            root: user_skills_dir.clone(),
            scan_extras: true,
        });
    }
    if write_repo_skills {
        roots.push(TargetRoot {
            target: "claude_code".to_string(),
            root: engine.project.project_root.join(".claude/skills"),
            scan_extras: true,
        });
    }

    for m in modules
        .iter()
        .filter(|m| matches!(m.module_type, ModuleType::Command))
        .filter(|m| m.targets.is_empty() || m.targets.iter().any(|t| t == "claude_code"))
    {
        let (_tmp, materialized) = engine.materialize_module(m, warnings)?;
        let cmd_file = first_file(&materialized)?;
        let name = cmd_file
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("command.md");
        let bytes = std::fs::read(&cmd_file)?;

        if write_user_commands {
            insert_file(
                desired,
                "claude_code",
                user_commands_dir.join(name),
                bytes.clone(),
                vec![m.id.clone()],
            )?;
        }
        if write_repo_commands {
            insert_file(
                desired,
                "claude_code",
                engine
                    .project
                    .project_root
                    .join(".claude/commands")
                    .join(name),
                bytes,
                vec![m.id.clone()],
            )?;
        }
    }

    for m in modules
        .iter()
        .filter(|m| matches!(m.module_type, ModuleType::Skill))
        .filter(|m| m.targets.is_empty() || m.targets.iter().any(|t| t == "claude_code"))
    {
        if !write_user_skills && !write_repo_skills {
            continue;
        }
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
                let dst = user_skills_dir.join(&skill_name).join(&rel);
                insert_file(
                    desired,
                    "claude_code",
                    dst,
                    bytes.clone(),
                    vec![m.id.clone()],
                )?;
            }
            if write_repo_skills {
                let dst = engine
                    .project
                    .project_root
                    .join(".claude/skills")
                    .join(&skill_name)
                    .join(&rel);
                insert_file(desired, "claude_code", dst, bytes, vec![m.id.clone()])?;
            }
        }
    }

    Ok(())
}
