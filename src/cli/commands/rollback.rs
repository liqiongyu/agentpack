use crate::app::rollback_json::rollback_json_data;
use crate::handlers::rollback::rollback;
use crate::output::{JsonEnvelope, print_json};

use super::Ctx;

pub(crate) fn run(ctx: &Ctx<'_>, snapshot_id: &str) -> anyhow::Result<()> {
    let event = rollback(ctx.home, snapshot_id, ctx.cli.json, ctx.cli.yes)?;
    if ctx.cli.json {
        let data = rollback_json_data(snapshot_id, &event.id);
        let envelope = JsonEnvelope::ok("rollback", data)
            .with_command_meta(ctx.cli.command_id(), ctx.cli.command_path());
        print_json(&envelope)?;
    } else {
        println!("Rolled back to snapshot {snapshot_id}. Event: {}", event.id);
    }

    Ok(())
}
