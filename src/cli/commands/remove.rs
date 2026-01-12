use anyhow::Context as _;

use crate::config::Manifest;
use crate::output::{JsonEnvelope, print_json};

use super::Ctx;

pub(crate) fn run(ctx: &Ctx<'_>, module_id: &str) -> anyhow::Result<()> {
    super::super::util::require_yes_for_json_mutation(ctx.cli, "remove")?;

    let mut manifest = Manifest::load(&ctx.repo.manifest_path).context("load manifest")?;
    let before = manifest.modules.len();
    manifest.modules.retain(|m| m.id != module_id);
    if manifest.modules.len() == before {
        anyhow::bail!("module not found: {module_id}");
    }
    manifest
        .save(&ctx.repo.manifest_path)
        .context("save manifest")?;

    if ctx.cli.json {
        let envelope = JsonEnvelope::ok(
            "remove",
            serde_json::json!({ "module_id": module_id, "manifest": ctx.repo.manifest_path.clone() }),
        );
        print_json(&envelope)?;
    } else {
        println!("Removed module {module_id}");
    }

    Ok(())
}
