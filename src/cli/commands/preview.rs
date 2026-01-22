use crate::app::preview_json::preview_json_data;
use crate::handlers::read_only::read_only_context;
use crate::output::{JsonEnvelope, print_json};

use super::Ctx;

pub(crate) fn run(ctx: &Ctx<'_>, diff: bool) -> anyhow::Result<()> {
    let crate::handlers::read_only::ReadOnlyContext {
        targets,
        desired,
        plan,
        mut warnings,
        roots,
    } = read_only_context(
        ctx.cli.repo.as_deref(),
        ctx.cli.machine.as_deref(),
        &ctx.cli.profile,
        &ctx.cli.target,
    )?;

    if ctx.cli.json {
        let data = preview_json_data(
            ctx.cli.profile.as_str(),
            targets,
            plan,
            desired,
            roots,
            diff,
            &mut warnings,
        )?;

        let mut envelope = JsonEnvelope::ok("preview", data);
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
        if diff {
            super::super::util::print_diff(&plan, &desired)?;
        } else {
            for c in &plan.changes {
                println!("{:?} {} {}", c.op, c.target, c.path);
            }
        }
    }

    Ok(())
}
