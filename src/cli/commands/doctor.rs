use anyhow::Context as _;

use super::Ctx;

use crate::engine::Engine;
use crate::output::{JsonEnvelope, print_json};

#[derive(Default)]
struct NextActions {
    human: std::collections::BTreeSet<String>,
    json: std::collections::BTreeSet<String>,
}

pub(crate) fn run(ctx: &Ctx<'_>, fix: bool) -> anyhow::Result<()> {
    if fix {
        super::super::util::require_yes_for_json_mutation(ctx.cli, "doctor --fix")?;
    }

    let engine = Engine::load(ctx.cli.repo.as_deref(), ctx.cli.machine.as_deref())?;
    let prefix = action_prefix(ctx.cli);

    let report =
        crate::handlers::doctor::doctor_report_in(&engine, &ctx.cli.profile, &ctx.cli.target, fix)?;
    let mut next_actions = NextActions::default();
    for c in &report.roots {
        if let Some(suggestion) = &c.suggestion {
            if let Some((_, cmd)) = suggestion.split_once(':') {
                let cmd = cmd.trim();
                if !cmd.is_empty() {
                    next_actions.human.insert(cmd.to_string());
                    next_actions.json.insert(cmd.to_string());
                }
            }
        }
    }

    if report.needs_gitignore_fix && !fix {
        next_actions.human.insert(format!("{prefix} doctor --fix"));
        next_actions
            .json
            .insert(format!("{prefix} doctor --fix --yes --json"));
    }

    let crate::handlers::doctor::DoctorReport {
        machine_id,
        roots: checks,
        gitignore_fixes,
        mut warnings,
        ..
    } = report;

    if ctx.cli.json {
        let mut data = serde_json::json!({
            "machine_id": machine_id,
            "roots": checks,
            "gitignore_fixes": gitignore_fixes,
        });
        if !next_actions.json.is_empty() {
            let ordered = ordered_next_actions(&next_actions.json);
            data.as_object_mut()
                .context("doctor json data must be an object")?
                .insert(
                    "next_actions".to_string(),
                    serde_json::to_value(&ordered).context("serialize next_actions")?,
                );
        }
        let mut envelope = JsonEnvelope::ok("doctor", data);
        envelope.warnings = warnings;
        print_json(&envelope)?;
    } else {
        for w in warnings.drain(..) {
            eprintln!("Warning: {w}");
        }
        println!("Machine ID: {machine_id}");
        if fix {
            for f in &gitignore_fixes {
                if f.updated {
                    println!(
                        "Updated {} (added .agentpack.manifest*.json)",
                        f.gitignore_path
                    );
                }
            }
        }
        for c in checks {
            let status = if c.issues.is_empty() { "ok" } else { "issues" };
            println!(
                "- {target} {root} ({status})",
                target = c.target,
                root = c.root
            );
            for issue in c.issues {
                println!("  - issue: {issue}");
            }
            if let Some(s) = c.suggestion {
                println!("  - suggestion: {s}");
            }
        }

        if !next_actions.human.is_empty() {
            println!();
            println!("Next actions:");
            for action in ordered_next_actions(&next_actions.human) {
                println!("- {action}");
            }
        }
    }

    Ok(())
}

fn action_prefix(cli: &crate::cli::args::Cli) -> String {
    let mut out = String::from("agentpack");
    if let Some(repo) = &cli.repo {
        out.push_str(&format!(" --repo {}", repo.display()));
    }
    if cli.profile != "default" {
        out.push_str(&format!(" --profile {}", cli.profile));
    }
    if cli.target != "all" {
        out.push_str(&format!(" --target {}", cli.target));
    }
    if let Some(machine) = &cli.machine {
        out.push_str(&format!(" --machine {machine}"));
    }
    out
}

fn ordered_next_actions(actions: &std::collections::BTreeSet<String>) -> Vec<String> {
    let mut out: Vec<String> = actions.iter().cloned().collect();
    out.sort_by(|a, b| {
        next_action_priority(a)
            .cmp(&next_action_priority(b))
            .then_with(|| a.cmp(b))
    });
    out
}

fn next_action_priority(action: &str) -> u8 {
    match next_action_subcommand(action) {
        Some("bootstrap") => 0,
        Some("doctor") => 10,
        Some("update") => 20,
        Some("preview") => 30,
        Some("diff") => 40,
        Some("plan") => 50,
        Some("deploy") => 60,
        Some("status") => 70,
        Some("evolve") => 80,
        Some("rollback") => 90,
        _ => 100,
    }
}

fn next_action_subcommand(action: &str) -> Option<&str> {
    let mut iter = action.split_whitespace();
    // Skip program name (usually "agentpack") and global flags (and their args).
    let _ = iter.next()?;

    while let Some(tok) = iter.next() {
        if !tok.starts_with("--") {
            return Some(tok);
        }

        // Skip flag value for the flags we know to take an argument.
        if matches!(tok, "--repo" | "--profile" | "--target" | "--machine") {
            let _ = iter.next();
        }
    }

    None
}
