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
        .get("cursor")
        .context("missing cursor target config")?;
    let opts = &target_cfg.options;

    let (_allow_user, allow_project) = scope_flags(&target_cfg.scope);
    let write_rules = allow_project && get_bool(opts, "write_rules", true);

    let rules_dir = engine.project.project_root.join(".cursor/rules");
    if write_rules {
        roots.push(TargetRoot {
            target: "cursor".to_string(),
            root: rules_dir.clone(),
            scan_extras: true,
        });
    }

    for m in modules
        .iter()
        .filter(|m| matches!(m.module_type, ModuleType::Instructions))
        .filter(|m| m.targets.is_empty() || m.targets.iter().any(|t| t == "cursor"))
    {
        if !write_rules {
            continue;
        }

        let (_tmp, materialized) = engine.materialize_module(m, warnings)?;
        let agents_file = materialized.join("AGENTS.md");
        let body_bytes = std::fs::read(&agents_file)
            .with_context(|| format!("read {}", agents_file.display()))?;

        let description = format!("agentpack: {}", m.id);
        let description_json =
            serde_json::to_string(&description).context("serialize cursor rule description")?;
        let header =
            format!("---\ndescription: {description_json}\nglobs: []\nalwaysApply: true\n---\n\n");

        let mut out = header.into_bytes();
        out.extend(body_bytes);
        if !out.ends_with(b"\n") {
            out.push(b'\n');
        }

        let name = format!("{}.mdc", crate::ids::module_fs_key(&m.id));
        insert_file(
            desired,
            "cursor",
            rules_dir.join(name),
            out,
            vec![m.id.clone()],
        )?;
    }

    Ok(())
}
