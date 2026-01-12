use clap::CommandFactory as _;

use crate::output::{JsonEnvelope, print_json};

use super::Ctx;

pub(crate) fn run(ctx: &Ctx<'_>) -> anyhow::Result<()> {
    if ctx.cli.json {
        let commands = serde_json::json!([
            {"id":"init","path":["init"],"mutating":true},
            {"id":"help","path":["help"],"mutating":false},
            {"id":"schema","path":["schema"],"mutating":false},
            {"id":"update","path":["update"],"mutating":true},
            {"id":"add","path":["add"],"mutating":true},
            {"id":"remove","path":["remove"],"mutating":true},
            {"id":"lock","path":["lock"],"mutating":true},
            {"id":"fetch","path":["fetch"],"mutating":true},
            {"id":"preview","path":["preview"],"mutating":false},
            {"id":"plan","path":["plan"],"mutating":false},
            {"id":"diff","path":["diff"],"mutating":false},
            {"id":"deploy","path":["deploy"],"mutating":false},
            {"id":"status","path":["status"],"mutating":false},
            {"id":"doctor","path":["doctor"],"mutating":false},
            {"id":"doctor --fix","path":["doctor"],"mutating":true},
            {"id":"remote set","path":["remote","set"],"mutating":true},
            {"id":"sync","path":["sync"],"mutating":true},
            {"id":"record","path":["record"],"mutating":true},
            {"id":"score","path":["score"],"mutating":false},
            {"id":"explain plan","path":["explain","plan"],"mutating":false},
            {"id":"explain diff","path":["explain","diff"],"mutating":false},
            {"id":"explain status","path":["explain","status"],"mutating":false},
            {"id":"evolve propose","path":["evolve","propose"],"mutating":true},
            {"id":"completions","path":["completions"],"mutating":false},
            {"id":"rollback","path":["rollback"],"mutating":true},
            {"id":"bootstrap","path":["bootstrap"],"mutating":true},
            {"id":"overlay edit","path":["overlay","edit"],"mutating":true},
            {"id":"overlay path","path":["overlay","path"],"mutating":false}
        ]);

        let envelope = JsonEnvelope::ok(
            "help",
            serde_json::json!({
                "commands": commands,
                "mutating_commands": super::super::util::MUTATING_COMMAND_IDS,
                "notes": [
                    "recommended: doctor -> update -> preview -> deploy --apply",
                    "recommended: status -> evolve propose -> review -> deploy --apply",
                    "in --json mode, mutating commands require --yes"
                ]
            }),
        );
        print_json(&envelope)?;
    } else {
        let mut cmd = super::super::args::Cli::command();
        cmd.print_long_help()?;
        println!();
    }

    Ok(())
}
