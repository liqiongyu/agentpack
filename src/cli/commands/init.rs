use anyhow::Context as _;

use crate::output::{JsonEnvelope, print_json};

use super::Ctx;

pub(crate) fn run(ctx: &Ctx<'_>) -> anyhow::Result<()> {
    super::super::util::require_yes_for_json_mutation(ctx.cli, "init")?;

    ctx.repo.init_repo_skeleton().context("init repo")?;
    if ctx.cli.json {
        let envelope = JsonEnvelope::ok(
            "init",
            serde_json::json!({
                "repo": ctx.repo.repo_dir,
                "repo_posix": crate::paths::path_to_posix_string(&ctx.repo.repo_dir),
            }),
        );
        print_json(&envelope)?;
    } else {
        println!(
            "Initialized agentpack repo at {}",
            ctx.repo.repo_dir.display()
        );
    }

    Ok(())
}
