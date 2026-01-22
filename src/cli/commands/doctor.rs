use anyhow::Context as _;

use super::Ctx;

use crate::app::doctor_next_actions::doctor_next_actions;
use crate::app::next_actions::ordered_next_actions;
use crate::engine::Engine;
use crate::output::{JsonEnvelope, print_json};

pub(crate) fn run(ctx: &Ctx<'_>, fix: bool) -> anyhow::Result<()> {
    if fix {
        super::super::util::require_yes_for_json_mutation(ctx.cli, "doctor --fix")?;
    }

    let engine = Engine::load(ctx.cli.repo.as_deref(), ctx.cli.machine.as_deref())?;
    let prefix = action_prefix(ctx.cli);

    let report =
        crate::handlers::doctor::doctor_report_in(&engine, &ctx.cli.profile, &ctx.cli.target, fix)?;
    let next_actions = doctor_next_actions(&report.roots, report.needs_gitignore_fix, fix, &prefix);

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
