use anyhow::Context as _;

use crate::config::Manifest;
use crate::lockfile::generate_lockfile;
use crate::output::{JsonEnvelope, print_json};
use crate::store::Store;

use super::Ctx;

pub(crate) fn run(ctx: &Ctx<'_>) -> anyhow::Result<()> {
    super::super::util::require_yes_for_json_mutation(ctx.cli, "lock")?;

    let manifest = Manifest::load(&ctx.repo.manifest_path).context("load manifest")?;
    let store = Store::new(ctx.home);
    let lock = generate_lockfile(ctx.repo, &manifest, &store).context("generate lockfile")?;
    lock.save(&ctx.repo.lockfile_path)
        .context("write lockfile")?;

    if ctx.cli.json {
        let envelope = JsonEnvelope::ok(
            "lock",
            serde_json::json!({
                "lockfile": ctx.repo.lockfile_path.clone(),
                "modules": lock.modules.len(),
            }),
        );
        print_json(&envelope)?;
    } else {
        println!(
            "Wrote lockfile {} ({} modules)",
            ctx.repo.lockfile_path.display(),
            lock.modules.len()
        );
    }

    Ok(())
}
