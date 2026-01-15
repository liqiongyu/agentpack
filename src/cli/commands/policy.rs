use anyhow::Context as _;

use crate::output::{JsonEnvelope, print_json};
use crate::user_error::UserError;

use super::super::args::PolicyCommands;
use super::Ctx;

pub(crate) fn run(ctx: &Ctx<'_>, command: &PolicyCommands) -> anyhow::Result<()> {
    match command {
        PolicyCommands::Lint => lint(ctx),
        PolicyCommands::Lock => lock(ctx),
    }
}

fn lint(ctx: &Ctx<'_>) -> anyhow::Result<()> {
    let report = crate::policy::lint(&ctx.repo.repo_dir).context("policy lint")?;
    let violations = report.summary.violations;

    if ctx.cli.json {
        if violations == 0 {
            let data = serde_json::to_value(&report).context("serialize policy lint report")?;
            let envelope = JsonEnvelope::ok("policy.lint", data);
            print_json(&envelope)?;
            return Ok(());
        }

        return Err(anyhow::Error::new(
            UserError::new(
                "E_POLICY_VIOLATIONS",
                format!("policy lint found {violations} violation(s)"),
            )
            .with_details(serde_json::to_value(&report).context("serialize policy lint report")?),
        ));
    }

    if violations == 0 {
        println!(
            "policy lint: ok ({} file(s) scanned)",
            report.summary.files_scanned
        );
        return Ok(());
    }

    Err(anyhow::Error::new(
        UserError::new(
            "E_POLICY_VIOLATIONS",
            format!("policy lint found {violations} violation(s)"),
        )
        .with_details(serde_json::to_value(&report).context("serialize policy lint report")?),
    ))
}

fn lock(ctx: &Ctx<'_>) -> anyhow::Result<()> {
    super::super::util::require_yes_for_json_mutation(ctx.cli, "policy lock")?;

    let report = crate::policy_pack::lock_policy_pack(ctx.home, &ctx.repo.repo_dir)
        .context("policy lock")?;

    if ctx.cli.json {
        let envelope = JsonEnvelope::ok(
            "policy.lock",
            serde_json::json!({
                "lockfile_path": report.lockfile_path,
                "lockfile_path_posix": report.lockfile_path_posix,
                "resolved_version": report.resolved_version,
                "sha256": report.sha256,
                "files": report.files,
            }),
        );
        print_json(&envelope)?;
        return Ok(());
    }

    println!("Wrote {}", report.lockfile_path);
    Ok(())
}
