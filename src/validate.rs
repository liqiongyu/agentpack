use std::path::{Path, PathBuf};

use anyhow::Context as _;

use crate::config::ModuleType;
use crate::fs::list_files;
use crate::user_error::UserError;

pub fn validate_materialized_module(
    module_type: &ModuleType,
    module_id: &str,
    materialized_root: &Path,
) -> anyhow::Result<()> {
    match module_type {
        ModuleType::Instructions => {
            let agents = materialized_root.join("AGENTS.md");
            if !agents.is_file() {
                anyhow::bail!("instructions module {module_id} is missing AGENTS.md");
            }
        }
        ModuleType::Skill => {
            let skill_md = materialized_root.join("SKILL.md");
            if !skill_md.is_file() {
                anyhow::bail!("skill module {module_id} is missing SKILL.md");
            }
            let text = std::fs::read_to_string(&skill_md)
                .with_context(|| format!("read skill module {}", skill_md.display()))?;
            validate_skill_frontmatter(module_id, &skill_md, &text)?;
        }
        ModuleType::Prompt => {
            let _file = require_single_markdown_file(materialized_root, module_id, "prompt")?;
        }
        ModuleType::Command => {
            let file = require_single_markdown_file(materialized_root, module_id, "command")?;
            let text = std::fs::read_to_string(&file)
                .with_context(|| format!("read command module {}", file.display()))?;
            validate_claude_command_frontmatter(module_id, &text)?;
        }
    }

    Ok(())
}

fn validate_skill_frontmatter(
    module_id: &str,
    skill_md: &Path,
    markdown: &str,
) -> anyhow::Result<()> {
    let frontmatter = match extract_yaml_frontmatter(markdown) {
        Ok(Some(v)) => v,
        Ok(None) => {
            return Err(anyhow::Error::new(
                UserError::new(
                    "E_CONFIG_INVALID",
                    format!("skill module {module_id} is missing YAML frontmatter in SKILL.md"),
                )
                .with_details(serde_json::json!({
                    "module_id": module_id,
                    "path": skill_md.to_string_lossy(),
                    "missing": ["frontmatter"],
                    "required_fields": ["name","description"],
                })),
            ));
        }
        Err(err) => {
            return Err(anyhow::Error::new(
                UserError::new(
                    "E_CONFIG_INVALID",
                    format!("skill module {module_id} has invalid YAML frontmatter in SKILL.md"),
                )
                .with_details(serde_json::json!({
                    "module_id": module_id,
                    "path": skill_md.to_string_lossy(),
                    "error": err.to_string(),
                })),
            ));
        }
    };

    let map = frontmatter.as_mapping().ok_or_else(|| {
        anyhow::Error::new(
            UserError::new(
                "E_CONFIG_INVALID",
                format!("skill module {module_id} frontmatter must be a YAML mapping"),
            )
            .with_details(serde_json::json!({
                "module_id": module_id,
                "path": skill_md.to_string_lossy(),
                "expected": "mapping",
            })),
        )
    })?;

    require_frontmatter_string(module_id, skill_md, map, "name")?;
    require_frontmatter_string(module_id, skill_md, map, "description")?;

    Ok(())
}

fn require_frontmatter_string(
    module_id: &str,
    skill_md: &Path,
    map: &serde_yaml::Mapping,
    key: &str,
) -> anyhow::Result<()> {
    let Some(value) = yaml_get(map, key) else {
        return Err(anyhow::Error::new(
            UserError::new(
                "E_CONFIG_INVALID",
                format!("skill module {module_id} frontmatter is missing {key}"),
            )
            .with_details(serde_json::json!({
                "module_id": module_id,
                "path": skill_md.to_string_lossy(),
                "missing": [key],
            })),
        ));
    };

    let s = match value {
        serde_yaml::Value::String(s) => s.as_str(),
        _ => {
            return Err(anyhow::Error::new(
                UserError::new(
                    "E_CONFIG_INVALID",
                    format!("skill module {module_id} frontmatter {key} must be a string"),
                )
                .with_details(serde_json::json!({
                    "module_id": module_id,
                    "path": skill_md.to_string_lossy(),
                    "field": key,
                    "expected": "string",
                })),
            ));
        }
    };

    if s.trim().is_empty() {
        return Err(anyhow::Error::new(
            UserError::new(
                "E_CONFIG_INVALID",
                format!("skill module {module_id} frontmatter {key} is empty"),
            )
            .with_details(serde_json::json!({
                "module_id": module_id,
                "path": skill_md.to_string_lossy(),
                "field": key,
            })),
        ));
    }

    Ok(())
}

fn require_single_markdown_file(
    materialized_root: &Path,
    module_id: &str,
    kind: &str,
) -> anyhow::Result<PathBuf> {
    let mut files = list_files(materialized_root)?;
    files.sort();

    if files.len() != 1 {
        anyhow::bail!(
            "{kind} module {module_id} must contain exactly one file, found {}",
            files.len()
        );
    }

    let file = files.remove(0);
    if file.extension().and_then(|s| s.to_str()) != Some("md") {
        anyhow::bail!(
            "{kind} module {module_id} must be a .md file: {}",
            file.display()
        );
    }

    Ok(file)
}

fn validate_claude_command_frontmatter(module_id: &str, markdown: &str) -> anyhow::Result<()> {
    let uses_bash = uses_bash_tool(markdown);
    let frontmatter = extract_yaml_frontmatter(markdown)?;
    let Some(frontmatter) = frontmatter else {
        anyhow::bail!(
            "claude command module {module_id} is missing YAML frontmatter (--- ... ---)"
        );
    };

    let map = frontmatter.as_mapping().with_context(|| {
        format!("claude command module {module_id} frontmatter must be a YAML mapping")
    })?;

    let Some(description) = yaml_get_string(map, "description") else {
        anyhow::bail!("claude command module {module_id} frontmatter is missing description");
    };
    if description.trim().is_empty() {
        anyhow::bail!("claude command module {module_id} frontmatter description is empty");
    }

    if uses_bash {
        let Some(allowed) = yaml_get(map, "allowed-tools") else {
            anyhow::bail!(
                "claude command module {module_id} uses bash but frontmatter is missing allowed-tools"
            );
        };
        if !allowed_tools_allows_bash(allowed) {
            anyhow::bail!(
                "claude command module {module_id} uses bash but allowed-tools does not include Bash(...)"
            );
        }
    }

    Ok(())
}

fn uses_bash_tool(markdown: &str) -> bool {
    markdown.contains("!bash") || markdown.contains("!`bash`")
}

fn extract_yaml_frontmatter(markdown: &str) -> anyhow::Result<Option<serde_yaml::Value>> {
    let mut lines = markdown.lines();
    let first = lines.next().unwrap_or("").trim_end_matches('\r');
    if first != "---" {
        return Ok(None);
    }

    let mut fm = Vec::new();
    let mut found_end = false;
    for line in lines {
        let line = line.trim_end_matches('\r');
        if line == "---" {
            found_end = true;
            break;
        }
        fm.push(line);
    }

    if !found_end {
        anyhow::bail!("unterminated YAML frontmatter (missing closing ---)");
    }

    let value: serde_yaml::Value =
        serde_yaml::from_str(&fm.join("\n")).context("parse YAML frontmatter")?;
    Ok(Some(value))
}

fn yaml_get<'a>(map: &'a serde_yaml::Mapping, key: &str) -> Option<&'a serde_yaml::Value> {
    map.iter().find_map(|(k, v)| match k {
        serde_yaml::Value::String(s) if s == key => Some(v),
        _ => None,
    })
}

fn yaml_get_string<'a>(map: &'a serde_yaml::Mapping, key: &str) -> Option<&'a str> {
    match yaml_get(map, key) {
        Some(serde_yaml::Value::String(s)) => Some(s.as_str()),
        _ => None,
    }
}

fn allowed_tools_allows_bash(allowed: &serde_yaml::Value) -> bool {
    match allowed {
        serde_yaml::Value::String(s) => s.contains("Bash("),
        serde_yaml::Value::Sequence(items) => items.iter().any(|v| match v {
            serde_yaml::Value::String(s) => s.contains("Bash("),
            _ => false,
        }),
        _ => false,
    }
}
