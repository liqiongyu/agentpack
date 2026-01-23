use crate::app::plan_json::plan_json_data;
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
        let data = plan_json_data(ctx.cli.profile.as_str(), targets, plan);
        let mut envelope = JsonEnvelope::ok("diff", data)
            .with_command_meta(ctx.cli.command_id(), ctx.cli.command_path());
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
