use anyhow::Context as _;

use crate::config::TargetScope;

pub(crate) fn extract_agentpack_version(text: &str) -> Option<String> {
    for line in text.lines() {
        if let Some((_, rest)) = line.split_once("agentpack_version:") {
            let mut value = rest.trim();
            value = value.trim_end_matches("-->");
            value = value.trim();
            value = value.trim_matches(|c| c == '"' || c == '\'');
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }
    None
}

pub(crate) fn check_operator_file(
    path: &std::path::Path,
    location: &str,
    current: &str,
    warnings: &mut Vec<String>,
    suggested: &str,
    record_next_action: &mut impl FnMut(&str),
) -> anyhow::Result<()> {
    if !path.exists() {
        warnings.push(format!(
            "operator assets missing ({location}): {}; run: {suggested}",
            path.display()
        ));
        record_next_action(suggested);
        return Ok(());
    }

    let text = std::fs::read_to_string(path)
        .with_context(|| format!("read operator asset {}", path.display()))?;
    let Some(have) = extract_agentpack_version(&text) else {
        warnings.push(format!(
            "operator assets missing agentpack_version ({location}): {}; run: {suggested}",
            path.display()
        ));
        record_next_action(suggested);
        return Ok(());
    };

    if have != current {
        warnings.push(format!(
            "operator assets outdated ({location}): {} has {}, want {}; run: {suggested}",
            path.display(),
            have,
            current
        ));
        record_next_action(suggested);
    }

    Ok(())
}

pub(crate) fn check_operator_command_dir(
    dir: &std::path::Path,
    location: &str,
    current: &str,
    warnings: &mut Vec<String>,
    suggested: &str,
    record_next_action: &mut impl FnMut(&str),
) -> anyhow::Result<()> {
    if !dir.exists() {
        warnings.push(format!(
            "operator assets missing ({location}): {}; run: {suggested}",
            dir.display()
        ));
        record_next_action(suggested);
        return Ok(());
    }

    let mut missing = Vec::new();
    let mut missing_version = None;
    for name in [
        "ap-doctor.md",
        "ap-update.md",
        "ap-preview.md",
        "ap-plan.md",
        "ap-deploy.md",
        "ap-status.md",
        "ap-diff.md",
        "ap-explain.md",
        "ap-evolve.md",
    ] {
        let path = dir.join(name);
        if !path.exists() {
            missing.push(name.to_string());
            continue;
        }
        let text = std::fs::read_to_string(&path)
            .with_context(|| format!("read operator asset {}", path.display()))?;
        let Some(have) = extract_agentpack_version(&text) else {
            missing_version = Some(path);
            continue;
        };
        if have != current {
            warnings.push(format!(
                "operator assets outdated ({location}): {} has {}, want {}; run: {suggested}",
                path.display(),
                have,
                current
            ));
        }
    }

    if let Some(path) = missing_version {
        warnings.push(format!(
            "operator assets missing agentpack_version ({location}): {}; run: {suggested}",
            path.display()
        ));
        record_next_action(suggested);
        return Ok(());
    }

    if !missing.is_empty() {
        warnings.push(format!(
            "operator assets incomplete ({location}): missing {}; run: {suggested}",
            missing.join(", "),
        ));
        record_next_action(suggested);
    }

    Ok(())
}

fn target_scope_flags(scope: &TargetScope) -> (bool, bool) {
    match scope {
        TargetScope::User => (true, false),
        TargetScope::Project => (false, true),
        TargetScope::Both => (true, true),
    }
}

fn get_bool(
    map: &std::collections::BTreeMap<String, serde_yaml::Value>,
    key: &str,
    default: bool,
) -> bool {
    match map.get(key) {
        Some(serde_yaml::Value::Bool(b)) => *b,
        Some(serde_yaml::Value::String(s)) => match s.trim().to_ascii_lowercase().as_str() {
            "true" | "yes" | "1" => true,
            "false" | "no" | "0" => false,
            _ => default,
        },
        _ => default,
    }
}

pub(crate) struct OperatorAssetsStatusPaths<'a> {
    pub codex_home: &'a std::path::Path,
    pub claude_user_commands_dir: &'a std::path::Path,
    pub claude_user_skills_dir: &'a std::path::Path,
}

pub(crate) fn warn_operator_assets_if_outdated_for_status(
    engine: &crate::engine::Engine,
    targets: &[String],
    paths: OperatorAssetsStatusPaths<'_>,
    warnings: &mut Vec<String>,
    bootstrap_action: &mut impl FnMut(&str, &str) -> String,
    record_next_action: &mut impl FnMut(&str),
) -> anyhow::Result<()> {
    let current = env!("CARGO_PKG_VERSION");

    for target in targets {
        match target.as_str() {
            "codex" => {
                let Some(cfg) = engine.manifest.targets.get("codex") else {
                    continue;
                };
                let (allow_user, allow_project) = target_scope_flags(&cfg.scope);

                if allow_user {
                    let path = paths.codex_home.join("skills/agentpack-operator/SKILL.md");
                    let suggested = bootstrap_action("codex", "user");
                    check_operator_file(
                        &path,
                        "codex/user",
                        current,
                        warnings,
                        &suggested,
                        record_next_action,
                    )?;
                }
                if allow_project {
                    let path = engine
                        .project
                        .project_root
                        .join(".codex/skills/agentpack-operator/SKILL.md");
                    let suggested = bootstrap_action("codex", "project");
                    check_operator_file(
                        &path,
                        "codex/project",
                        current,
                        warnings,
                        &suggested,
                        record_next_action,
                    )?;
                }
            }
            "claude_code" => {
                let Some(cfg) = engine.manifest.targets.get("claude_code") else {
                    continue;
                };
                let (allow_user, allow_project) = target_scope_flags(&cfg.scope);
                let check_user_skills =
                    allow_user && get_bool(&cfg.options, "write_user_skills", false);
                let check_repo_skills =
                    allow_project && get_bool(&cfg.options, "write_repo_skills", false);

                if allow_user {
                    let suggested = bootstrap_action("claude_code", "user");
                    check_operator_command_dir(
                        paths.claude_user_commands_dir,
                        "claude_code/user",
                        current,
                        warnings,
                        &suggested,
                        record_next_action,
                    )?;
                    if check_user_skills {
                        check_operator_file(
                            &paths
                                .claude_user_skills_dir
                                .join("agentpack-operator/SKILL.md"),
                            "claude_code/user",
                            current,
                            warnings,
                            &suggested,
                            record_next_action,
                        )?;
                    }
                }
                if allow_project {
                    let suggested = bootstrap_action("claude_code", "project");
                    check_operator_command_dir(
                        &engine.project.project_root.join(".claude/commands"),
                        "claude_code/project",
                        current,
                        warnings,
                        &suggested,
                        record_next_action,
                    )?;
                    if check_repo_skills {
                        check_operator_file(
                            &engine
                                .project
                                .project_root
                                .join(".claude/skills/agentpack-operator/SKILL.md"),
                            "claude_code/project",
                            current,
                            warnings,
                            &suggested,
                            record_next_action,
                        )?;
                    }
                }
            }
            _ => {}
        }
    }

    Ok(())
}
