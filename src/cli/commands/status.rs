use anyhow::Context as _;

use crate::app::next_actions::{next_action_code, ordered_next_actions};
use crate::app::operator_assets::{
    OperatorAssetsStatusPaths, warn_operator_assets_if_outdated_for_status,
};
use crate::app::status_drift::{drift_summary_by_root, filter_drift_by_kind};
use crate::engine::Engine;
use crate::output::{JsonEnvelope, print_json};

use super::Ctx;

#[derive(Default)]
struct NextActions {
    human: std::collections::BTreeSet<String>,
    json: std::collections::BTreeSet<String>,
}

pub(crate) fn run(ctx: &Ctx<'_>, only: &[crate::cli::args::StatusOnly]) -> anyhow::Result<()> {
    #[derive(serde::Serialize)]
    struct NextActionDetailed {
        action: String,
        command: String,
    }

    let engine = Engine::load(ctx.cli.repo.as_deref(), ctx.cli.machine.as_deref())?;
    let targets = super::super::util::selected_targets(&engine.manifest, &ctx.cli.target)?;
    let render = engine.desired_state(&ctx.cli.profile, &ctx.cli.target)?;
    let desired = render.desired;
    let mut warnings = render.warnings;
    let roots = render.roots;
    let mut next_actions = NextActions::default();
    if targets
        .iter()
        .any(|t| matches!(t.as_str(), "codex" | "claude_code"))
    {
        let codex_home = super::super::util::codex_home_for_manifest(&engine.manifest)?;
        let claude_user_commands_dir = super::super::util::expand_tilde("~/.claude/commands")?;
        let claude_user_skills_dir = super::super::util::expand_tilde("~/.claude/skills")?;
        let mut record_next_action = |suggested: &str| {
            next_actions.human.insert(suggested.to_string());
            next_actions
                .json
                .insert(format!("{suggested} --yes --json"));
        };
        warn_operator_assets_if_outdated_for_status(
            &engine,
            &targets,
            OperatorAssetsStatusPaths {
                codex_home: &codex_home,
                claude_user_commands_dir: &claude_user_commands_dir,
                claude_user_skills_dir: &claude_user_skills_dir,
            },
            &mut warnings,
            &mut |target, scope| bootstrap_action(ctx.cli, target, scope),
            &mut record_next_action,
        )?;
    }
    let prefix = action_prefix(ctx.cli);

    let report = crate::handlers::status::status_drift_report(
        &desired,
        &roots,
        warnings,
        crate::handlers::status::ExtraScanHashMode::IncludeHashes,
    )?;
    let warnings = report.warnings;
    let drift = report.drift;
    let summary = report.summary;
    let any_manifest = report.any_manifest;

    if report.needs_deploy_apply {
        next_actions
            .human
            .insert(format!("{prefix} deploy --apply"));
        next_actions
            .json
            .insert(format!("{prefix} deploy --apply --yes --json"));
    }

    if summary.modified > 0 || summary.missing > 0 {
        next_actions
            .human
            .insert(format!("{prefix} preview --diff"));
        next_actions
            .human
            .insert(format!("{prefix} deploy --apply"));

        next_actions
            .json
            .insert(format!("{prefix} preview --diff --json"));
        next_actions
            .json
            .insert(format!("{prefix} deploy --apply --yes --json"));

        // Only suggest evolve when there is a reliable baseline (a previous deploy wrote manifests).
        if any_manifest {
            next_actions
                .human
                .insert(format!("{prefix} evolve propose"));
            next_actions
                .json
                .insert(format!("{prefix} evolve propose --yes --json"));
        }
    } else if summary.extra > 0 {
        next_actions
            .human
            .insert(format!("{prefix} preview --diff"));
        next_actions
            .json
            .insert(format!("{prefix} preview --diff --json"));
    }

    let summary_total = summary;
    let only_kinds: std::collections::BTreeSet<&'static str> = only
        .iter()
        .map(|o| match o {
            crate::cli::args::StatusOnly::Missing => "missing",
            crate::cli::args::StatusOnly::Modified => "modified",
            crate::cli::args::StatusOnly::Extra => "extra",
        })
        .collect();

    let (mut drift, summary, summary_total_opt) =
        filter_drift_by_kind(drift, &only_kinds, summary_total);
    let summary_by_root = drift_summary_by_root(&drift);

    if ctx.cli.json {
        let mut data = serde_json::json!({
            "profile": ctx.cli.profile,
            "targets": targets,
            "drift": drift,
            "summary": summary,
            "summary_by_root": summary_by_root,
        });
        if let Some(summary_total) = summary_total_opt {
            data.as_object_mut()
                .context("status json data must be an object")?
                .insert(
                    "summary_total".to_string(),
                    serde_json::to_value(summary_total).context("serialize summary_total")?,
                );
        }
        if !next_actions.json.is_empty() {
            let ordered = ordered_next_actions(&next_actions.json);
            data.as_object_mut()
                .context("status json data must be an object")?
                .insert(
                    "next_actions".to_string(),
                    serde_json::to_value(&ordered).context("serialize next_actions")?,
                );
            let detailed: Vec<NextActionDetailed> = ordered
                .into_iter()
                .map(|command| NextActionDetailed {
                    action: next_action_code(&command).to_string(),
                    command,
                })
                .collect();
            data.as_object_mut()
                .context("status json data must be an object")?
                .insert(
                    "next_actions_detailed".to_string(),
                    serde_json::to_value(&detailed).context("serialize next_actions_detailed")?,
                );
        }

        let mut envelope = JsonEnvelope::ok("status", data);
        envelope.warnings = warnings;
        print_json(&envelope)?;
    } else if drift.is_empty() {
        for w in warnings {
            eprintln!("Warning: {w}");
        }
        let is_filtered = summary_total_opt.is_some();
        if is_filtered {
            println!("No drift (filtered)");
            if let Some(total) = summary_total_opt {
                if total.modified > 0 || total.missing > 0 || total.extra > 0 {
                    println!(
                        "Summary (total): modified={} missing={} extra={}",
                        total.modified, total.missing, total.extra
                    );
                }
            }
        } else {
            println!("No drift");
        }

        if !next_actions.human.is_empty() {
            println!();
            println!("Next actions:");
            for action in ordered_next_actions(&next_actions.human) {
                println!("- {action}");
            }
        }
    } else {
        for w in warnings {
            eprintln!("Warning: {w}");
        }
        println!("Drift ({}):", drift.len());
        if let Some(total) = summary_total_opt {
            println!(
                "Summary (filtered): modified={} missing={} extra={}",
                summary.modified, summary.missing, summary.extra
            );
            println!(
                "Summary (total): modified={} missing={} extra={}",
                total.modified, total.missing, total.extra
            );
        } else {
            println!(
                "Summary: modified={} missing={} extra={}",
                summary.modified, summary.missing, summary.extra
            );
        }
        drift.sort_by(|a, b| {
            (
                a.target.as_str(),
                a.root.as_deref().unwrap_or(""),
                a.path.as_str(),
            )
                .cmp(&(
                    b.target.as_str(),
                    b.root.as_deref().unwrap_or(""),
                    b.path.as_str(),
                ))
        });
        let by_root: std::collections::BTreeMap<
            (String, String),
            crate::handlers::status::DriftSummary,
        > = summary_by_root
            .into_iter()
            .map(|s| ((s.target, s.root), s.summary))
            .collect();
        let mut last_group: Option<(String, String)> = None;
        for d in drift {
            let root = d.root.as_deref().unwrap_or("<unknown>");
            let group = (d.target.clone(), root.to_string());
            if last_group.as_ref() != Some(&group) {
                let group_summary = by_root.get(&group).copied().unwrap_or_default();
                println!(
                    "Root: {} ({}) modified={} missing={} extra={}",
                    root,
                    d.target,
                    group_summary.modified,
                    group_summary.missing,
                    group_summary.extra
                );
                last_group = Some(group);
            }
            println!("- {} {}", d.kind, d.path);
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

fn bootstrap_action(cli: &crate::cli::args::Cli, target: &str, scope: &str) -> String {
    let mut out = String::from("agentpack");
    if let Some(repo) = &cli.repo {
        out.push_str(&format!(" --repo {}", repo.display()));
    }
    if let Some(machine) = &cli.machine {
        out.push_str(&format!(" --machine {machine}"));
    }
    out.push_str(&format!(" --target {target} bootstrap --scope {scope}"));
    out
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
