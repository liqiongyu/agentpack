use anyhow::Context as _;

use crate::config::{Manifest, Module, ModuleType};
use crate::output::{JsonEnvelope, print_json};
use crate::source::parse_source_spec;

use super::Ctx;

pub(crate) fn run(
    ctx: &Ctx<'_>,
    module_type: &ModuleType,
    source: &str,
    id: &Option<String>,
    tags: &[String],
    targets: &[String],
) -> anyhow::Result<()> {
    super::super::util::require_yes_for_json_mutation(ctx.cli, "add")?;

    let mut manifest = Manifest::load(&ctx.repo.manifest_path).context("load manifest")?;
    let parsed_source = parse_source_spec(source).context("parse source")?;
    let module_id = id
        .clone()
        .unwrap_or_else(|| derive_module_id(module_type, source));

    manifest.modules.push(Module {
        id: module_id.clone(),
        module_type: module_type.clone(),
        enabled: true,
        tags: tags.to_vec(),
        targets: targets.to_vec(),
        source: parsed_source,
        metadata: Default::default(),
    });

    manifest
        .save(&ctx.repo.manifest_path)
        .context("save manifest")?;

    if ctx.cli.json {
        let envelope = JsonEnvelope::ok(
            "add",
            serde_json::json!({
                "module_id": module_id,
                "manifest": ctx.repo.manifest_path.clone(),
                "manifest_posix": crate::paths::path_to_posix_string(&ctx.repo.manifest_path),
            }),
        )
        .with_command_meta(ctx.cli.command_id(), ctx.cli.command_path());
        print_json(&envelope)?;
    } else {
        println!("Added module {module_id}");
    }

    Ok(())
}

fn derive_module_id(module_type: &ModuleType, source_spec: &str) -> String {
    let prefix = match module_type {
        ModuleType::Instructions => "instructions",
        ModuleType::Skill => "skill",
        ModuleType::Prompt => "prompt",
        ModuleType::Command => "command",
    };

    let name = if let Some(path) = source_spec.strip_prefix("local:") {
        std::path::Path::new(path)
            .file_stem()
            .and_then(|s| s.to_str())
            .or_else(|| {
                std::path::Path::new(path)
                    .file_name()
                    .and_then(|s| s.to_str())
            })
            .unwrap_or("module")
            .to_string()
    } else if let Some(rest) = source_spec.strip_prefix("git:") {
        let (url, query) = rest.split_once('#').unwrap_or((rest, ""));
        if let Some(subdir) = query.split('&').find_map(|kv| {
            kv.split_once('=')
                .filter(|(k, _)| *k == "subdir")
                .map(|(_, v)| v)
        }) {
            std::path::Path::new(subdir)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("module")
                .to_string()
        } else {
            url.rsplit_once('/')
                .map(|(_, name)| name)
                .unwrap_or(url)
                .trim_end_matches(".git")
                .to_string()
        }
    } else {
        "module".to_string()
    };

    format!("{prefix}:{name}")
}
