use anyhow::Context as _;

use crate::output::{JsonEnvelope, print_json};
use crate::user_error::UserError;

use super::super::args::PolicyCommands;
use super::Ctx;

pub(crate) fn run(ctx: &Ctx<'_>, command: &PolicyCommands) -> anyhow::Result<()> {
    match command {
        PolicyCommands::Lint => lint(ctx),
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
