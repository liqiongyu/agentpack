use crate::app::preview_diff::preview_diff_files;
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
        let plan_changes = plan.changes.clone();
        let plan_summary = plan.summary.clone();
        let mut data = serde_json::json!({
            "profile": ctx.cli.profile,
            "targets": targets,
            "plan": {
                "changes": plan_changes,
                "summary": plan_summary,
            },
        });
        if diff {
            let files = preview_diff_files(&plan, &desired, &roots, &mut warnings)?;
            data["diff"] = serde_json::json!({
                "changes": plan.changes,
                "summary": plan.summary,
                "files": files,
            });
        }

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
