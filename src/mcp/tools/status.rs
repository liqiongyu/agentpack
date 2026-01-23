use std::collections::BTreeSet;

use anyhow::Context as _;

use crate::app::operator_assets::{
    OperatorAssetsStatusPaths, warn_operator_assets_if_outdated_for_status,
};
use crate::app::status_drift::{drift_summary_by_root, filter_drift_by_kind};
use crate::app::status_json::status_json_data;
use crate::app::status_next_actions::status_next_actions;

fn action_prefix_common(common: &super::CommonArgs) -> String {
    let mut out = String::from("agentpack");
    if let Some(repo) = &common.repo {
        out.push_str(&format!(" --repo {repo}"));
    }
    let profile = common.profile.as_deref().unwrap_or("default");
    if profile != "default" {
        out.push_str(&format!(" --profile {profile}"));
    }
    let target = common.target.as_deref().unwrap_or("all");
    if target != "all" {
        out.push_str(&format!(" --target {target}"));
    }
    if let Some(machine) = &common.machine {
        out.push_str(&format!(" --machine {machine}"));
    }
    out
}

fn bootstrap_action(common: &super::CommonArgs, target: &str, scope: &str) -> String {
    let mut out = String::from("agentpack");
    if let Some(repo) = &common.repo {
        out.push_str(&format!(" --repo {repo}"));
    }
    if let Some(machine) = &common.machine {
        out.push_str(&format!(" --machine {machine}"));
    }
    out.push_str(&format!(" --target {target} bootstrap --scope {scope}"));
    out
}

pub(super) async fn call_status_in_process(
    args: super::StatusArgs,
) -> anyhow::Result<(String, serde_json::Value)> {
    tokio::task::spawn_blocking(move || {
        #[derive(Default)]
        struct NextActions {
            json: BTreeSet<String>,
        }

        let repo_override = args.common.repo.as_ref().map(std::path::PathBuf::from);
        let profile = args.common.profile.as_deref().unwrap_or("default");
        let target = args.common.target.as_deref().unwrap_or("all");
        let machine_override = args.common.machine.as_deref();

        let result = (|| -> anyhow::Result<(String, serde_json::Value)> {
            let engine = crate::engine::Engine::load(repo_override.as_deref(), machine_override)?;
            let targets = crate::cli::util::selected_targets(&engine.manifest, target)?;
            let render = engine.desired_state(profile, target)?;
            let desired = render.desired;
            let mut warnings = render.warnings;
            let roots = render.roots;

            let mut next_actions = NextActions::default();
            if targets
                .iter()
                .any(|t| matches!(t.as_str(), "codex" | "claude_code"))
            {
                let codex_home = crate::cli::util::codex_home_for_manifest(&engine.manifest)?;
                let claude_user_commands_dir =
                    crate::cli::util::expand_tilde("~/.claude/commands")?;
                let claude_user_skills_dir = crate::cli::util::expand_tilde("~/.claude/skills")?;
                let mut record_next_action = |suggested: &str| {
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
                    &mut |target, scope| bootstrap_action(&args.common, target, scope),
                    &mut record_next_action,
                )?;
            }
            let prefix = action_prefix_common(&args.common);

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

            for action in status_next_actions(
                summary.modified,
                summary.missing,
                summary.extra,
                any_manifest,
                report.needs_deploy_apply,
            ) {
                next_actions.json.insert(action.json_command(&prefix));
            }

            let summary_total = summary;
            let only_kinds: BTreeSet<&'static str> = args
                .only
                .as_deref()
                .unwrap_or_default()
                .iter()
                .map(|o| match o {
                    super::StatusOnly::Missing => "missing",
                    super::StatusOnly::Modified => "modified",
                    super::StatusOnly::Extra => "extra",
                })
                .collect();

            let (drift, summary, summary_total_opt) =
                filter_drift_by_kind(drift, &only_kinds, summary_total);
            let summary_by_root = drift_summary_by_root(&drift);

            let data = status_json_data(
                profile,
                targets,
                drift,
                summary,
                summary_by_root,
                summary_total_opt,
                &next_actions.json,
            )?;

            let mut envelope = crate::output::JsonEnvelope::ok("status", data);
            envelope.warnings = warnings;

            let text = serde_json::to_string_pretty(&envelope)?;
            let envelope = serde_json::to_value(&envelope)?;
            Ok((text, envelope))
        })();

        match result {
            Ok(v) => Ok(v),
            Err(err) => {
                let envelope = super::envelope_from_anyhow_error("status", &err);
                let text = serde_json::to_string_pretty(&envelope)?;
                Ok((text, envelope))
            }
        }
    })
    .await
    .context("mcp status handler task join")?
}
