use anyhow::Context as _;

use crate::config::TargetScope;
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
    struct DriftSummaryByRoot {
        target: String,
        root: String,
        root_posix: String,
        summary: crate::handlers::status::DriftSummary,
    }

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
    warn_operator_assets_if_outdated(&engine, ctx.cli, &targets, &mut warnings, &mut next_actions)?;
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

    let (mut drift, summary, summary_total_opt) = if only_kinds.is_empty() {
        (drift, summary_total, None)
    } else {
        let drift: Vec<crate::handlers::status::DriftItem> = drift
            .into_iter()
            .filter(|d| only_kinds.contains(d.kind.as_str()))
            .collect();

        let mut summary = crate::handlers::status::DriftSummary::default();
        for d in &drift {
            match d.kind.as_str() {
                "modified" => summary.modified += 1,
                "missing" => summary.missing += 1,
                "extra" => summary.extra += 1,
                _ => {}
            }
        }

        (drift, summary, Some(summary_total))
    };

    let summary_by_root =
        |drift: &[crate::handlers::status::DriftItem]| -> Vec<DriftSummaryByRoot> {
            let mut by_root: std::collections::BTreeMap<(String, String), DriftSummaryByRoot> =
                std::collections::BTreeMap::new();
            for d in drift {
                let root = d.root.as_deref().unwrap_or("<unknown>").to_string();
                let root_posix = d.root_posix.as_deref().unwrap_or("<unknown>").to_string();
                let key = (d.target.clone(), root_posix.clone());
                let entry = by_root.entry(key).or_insert_with(|| DriftSummaryByRoot {
                    target: d.target.clone(),
                    root,
                    root_posix,
                    summary: crate::handlers::status::DriftSummary::default(),
                });
                match d.kind.as_str() {
                    "modified" => entry.summary.modified += 1,
                    "missing" => entry.summary.missing += 1,
                    "extra" => entry.summary.extra += 1,
                    _ => {}
                }
            }
            by_root.into_values().collect()
        };
    let summary_by_root = summary_by_root(&drift);

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

fn target_scope_flags(scope: &TargetScope) -> (bool, bool) {
    match scope {
        TargetScope::User => (true, false),
        TargetScope::Project => (false, true),
        TargetScope::Both => (true, true),
    }
}

fn get_bool(
    map: &std::collections::BTreeMap<String, serde_yaml::Value>,
    key: &str,
    default: bool,
) -> bool {
    match map.get(key) {
        Some(serde_yaml::Value::Bool(b)) => *b,
        Some(serde_yaml::Value::String(s)) => match s.trim().to_ascii_lowercase().as_str() {
            "true" | "yes" | "1" => true,
            "false" | "no" | "0" => false,
            _ => default,
        },
        _ => default,
    }
}

fn extract_agentpack_version(text: &str) -> Option<String> {
    for line in text.lines() {
        if let Some((_, rest)) = line.split_once("agentpack_version:") {
            let mut value = rest.trim();
            value = value.trim_end_matches("-->");
            value = value.trim();
            value = value.trim_matches(|c| c == '"' || c == '\'');
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }
    None
}

fn warn_operator_assets_if_outdated(
    engine: &Engine,
    cli: &crate::cli::args::Cli,
    targets: &[String],
    warnings: &mut Vec<String>,
    next_actions: &mut NextActions,
) -> anyhow::Result<()> {
    let current = env!("CARGO_PKG_VERSION");

    for target in targets {
        match target.as_str() {
            "codex" => {
                let Some(cfg) = engine.manifest.targets.get("codex") else {
                    continue;
                };
                let (allow_user, allow_project) = target_scope_flags(&cfg.scope);
                let codex_home = super::super::util::codex_home_for_manifest(&engine.manifest)?;

                if allow_user {
                    let path = codex_home.join("skills/agentpack-operator/SKILL.md");
                    check_operator_file(
                        &path,
                        "codex/user",
                        current,
                        warnings,
                        &bootstrap_action(cli, "codex", "user"),
                        next_actions,
                    )?;
                }
                if allow_project {
                    let path = engine
                        .project
                        .project_root
                        .join(".codex/skills/agentpack-operator/SKILL.md");
                    check_operator_file(
                        &path,
                        "codex/project",
                        current,
                        warnings,
                        &bootstrap_action(cli, "codex", "project"),
                        next_actions,
                    )?;
                }
            }
            "claude_code" => {
                let Some(cfg) = engine.manifest.targets.get("claude_code") else {
                    continue;
                };
                let (allow_user, allow_project) = target_scope_flags(&cfg.scope);
                let check_user_skills =
                    allow_user && get_bool(&cfg.options, "write_user_skills", false);
                let check_repo_skills =
                    allow_project && get_bool(&cfg.options, "write_repo_skills", false);

                if allow_user {
                    let dir = super::super::util::expand_tilde("~/.claude/commands")?;
                    check_operator_command_dir(
                        &dir,
                        "claude_code/user",
                        current,
                        warnings,
                        &bootstrap_action(cli, "claude_code", "user"),
                        next_actions,
                    )?;
                    if check_user_skills {
                        let skills_dir = super::super::util::expand_tilde("~/.claude/skills")?;
                        check_operator_file(
                            &skills_dir.join("agentpack-operator/SKILL.md"),
                            "claude_code/user",
                            current,
                            warnings,
                            &bootstrap_action(cli, "claude_code", "user"),
                            next_actions,
                        )?;
                    }
                }
                if allow_project {
                    let dir = engine.project.project_root.join(".claude/commands");
                    check_operator_command_dir(
                        &dir,
                        "claude_code/project",
                        current,
                        warnings,
                        &bootstrap_action(cli, "claude_code", "project"),
                        next_actions,
                    )?;
                    if check_repo_skills {
                        check_operator_file(
                            &engine
                                .project
                                .project_root
                                .join(".claude/skills/agentpack-operator/SKILL.md"),
                            "claude_code/project",
                            current,
                            warnings,
                            &bootstrap_action(cli, "claude_code", "project"),
                            next_actions,
                        )?;
                    }
                }
            }
            _ => {}
        }
    }

    Ok(())
}

fn check_operator_file(
    path: &std::path::Path,
    location: &str,
    current: &str,
    warnings: &mut Vec<String>,
    suggested: &str,
    next_actions: &mut NextActions,
) -> anyhow::Result<()> {
    if !path.exists() {
        warnings.push(format!(
            "operator assets missing ({location}): {}; run: {suggested}",
            path.display()
        ));
        next_actions.human.insert(suggested.to_string());
        next_actions
            .json
            .insert(format!("{suggested} --yes --json"));
        return Ok(());
    }

    let text = std::fs::read_to_string(path)
        .with_context(|| format!("read operator asset {}", path.display()))?;
    let Some(have) = extract_agentpack_version(&text) else {
        warnings.push(format!(
            "operator assets missing agentpack_version ({location}): {}; run: {suggested}",
            path.display()
        ));
        next_actions.human.insert(suggested.to_string());
        next_actions
            .json
            .insert(format!("{suggested} --yes --json"));
        return Ok(());
    };

    if have != current {
        warnings.push(format!(
            "operator assets outdated ({location}): {} has {}, want {}; run: {suggested}",
            path.display(),
            have,
            current
        ));
        next_actions.human.insert(suggested.to_string());
        next_actions
            .json
            .insert(format!("{suggested} --yes --json"));
    }

    Ok(())
}

fn check_operator_command_dir(
    dir: &std::path::Path,
    location: &str,
    current: &str,
    warnings: &mut Vec<String>,
    suggested: &str,
    next_actions: &mut NextActions,
) -> anyhow::Result<()> {
    if !dir.exists() {
        warnings.push(format!(
            "operator assets missing ({location}): {}; run: {suggested}",
            dir.display()
        ));
        next_actions.human.insert(suggested.to_string());
        next_actions
            .json
            .insert(format!("{suggested} --yes --json"));
        return Ok(());
    }

    let mut missing = Vec::new();
    let mut missing_version = None;
    for name in [
        "ap-doctor.md",
        "ap-update.md",
        "ap-preview.md",
        "ap-plan.md",
        "ap-deploy.md",
        "ap-status.md",
        "ap-diff.md",
        "ap-explain.md",
        "ap-evolve.md",
    ] {
        let path = dir.join(name);
        if !path.exists() {
            missing.push(name.to_string());
            continue;
        }
        let text = std::fs::read_to_string(&path)
            .with_context(|| format!("read operator asset {}", path.display()))?;
        let Some(have) = extract_agentpack_version(&text) else {
            missing_version = Some(path);
            continue;
        };
        if have != current {
            warnings.push(format!(
                "operator assets outdated ({location}): {} has {}, want {}; run: {suggested}",
                path.display(),
                have,
                current
            ));
        }
    }

    if let Some(path) = missing_version {
        warnings.push(format!(
            "operator assets missing agentpack_version ({location}): {}; run: {suggested}",
            path.display()
        ));
        next_actions.human.insert(suggested.to_string());
        next_actions
            .json
            .insert(format!("{suggested} --yes --json"));
        return Ok(());
    }

    if !missing.is_empty() {
        warnings.push(format!(
            "operator assets incomplete ({location}): missing {}; run: {suggested}",
            missing.join(", "),
        ));
        next_actions.human.insert(suggested.to_string());
        next_actions
            .json
            .insert(format!("{suggested} --yes --json"));
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
        Some("evolve") => {
            if action.contains(" propose") {
                80
            } else {
                81
            }
        }
        Some("rollback") => 90,
        _ => 100,
    }
}

fn next_action_code(action: &str) -> &'static str {
    match next_action_subcommand(action) {
        Some("bootstrap") => "bootstrap",
        Some("doctor") => "doctor",
        Some("update") => "update",
        Some("preview") => {
            if action.contains(" --diff") {
                "preview_diff"
            } else {
                "preview"
            }
        }
        Some("diff") => "diff",
        Some("plan") => "plan",
        Some("deploy") => {
            if action.contains(" --apply") {
                "deploy_apply"
            } else {
                "deploy"
            }
        }
        Some("status") => "status",
        Some("evolve") => {
            if action.contains(" propose") {
                "evolve_propose"
            } else if action.contains(" restore") {
                "evolve_restore"
            } else {
                "evolve"
            }
        }
        Some("rollback") => "rollback",
        _ => "other",
    }
}

fn next_action_subcommand(action: &str) -> Option<&str> {
    let mut iter = action.split_whitespace();
    // Skip program name ("agentpack") and global flags (and their args).
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
