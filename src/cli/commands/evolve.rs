use crate::app::evolve_propose_json::{
    evolve_propose_json_data_created, evolve_propose_json_data_dry_run,
    evolve_propose_json_data_noop,
};
use crate::app::evolve_restore_json::evolve_restore_json_data;
use crate::engine::Engine;
use crate::output::{JsonEnvelope, print_json};

use super::super::args::{EvolveCommands, EvolveScope};
use super::Ctx;

pub(crate) fn run(ctx: &Ctx<'_>, command: &EvolveCommands) -> anyhow::Result<()> {
    let engine = Engine::load(ctx.cli.repo.as_deref(), ctx.cli.machine.as_deref())?;
    match command {
        EvolveCommands::Propose {
            module_id,
            scope,
            branch,
        } => evolve_propose(
            ctx.cli,
            &engine,
            module_id.as_deref(),
            *scope,
            branch.as_deref(),
        ),
        EvolveCommands::Restore { module_id } => {
            evolve_restore(ctx.cli, &engine, module_id.as_deref())
        }
    }
}

fn evolve_propose(
    cli: &super::super::args::Cli,
    engine: &Engine,
    module_filter: Option<&str>,
    scope: EvolveScope,
    branch_override: Option<&str>,
) -> anyhow::Result<()> {
    let prefix = action_prefix(cli);
    let handler_scope = match scope {
        EvolveScope::Global => crate::handlers::evolve::EvolveScope::Global,
        EvolveScope::Machine => crate::handlers::evolve::EvolveScope::Machine,
        EvolveScope::Project => crate::handlers::evolve::EvolveScope::Project,
    };

    let mut outcome = crate::handlers::evolve::evolve_propose_in(
        engine,
        crate::handlers::evolve::EvolveProposeInput {
            profile: &cli.profile,
            target_filter: &cli.target,
            action_prefix: &prefix,
            module_filter,
            scope: handler_scope,
            branch_override,
            dry_run: cli.dry_run,
            confirmed: cli.yes,
            json: cli.json,
        },
    )?;

    if matches!(
        outcome,
        crate::handlers::evolve::EvolveProposeOutcome::NeedsConfirmation
    ) {
        if !super::super::util::confirm("Create evolve proposal branch?")? {
            println!("Aborted");
            return Ok(());
        }
        outcome = crate::handlers::evolve::evolve_propose_in(
            engine,
            crate::handlers::evolve::EvolveProposeInput {
                profile: &cli.profile,
                target_filter: &cli.target,
                action_prefix: &prefix,
                module_filter,
                scope: handler_scope,
                branch_override,
                dry_run: cli.dry_run,
                confirmed: true,
                json: false,
            },
        )?;
    }

    match outcome {
        crate::handlers::evolve::EvolveProposeOutcome::Noop(report) => {
            if cli.json {
                let data =
                    evolve_propose_json_data_noop(report.reason, report.summary, report.skipped);
                let mut envelope = JsonEnvelope::ok("evolve.propose", data);
                envelope.warnings = report.warnings;
                print_json(&envelope)?;
            } else {
                for w in report.warnings {
                    eprintln!("Warning: {w}");
                }
                if report.reason == "no_drift" {
                    println!("No drifted managed files to propose");
                } else {
                    println!("No proposeable drifted files to propose");
                    if !report.skipped.is_empty() {
                        println!("Skipped drift (not proposeable):");
                        for s in report.skipped {
                            let who = s
                                .module_id
                                .as_deref()
                                .map(|m| m.to_string())
                                .unwrap_or_else(|| {
                                    if s.module_ids.is_empty() {
                                        "-".to_string()
                                    } else {
                                        s.module_ids.join(",")
                                    }
                                });
                            println!("- {} {} {} modules={who}", s.reason, s.target, s.path);
                            match s.reason.as_str() {
                                "missing" => {
                                    println!(
                                        "  hint: run agentpack evolve restore (create-only) or agentpack deploy --apply"
                                    );
                                }
                                "multi_module_output" => {
                                    println!(
                                        "  hint: add per-module markers to aggregated outputs or split outputs so each file maps to one module"
                                    );
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
            Ok(())
        }
        crate::handlers::evolve::EvolveProposeOutcome::DryRun(report) => {
            if cli.json {
                let data = evolve_propose_json_data_dry_run(
                    report.candidates,
                    report.skipped,
                    report.summary,
                );
                let mut envelope = JsonEnvelope::ok("evolve.propose", data);
                envelope.warnings = report.warnings;
                print_json(&envelope)?;
            } else {
                for w in report.warnings {
                    eprintln!("Warning: {w}");
                }
                println!("Candidates (dry-run):");
                for i in report.candidates {
                    println!("- {} {} {}", i.module_id, i.target, i.path);
                }
                if !report.skipped.is_empty() {
                    println!("Skipped drift (not proposeable):");
                    for s in report.skipped {
                        let who = s
                            .module_id
                            .as_deref()
                            .map(|m| m.to_string())
                            .unwrap_or_else(|| {
                                if s.module_ids.is_empty() {
                                    "-".to_string()
                                } else {
                                    s.module_ids.join(",")
                                }
                            });
                        println!("- {} {} {} modules={who}", s.reason, s.target, s.path);
                        match s.reason.as_str() {
                            "missing" => {
                                println!(
                                    "  hint: run agentpack evolve restore (create-only) or agentpack deploy --apply"
                                );
                            }
                            "multi_module_output" => {
                                println!(
                                    "  hint: add per-module markers to aggregated outputs or split outputs so each file maps to one module"
                                );
                            }
                            _ => {}
                        }
                    }
                }
            }
            Ok(())
        }
        crate::handlers::evolve::EvolveProposeOutcome::Created(report) => {
            if let Some(warning) = report.commit_warning.as_deref() {
                eprintln!("Warning: {warning}");
            }

            if cli.json {
                let data = evolve_propose_json_data_created(
                    report.branch,
                    report.scope,
                    report.files,
                    report.files_posix,
                    report.committed,
                );
                let envelope = JsonEnvelope::ok("evolve.propose", data);
                print_json(&envelope)?;
            } else {
                println!("Created proposal branch: {}", report.branch);
                for f in &report.files {
                    println!("- {f}");
                }
                if !report.committed {
                    println!("Note: commit failed; changes are left on the proposal branch.");
                }
            }
            Ok(())
        }
        crate::handlers::evolve::EvolveProposeOutcome::NeedsConfirmation => {
            anyhow::bail!("evolve propose requires confirmation, but confirmation was not provided")
        }
    }
}

fn evolve_restore(
    cli: &super::super::args::Cli,
    engine: &Engine,
    module_filter: Option<&str>,
) -> anyhow::Result<()> {
    let mut outcome = crate::handlers::evolve::evolve_restore_in(
        engine,
        &cli.profile,
        &cli.target,
        module_filter,
        cli.dry_run,
        cli.yes,
        cli.json,
    )?;

    if matches!(
        outcome,
        crate::handlers::evolve::EvolveRestoreOutcome::NeedsConfirmation
    ) {
        if !super::super::util::confirm("Restore missing desired outputs?")? {
            println!("Aborted");
            return Ok(());
        }
        outcome = crate::handlers::evolve::evolve_restore_in(
            engine,
            &cli.profile,
            &cli.target,
            module_filter,
            cli.dry_run,
            true,
            false,
        )?;
    }

    let crate::handlers::evolve::EvolveRestoreOutcome::Done(report) = outcome else {
        anyhow::bail!("evolve restore requires confirmation, but confirmation was not provided")
    };

    if cli.json {
        let data = evolve_restore_json_data(report.restored, report.summary, report.reason);
        let mut envelope = JsonEnvelope::ok("evolve.restore", data);
        envelope.warnings = report.warnings;
        print_json(&envelope)?;
    } else {
        for w in report.warnings {
            eprintln!("Warning: {w}");
        }

        match report.reason {
            "no_missing" => {
                println!("No missing desired outputs to restore");
            }
            "dry_run" => {
                println!("Would restore missing desired outputs (dry-run):");
                for item in report.restored {
                    println!("- {} {}", item.target, item.path);
                }
            }
            _ => {
                println!("Restored missing desired outputs:");
                for item in report.restored {
                    println!("- {} {}", item.target, item.path);
                }
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
