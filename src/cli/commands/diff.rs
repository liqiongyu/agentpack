use crate::handlers::read_only::read_only_context;
use crate::output::{JsonEnvelope, print_json};

use super::Ctx;

pub(crate) fn run(ctx: &Ctx<'_>) -> anyhow::Result<()> {
    let crate::handlers::read_only::ReadOnlyContext {
        targets,
        desired,
        plan,
        warnings,
        ..
    } = read_only_context(
        ctx.cli.repo.as_deref(),
        ctx.cli.machine.as_deref(),
        &ctx.cli.profile,
        &ctx.cli.target,
    )?;

    if ctx.cli.json {
        let mut envelope = JsonEnvelope::ok(
            "diff",
            serde_json::json!({
                "profile": ctx.cli.profile,
                "targets": targets,
                "changes": plan.changes,
                "summary": plan.summary,
            }),
        );
        envelope.warnings = warnings;
        print_json(&envelope)?;
        return Ok(());
    }

    for w in warnings {
        eprintln!("Warning: {w}");
    }
    super::super::util::print_diff(&plan, &desired)?;

    Ok(())
}
