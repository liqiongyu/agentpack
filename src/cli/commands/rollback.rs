use anyhow::Context as _;

use crate::output::{JsonEnvelope, print_json};

use super::Ctx;

pub(crate) fn run(ctx: &Ctx<'_>, snapshot_id: &str) -> anyhow::Result<()> {
    super::super::util::require_yes_for_json_mutation(ctx.cli, "rollback")?;

    let event = crate::apply::rollback(ctx.home, snapshot_id).context("rollback")?;
    if ctx.cli.json {
        let envelope = JsonEnvelope::ok(
            "rollback",
            serde_json::json!({
                "rolled_back_to": snapshot_id,
                "event_snapshot_id": event.id,
            }),
        );
        print_json(&envelope)?;
    } else {
        println!("Rolled back to snapshot {snapshot_id}. Event: {}", event.id);
    }

    Ok(())
}
