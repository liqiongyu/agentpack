use crate::output::{JsonEnvelope, print_json};

use super::Ctx;

pub(crate) fn run(ctx: &Ctx<'_>) -> anyhow::Result<()> {
    super::super::util::require_yes_for_json_mutation(ctx.cli, "record")?;

    let event = crate::events::read_stdin_event()?;
    let machine_id = crate::machine::detect_machine_id()?;
    let record = crate::events::new_record(machine_id.clone(), event)?;
    let path = crate::events::append_event(ctx.home, &record)?;

    if ctx.cli.json {
        let envelope = JsonEnvelope::ok(
            "record",
            serde_json::json!({
                "path": path,
                "path_posix": crate::paths::path_to_posix_string(&path),
                "recorded_at": record.recorded_at,
                "machine_id": record.machine_id,
            }),
        )
        .with_command_meta(ctx.cli.command_id(), ctx.cli.command_path());
        print_json(&envelope)?;
    } else {
        println!("Recorded event to {}", path.display());
    }

    Ok(())
}
