use std::collections::BTreeSet;
use std::fmt::Write as _;

use clap::{Arg, ArgAction, Command};

pub fn render_cli_reference_markdown() -> String {
    let cmd = crate::cli::clap_command();
    render_cli_reference_markdown_from_command(&cmd)
}

fn render_cli_reference_markdown_from_command(cmd: &Command) -> String {
    let mut out = String::new();

    writeln!(&mut out, "# CLI reference").expect("write");
    writeln!(&mut out).expect("write");
    writeln!(
        &mut out,
        "> Language: English | [Chinese (Simplified)](../zh-CN/reference/cli.md)"
    )
    .expect("write");
    writeln!(&mut out).expect("write");
    writeln!(
        &mut out,
        "This document is for quickly looking up how a command works. For workflow-oriented guidance, see `../howto/workflows.md`."
    )
    .expect("write");
    writeln!(&mut out).expect("write");

    writeln!(&mut out, "## Global flags (supported by all commands)").expect("write");
    writeln!(&mut out).expect("write");

    let mut global_args: Vec<&Arg> = cmd
        .get_arguments()
        .filter(|arg| !is_builtin_clap_arg_id(arg.get_id().as_str()))
        .collect();
    global_args.sort_by(|a, b| a.get_id().as_str().cmp(b.get_id().as_str()));

    for arg in global_args {
        let usage = render_arg_usage(arg);
        let help = arg.get_help().map(|h| h.to_string()).unwrap_or_default();
        write_usage_help_bullet(&mut out, &usage, &help);
    }

    writeln!(&mut out).expect("write");
    writeln!(
        &mut out,
        "Tips:\n- `agentpack help --json` returns a structured command list and the mutating command set.\n- `agentpack schema --json` describes the JSON envelope and common `data` payload shapes."
    )
    .expect("write");
    writeln!(&mut out).expect("write");

    let global_arg_ids = cmd
        .get_arguments()
        .filter_map(|arg| {
            let id = arg.get_id().as_str();
            (!is_builtin_clap_arg_id(id)).then_some(id.to_string())
        })
        .collect::<BTreeSet<String>>();

    let mut sections = Vec::new();
    for sub in cmd.get_subcommands() {
        collect_leaf_command_sections(sub, Vec::new(), &global_arg_ids, &mut sections);
    }
    sections.sort_by(|a, b| a.0.cmp(&b.0));

    writeln!(&mut out, "## Commands").expect("write");
    writeln!(&mut out).expect("write");
    writeln!(
        &mut out,
        "All commands below also accept the global flags listed above."
    )
    .expect("write");
    writeln!(&mut out).expect("write");

    for (_id, section) in sections {
        out.push_str(&section);
    }

    #[cfg(not(feature = "tui"))]
    {
        writeln!(&mut out, "## Optional commands").expect("write");
        writeln!(&mut out).expect("write");
        writeln!(&mut out, "### tui").expect("write");
        writeln!(&mut out).expect("write");
        writeln!(
            &mut out,
            "The `tui` command is feature-gated. Build with `--features tui` to enable it. It does not support `--json`."
        )
        .expect("write");
        writeln!(&mut out).expect("write");
    }

    let mut normalized = out.trim_end_matches('\n').to_string();
    normalized.push('\n');
    normalized
}

fn collect_leaf_command_sections(
    cmd: &Command,
    mut parent_path: Vec<String>,
    global_arg_ids: &BTreeSet<String>,
    out: &mut Vec<(String, String)>,
) {
    let name = cmd.get_name().to_string();
    parent_path.push(name);
    let path = parent_path.clone();

    if cmd.get_subcommands().count() == 0 {
        let id = path.join(" ");
        out.push((
            id.clone(),
            render_leaf_command_section(&id, cmd, global_arg_ids),
        ));
        return;
    }

    for sub in cmd.get_subcommands() {
        collect_leaf_command_sections(sub, parent_path.clone(), global_arg_ids, out);
    }
}

fn render_leaf_command_section(
    command_id: &str,
    cmd: &Command,
    global_arg_ids: &BTreeSet<String>,
) -> String {
    let mut out = String::new();

    writeln!(&mut out, "### {command_id}").expect("write");
    writeln!(&mut out).expect("write");

    if let Some(about) = cmd.get_about().map(|s| s.to_string()) {
        writeln!(&mut out, "{about}").expect("write");
        writeln!(&mut out).expect("write");
    }

    let mut positionals: Vec<&Arg> = Vec::new();
    let mut options: Vec<&Arg> = Vec::new();

    for arg in cmd.get_arguments() {
        let id = arg.get_id().as_str();
        if is_builtin_clap_arg_id(id) || global_arg_ids.contains(id) {
            continue;
        }
        if arg.is_positional() {
            positionals.push(arg);
        } else {
            options.push(arg);
        }
    }

    positionals.sort_by(|a, b| {
        (a.get_index().unwrap_or(usize::MAX), a.get_id().as_str())
            .cmp(&(b.get_index().unwrap_or(usize::MAX), b.get_id().as_str()))
    });
    options.sort_by(|a, b| {
        (arg_kind_rank(a), a.get_id().as_str()).cmp(&(arg_kind_rank(b), b.get_id().as_str()))
    });

    let mut usage = format!("agentpack {command_id}");
    for positional in &positionals {
        write!(&mut usage, " {}", render_positional_placeholder(positional)).expect("write");
    }
    write!(&mut usage, " [OPTIONS]").expect("write");

    writeln!(&mut out, "Usage: `{usage}`").expect("write");
    writeln!(&mut out).expect("write");

    if !positionals.is_empty() {
        writeln!(&mut out, "Positional arguments:").expect("write");
        for positional in positionals {
            let name = render_positional_placeholder(positional);
            let help = positional
                .get_help()
                .map(|h| h.to_string())
                .unwrap_or_default();
            write_usage_help_bullet(&mut out, &name, &help);
        }
        writeln!(&mut out).expect("write");
    }

    if !options.is_empty() {
        writeln!(&mut out, "Options:").expect("write");
        for arg in options {
            let usage = render_arg_usage(arg);
            let help = arg.get_help().map(|h| h.to_string()).unwrap_or_default();
            write_usage_help_bullet(&mut out, &usage, &help);
        }
        writeln!(&mut out).expect("write");
    }

    out
}

fn render_arg_usage(arg: &Arg) -> String {
    if arg.is_positional() {
        return render_positional_placeholder(arg);
    }

    let mut out = if let Some(long) = arg.get_long() {
        format!("--{long}")
    } else if let Some(short) = arg.get_short() {
        format!("-{short}")
    } else {
        arg.get_id().as_str().to_string()
    };

    if is_value_taking_arg(arg) {
        let placeholder = render_value_placeholder(arg);
        write!(&mut out, " {placeholder}").expect("write");
    }

    out
}

fn is_value_taking_arg(arg: &Arg) -> bool {
    if arg.is_positional() {
        return true;
    }
    !matches!(
        arg.get_action(),
        ArgAction::SetTrue | ArgAction::SetFalse | ArgAction::Count
    )
}

fn render_value_placeholder(arg: &Arg) -> String {
    let possible = arg
        .get_possible_values()
        .iter()
        .map(|v| v.get_name().to_string())
        .collect::<Vec<String>>();

    if !possible.is_empty() {
        return format!("<{}>", possible.join("|"));
    }

    match arg
        .get_value_names()
        .and_then(|names| names.first().map(|n| n.as_str()))
    {
        Some(name) if !name.is_empty() => format!("<{}>", name.to_ascii_lowercase()),
        _ => "<value>".to_string(),
    }
}

fn render_positional_placeholder(arg: &Arg) -> String {
    let possible = arg
        .get_possible_values()
        .iter()
        .map(|v| v.get_name().to_string())
        .collect::<Vec<String>>();

    if !possible.is_empty() {
        return format!("<{}>", possible.join("|"));
    }

    format!("<{}>", arg.get_id().as_str())
}

fn write_usage_help_bullet(out: &mut String, usage: &str, help: &str) {
    if help.trim().is_empty() {
        writeln!(out, "- `{usage}`").expect("write");
        return;
    }

    writeln!(out, "- `{usage}`: {help}").expect("write");
}

fn is_builtin_clap_arg_id(id: &str) -> bool {
    matches!(id, "help" | "version")
}

fn arg_kind_rank(arg: &Arg) -> u8 {
    if arg.is_positional() {
        return 0;
    }

    match arg.get_action() {
        ArgAction::SetTrue | ArgAction::SetFalse | ArgAction::Count => 2,
        _ => 1,
    }
}
