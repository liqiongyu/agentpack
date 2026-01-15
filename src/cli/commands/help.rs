use std::collections::BTreeSet;

use clap::{ArgAction, Command, CommandFactory as _};

use crate::output::{JsonEnvelope, print_json};

use super::Ctx;

#[derive(Debug, Clone, serde::Serialize)]
struct HelpArg {
    id: String,
    kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    long: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    short: Option<String>,
    required: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
struct HelpCommand {
    id: String,
    path: Vec<String>,
    mutating: bool,
    supports_json: bool,
    args: Vec<HelpArg>,
}

pub(crate) fn run(ctx: &Ctx<'_>) -> anyhow::Result<()> {
    if ctx.cli.json {
        let cmd = super::super::args::Cli::command();
        let global_args = help_global_args(&cmd);
        let mut commands = help_leaf_commands(&cmd);
        add_legacy_doctor_fix_variant(&mut commands);
        commands.sort_by(|a, b| a.id.cmp(&b.id));

        let envelope = JsonEnvelope::ok(
            "help",
            serde_json::json!({
                "global_args": global_args,
                "commands": commands,
                "mutating_commands": super::super::util::MUTATING_COMMAND_IDS,
                "targets": crate::target_registry::COMPILED_TARGETS,
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

fn help_global_args(cmd: &Command) -> Vec<HelpArg> {
    let mut out: Vec<HelpArg> = cmd
        .get_arguments()
        .filter_map(|arg| help_arg_from_clap(arg, None))
        .collect();
    out.sort_by(|a, b| a.id.cmp(&b.id));
    out
}

fn help_leaf_commands(cmd: &Command) -> Vec<HelpCommand> {
    let global_arg_ids = cmd
        .get_arguments()
        .filter_map(|arg| {
            let id = arg.get_id().as_str();
            (!is_builtin_clap_arg_id(id)).then_some(id.to_string())
        })
        .collect::<BTreeSet<String>>();

    let mut out = Vec::new();
    for sub in cmd.get_subcommands() {
        collect_leaf_commands(sub, Vec::new(), &global_arg_ids, &mut out);
    }
    out
}

fn collect_leaf_commands(
    cmd: &Command,
    mut parent_path: Vec<String>,
    global_arg_ids: &BTreeSet<String>,
    out: &mut Vec<HelpCommand>,
) {
    let name = cmd.get_name().to_string();
    parent_path.push(name);
    let path = parent_path.clone();

    if cmd.get_subcommands().count() == 0 {
        let id = path.join(" ");
        let mut args: Vec<HelpArg> = cmd
            .get_arguments()
            .filter_map(|arg| help_arg_from_clap(arg, Some(global_arg_ids)))
            .collect();
        args.sort_by(|a, b| (arg_kind_rank(&a.kind), &a.id).cmp(&(arg_kind_rank(&b.kind), &b.id)));

        out.push(HelpCommand {
            mutating: super::super::util::MUTATING_COMMAND_IDS.contains(&id.as_str()),
            supports_json: supports_json_for_command(&id),
            id,
            path,
            args,
        });
        return;
    }

    for sub in cmd.get_subcommands() {
        collect_leaf_commands(sub, parent_path.clone(), global_arg_ids, out);
    }
}

fn add_legacy_doctor_fix_variant(commands: &mut Vec<HelpCommand>) {
    if commands.iter().any(|c| c.id == "doctor --fix") {
        return;
    }
    let Some(doctor) = commands.iter().find(|c| c.id == "doctor").cloned() else {
        return;
    };

    let mut fix = doctor;
    fix.id = "doctor --fix".to_string();
    fix.mutating = true;
    commands.push(fix);
}

fn supports_json_for_command(command_id: &str) -> bool {
    command_id != "completions"
}

fn help_arg_from_clap(
    arg: &clap::Arg,
    global_arg_ids: Option<&BTreeSet<String>>,
) -> Option<HelpArg> {
    let id = arg.get_id().as_str();
    if is_builtin_clap_arg_id(id) {
        return None;
    }
    if global_arg_ids.is_some_and(|ids| ids.contains(id)) {
        return None;
    }

    let kind = if arg.get_index().is_some() {
        "positional"
    } else {
        match arg.get_action() {
            ArgAction::SetTrue | ArgAction::SetFalse | ArgAction::Count => "flag",
            _ => "option",
        }
    };

    let short = arg.get_short().map(|c| c.to_string());
    let long = arg.get_long().map(|s| s.to_string());

    Some(HelpArg {
        id: id.to_string(),
        kind: kind.to_string(),
        long,
        short,
        required: arg.is_required_set(),
    })
}

fn is_builtin_clap_arg_id(id: &str) -> bool {
    matches!(id, "help" | "version")
}

fn arg_kind_rank(kind: &str) -> u8 {
    match kind {
        "positional" => 0,
        "option" => 1,
        "flag" => 2,
        _ => 3,
    }
}
