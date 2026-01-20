use crate::handlers::rollback::rollback;
use crate::output::{JsonEnvelope, print_json};

use super::Ctx;

pub(crate) fn run(ctx: &Ctx<'_>, snapshot_id: &str) -> anyhow::Result<()> {
    let event = rollback(ctx.home, snapshot_id, ctx.cli.json, ctx.cli.yes)?;
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
