use anyhow::Context as _;

use crate::config::Manifest;
use crate::lockfile::{Lockfile, generate_lockfile, hash_tree};
use crate::output::{JsonEnvelope, print_json};
use crate::store::Store;
use crate::user_error::UserError;

use super::Ctx;

pub(crate) fn run(
    ctx: &Ctx<'_>,
    lock: bool,
    fetch: bool,
    no_lock: bool,
    no_fetch: bool,
) -> anyhow::Result<()> {
    #[derive(Debug, Clone, serde::Serialize)]
    struct UpdateStep {
        name: String,
        ok: bool,
        detail: serde_json::Value,
    }

    let lockfile_exists = ctx.repo.lockfile_path.exists();
    let mut do_lock = !lockfile_exists;
    let mut do_fetch = true;

    if lock {
        do_lock = true;
    }
    if fetch {
        do_fetch = true;
    }
    if no_lock {
        do_lock = false;
    }
    if no_fetch {
        do_fetch = false;
    }

    if do_fetch && !do_lock && !lockfile_exists {
        anyhow::bail!(
            "lockfile missing: {}; run `agentpack lock` first or omit --no-lock",
            ctx.repo.lockfile_path.display()
        );
    }

    let will_write = do_lock || do_fetch;
    if ctx.cli.json && will_write && !ctx.cli.yes {
        return Err(UserError::confirm_required("update"));
    }

    let mut steps: Vec<UpdateStep> = Vec::new();
    let store = Store::new(ctx.home);

    let lock = if do_lock {
        let manifest = Manifest::load(&ctx.repo.manifest_path).context("load manifest")?;
        let lock = generate_lockfile(ctx.repo, &manifest, &store).context("generate lockfile")?;
        lock.save(&ctx.repo.lockfile_path)
            .context("write lockfile")?;
        steps.push(UpdateStep {
            name: "lock".to_string(),
            ok: true,
            detail: serde_json::json!({
                "lockfile": ctx.repo.lockfile_path.clone(),
                "lockfile_posix": crate::paths::path_to_posix_string(&ctx.repo.lockfile_path),
                "modules": lock.modules.len(),
            }),
        });
        Some(lock)
    } else {
        None
    };

    let mut fetched = 0usize;
    if do_fetch {
        let lock = match lock {
            Some(l) => l,
            None => Lockfile::load(&ctx.repo.lockfile_path).context("load lockfile")?,
        };
        store.ensure_layout()?;

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

        steps.push(UpdateStep {
            name: "fetch".to_string(),
            ok: true,
            detail: serde_json::json!({
                "store": ctx.home.cache_dir.clone(),
                "store_posix": crate::paths::path_to_posix_string(&ctx.home.cache_dir),
                "git_modules_fetched": fetched,
            }),
        });
    }

    if ctx.cli.json {
        let envelope = JsonEnvelope::ok(
            "update",
            serde_json::json!({
                "lockfile": ctx.repo.lockfile_path.clone(),
                "lockfile_posix": crate::paths::path_to_posix_string(&ctx.repo.lockfile_path),
                "store": ctx.home.cache_dir.clone(),
                "store_posix": crate::paths::path_to_posix_string(&ctx.home.cache_dir),
                "steps": steps,
                "git_modules_fetched": fetched,
            }),
        )
        .with_command_meta(ctx.cli.command_id(), ctx.cli.command_path());
        print_json(&envelope)?;
    } else if steps.is_empty() {
        println!("No steps to run");
    } else {
        for s in &steps {
            println!("- {}", s.name);
        }
    }

    Ok(())
}
