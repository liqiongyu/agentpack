use anyhow::Context as _;

use crate::lockfile::{Lockfile, hash_tree};
use crate::output::{JsonEnvelope, print_json};
use crate::store::Store;

use super::Ctx;

pub(crate) fn run(ctx: &Ctx<'_>) -> anyhow::Result<()> {
    super::super::util::require_yes_for_json_mutation(ctx.cli, "fetch")?;

    let lock = Lockfile::load(&ctx.repo.lockfile_path).context("load lockfile")?;
    let store = Store::new(ctx.home);
    store.ensure_layout()?;

    let mut fetched = 0usize;
    for m in &lock.modules {
        let Some(gs) = &m.resolved_source.git else {
            continue;
        };

        let src = crate::config::GitSource {
            url: gs.url.clone(),
            ref_name: gs.commit.clone(),
            subdir: gs.subdir.clone(),
            shallow: false,
        };
        let checkout = store.ensure_git_checkout(&m.id, &src, &gs.commit)?;
        let root = Store::module_root_in_checkout(&checkout, &gs.subdir);
        let (_files, hash) = hash_tree(&root)?;
        if hash != m.sha256 {
            anyhow::bail!(
                "store content hash mismatch for {}: expected {}, got {}",
                m.id,
                m.sha256,
                hash
            );
        }
        fetched += 1;
    }

    if ctx.cli.json {
        let envelope = JsonEnvelope::ok(
            "fetch",
            serde_json::json!({
                "store": ctx.home.cache_dir.clone(),
                "git_modules_fetched": fetched,
            }),
        );
        print_json(&envelope)?;
    } else {
        println!(
            "Fetched/verified {fetched} git module(s) into {}",
            ctx.home.cache_dir.display()
        );
    }

    Ok(())
}
