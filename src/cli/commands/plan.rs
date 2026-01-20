use crate::handlers::read_only::read_only_context;
use crate::output::{JsonEnvelope, print_json};

use super::Ctx;

pub(crate) fn run(ctx: &Ctx<'_>) -> anyhow::Result<()> {
    let crate::handlers::read_only::ReadOnlyContext {
        targets,
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
            "plan",
            serde_json::json!({
                "profile": ctx.cli.profile,
                "targets": targets,
                "changes": plan.changes,
                "summary": plan.summary,
            }),
        );
        envelope.warnings = warnings;
        print_json(&envelope)?;
    } else {
        for w in warnings {
            eprintln!("Warning: {w}");
        }
        println!(
            "Plan: +{} ~{} -{}",
            plan.summary.create, plan.summary.update, plan.summary.delete
        );
        for c in &plan.changes {
            println!("{:?} {} {}", c.op, c.target, c.path);
        }
    }

    Ok(())
}
