use anyhow::Context as _;

use crate::output::{JsonEnvelope, print_json};
use crate::user_error::UserError;

use super::super::args::PolicyCommands;
use super::Ctx;

pub(crate) fn run(ctx: &Ctx<'_>, command: &PolicyCommands) -> anyhow::Result<()> {
    match command {
        PolicyCommands::Lint => lint(ctx),
        PolicyCommands::Audit => audit(ctx),
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

fn audit(ctx: &Ctx<'_>) -> anyhow::Result<()> {
    let outcome = crate::policy::audit(&ctx.repo.repo_dir).context("policy audit")?;

    if ctx.cli.json {
        let data =
            serde_json::to_value(&outcome.report).context("serialize policy audit report")?;
        let mut envelope = JsonEnvelope::ok("policy.audit", data);
        envelope.warnings = outcome.warnings;
        print_json(&envelope)?;
        return Ok(());
    }

    for w in &outcome.warnings {
        eprintln!("Warning: {w}");
    }

    println!(
        "policy audit: {} module(s) (lockfile: {})",
        outcome.report.modules.len(),
        outcome.report.lockfile.lockfile_path
    );
    for m in &outcome.report.modules {
        let source = if let Some(git) = m.resolved_source.git.as_ref() {
            format!(
                "git {}@{}{}",
                git.url,
                git.commit,
                format_git_subdir(&git.subdir)
            )
        } else if let Some(lp) = m.resolved_source.local_path.as_ref() {
            format!("local {}", lp.path)
        } else {
            "unknown".to_string()
        };
        println!(
            "- {} ({:?}): {} (sha256={}, files={}, bytes={})",
            m.id, m.module_type, source, m.sha256, m.files, m.bytes
        );
    }

    if let Some(org) = outcome.report.org_policy_pack.as_ref() {
        println!(
            "policy pack: {} (sha256={}, files={}, bytes={})",
            org.resolved_version, org.sha256, org.files, org.bytes
        );
    }

    Ok(())
}

fn format_git_subdir(subdir: &str) -> String {
    let s = subdir.trim();
    if s.is_empty() {
        return String::new();
    }
    format!(" ({s})")
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
