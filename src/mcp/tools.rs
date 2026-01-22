use std::sync::Arc;
use std::time::Instant;

use anyhow::Context as _;
use rmcp::{
    ErrorData as McpError,
    model::{CallToolRequestParam, CallToolResult, Content, JsonObject, Tool, ToolAnnotations},
};

use crate::app::doctor_json::doctor_json_data;
use crate::app::doctor_next_actions::doctor_next_actions;
use crate::app::operator_assets::{
    OperatorAssetsStatusPaths, warn_operator_assets_if_outdated_for_status,
};
use crate::app::status_drift::{drift_summary_by_root, filter_drift_by_kind};
use crate::app::status_json::status_json_data;
use crate::app::status_next_actions::status_next_actions;
use crate::user_error::UserError;

use super::confirm::{CONFIRM_TOKEN_TTL, ConfirmTokenBinding};
use super::{AgentpackMcp, confirm};

pub(super) const TOOLS_INSTRUCTIONS: &str = "Agentpack MCP server (stdio). Tools: plan, diff, preview, status, doctor, deploy, deploy_apply, rollback, evolve_propose, evolve_restore, explain.";

fn envelope_error(
    command: &str,
    code: &str,
    message: &str,
    details: Option<serde_json::Value>,
) -> serde_json::Value {
    let mut err = serde_json::Map::from_iter([
        (
            "code".to_string(),
            serde_json::Value::String(code.to_string()),
        ),
        (
            "message".to_string(),
            serde_json::Value::String(message.to_string()),
        ),
    ]);
    if let Some(details) = details {
        err.insert("details".to_string(), details);
    }

    serde_json::Value::Object(serde_json::Map::from_iter([
        (
            "schema_version".to_string(),
            serde_json::Value::Number(1.into()),
        ),
        ("ok".to_string(), serde_json::Value::Bool(false)),
        (
            "command".to_string(),
            serde_json::Value::String(command.to_string()),
        ),
        (
            "version".to_string(),
            serde_json::Value::String(env!("CARGO_PKG_VERSION").to_string()),
        ),
        (
            "data".to_string(),
            serde_json::Value::Object(serde_json::Map::new()),
        ),
        ("warnings".to_string(), serde_json::Value::Array(Vec::new())),
        (
            "errors".to_string(),
            serde_json::Value::Array(vec![serde_json::Value::Object(err)]),
        ),
    ]))
}

#[derive(Debug, Default, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct CommonArgs {
    #[serde(default)]
    pub repo: Option<String>,
    #[serde(default)]
    pub profile: Option<String>,
    #[serde(default)]
    pub target: Option<String>,
    #[serde(default)]
    pub machine: Option<String>,
    #[serde(default)]
    pub dry_run: Option<bool>,
}

#[derive(Debug, Default, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct StatusArgs {
    #[serde(flatten)]
    pub common: CommonArgs,
    #[serde(default)]
    pub only: Option<Vec<StatusOnly>>,
}

#[derive(Debug, Clone, Copy, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub(super) enum StatusOnly {
    Missing,
    Modified,
    Extra,
}

#[derive(Debug, Default, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct DoctorArgs {
    #[serde(default)]
    pub repo: Option<String>,
    #[serde(default)]
    pub target: Option<String>,
}

#[derive(Debug, Default, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct DeployApplyArgs {
    #[serde(flatten)]
    pub common: CommonArgs,
    #[serde(default)]
    pub adopt: Option<bool>,
    #[serde(default)]
    pub confirm_token: Option<String>,
    #[serde(default)]
    pub yes: bool,
}

#[derive(Debug, Default, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct RollbackArgs {
    #[serde(default)]
    pub repo: Option<String>,
    pub to: String,
    #[serde(default)]
    pub yes: bool,
}

#[derive(Debug, Default, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct PreviewArgs {
    #[serde(flatten)]
    pub common: CommonArgs,
    #[serde(default)]
    pub diff: bool,
}

#[derive(Debug, Clone, Copy, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub(super) enum EvolveScopeArg {
    Global,
    Machine,
    Project,
}

#[derive(Debug, Default, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct EvolveProposeArgs {
    #[serde(flatten)]
    pub common: CommonArgs,
    #[serde(default)]
    pub module_id: Option<String>,
    #[serde(default)]
    pub scope: Option<EvolveScopeArg>,
    #[serde(default)]
    pub branch: Option<String>,
    #[serde(default)]
    pub yes: bool,
}

#[derive(Debug, Default, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct EvolveRestoreArgs {
    #[serde(flatten)]
    pub common: CommonArgs,
    #[serde(default)]
    pub module_id: Option<String>,
    #[serde(default)]
    pub yes: bool,
}

#[derive(Debug, Clone, Copy, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub(super) enum ExplainKindArg {
    Plan,
    Diff,
    Status,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct ExplainArgs {
    #[serde(flatten)]
    pub common: CommonArgs,
    pub kind: ExplainKindArg,
}

async fn call_read_only_in_process(
    command: &'static str,
    args: CommonArgs,
) -> anyhow::Result<(String, serde_json::Value)> {
    tokio::task::spawn_blocking(move || {
        let repo_override = args.repo.as_ref().map(std::path::PathBuf::from);
        let profile = args.profile.as_deref().unwrap_or("default");
        let target = args.target.as_deref().unwrap_or("all");
        let machine_override = args.machine.as_deref();

        let result = crate::handlers::read_only::read_only_context(
            repo_override.as_deref(),
            machine_override,
            profile,
            target,
        );

        let (text, envelope) = match result {
            Ok(crate::handlers::read_only::ReadOnlyContext {
                targets,
                plan,
                warnings,
                ..
            }) => {
                let data = crate::app::plan_json::plan_json_data(profile, targets, plan);
                let mut envelope = crate::output::JsonEnvelope::ok(command, data);
                envelope.warnings = warnings;
                let text = serde_json::to_string_pretty(&envelope)?;
                let envelope = serde_json::to_value(&envelope)?;
                (text, envelope)
            }
            Err(err) => {
                let user_err = err.chain().find_map(|e| e.downcast_ref::<UserError>());
                let code = user_err
                    .map(|e| e.code.clone())
                    .unwrap_or_else(|| "E_UNEXPECTED".to_string());
                let message = user_err
                    .map(|e| e.message.clone())
                    .unwrap_or_else(|| err.to_string());
                let details = user_err.and_then(|e| e.details.clone());
                let envelope = envelope_error(command, &code, &message, details);
                let text = serde_json::to_string_pretty(&envelope)?;
                (text, envelope)
            }
        };

        Ok((text, envelope))
    })
    .await
    .context("mcp read-only handler task join")?
}

fn action_prefix(repo: Option<&str>, target: &str) -> String {
    let mut out = String::from("agentpack");
    if let Some(repo) = repo {
        out.push_str(&format!(" --repo {repo}"));
    }
    if target != "all" {
        out.push_str(&format!(" --target {target}"));
    }
    out
}

fn action_prefix_common(common: &CommonArgs) -> String {
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

fn bootstrap_action(common: &CommonArgs, target: &str, scope: &str) -> String {
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

async fn call_doctor_in_process(args: DoctorArgs) -> anyhow::Result<(String, serde_json::Value)> {
    tokio::task::spawn_blocking(move || {
        let repo_override = args.repo.as_ref().map(std::path::PathBuf::from);
        let target = args.target.as_deref().unwrap_or("all");

        let engine = match crate::engine::Engine::load(repo_override.as_deref(), None) {
            Ok(v) => v,
            Err(err) => {
                let user_err = err.chain().find_map(|e| e.downcast_ref::<UserError>());
                let code = user_err
                    .map(|e| e.code.clone())
                    .unwrap_or_else(|| "E_UNEXPECTED".to_string());
                let message = user_err
                    .map(|e| e.message.clone())
                    .unwrap_or_else(|| err.to_string());
                let details = user_err.and_then(|e| e.details.clone());
                let envelope = envelope_error("doctor", &code, &message, details);
                let text = serde_json::to_string_pretty(&envelope)?;
                return Ok((text, envelope));
            }
        };

        let report = crate::handlers::doctor::doctor_report_in(&engine, "default", target, false);
        let report = match report {
            Ok(v) => v,
            Err(err) => {
                let user_err = err.chain().find_map(|e| e.downcast_ref::<UserError>());
                let code = user_err
                    .map(|e| e.code.clone())
                    .unwrap_or_else(|| "E_UNEXPECTED".to_string());
                let message = user_err
                    .map(|e| e.message.clone())
                    .unwrap_or_else(|| err.to_string());
                let details = user_err.and_then(|e| e.details.clone());
                let envelope = envelope_error("doctor", &code, &message, details);
                let text = serde_json::to_string_pretty(&envelope)?;
                return Ok((text, envelope));
            }
        };

        let prefix = action_prefix(args.repo.as_deref(), target);
        let next_actions =
            doctor_next_actions(&report.roots, report.needs_gitignore_fix, false, &prefix);

        let crate::handlers::doctor::DoctorReport {
            machine_id,
            roots,
            gitignore_fixes,
            warnings,
            ..
        } = report;
        let data = doctor_json_data(machine_id, roots, gitignore_fixes, &next_actions.json)?;

        let mut envelope = crate::output::JsonEnvelope::ok("doctor", data);
        envelope.warnings = warnings;

        let text = serde_json::to_string_pretty(&envelope)?;
        let envelope = serde_json::to_value(&envelope)?;
        Ok((text, envelope))
    })
    .await
    .context("mcp doctor handler task join")?
}

async fn call_status_in_process(args: StatusArgs) -> anyhow::Result<(String, serde_json::Value)> {
    tokio::task::spawn_blocking(move || {
        #[derive(Default)]
        struct NextActions {
            json: std::collections::BTreeSet<String>,
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
            let only_kinds: std::collections::BTreeSet<&'static str> = args
                .only
                .as_deref()
                .unwrap_or_default()
                .iter()
                .map(|o| match o {
                    StatusOnly::Missing => "missing",
                    StatusOnly::Modified => "modified",
                    StatusOnly::Extra => "extra",
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
                let user_err = err.chain().find_map(|e| e.downcast_ref::<UserError>());
                let code = user_err
                    .map(|e| e.code.clone())
                    .unwrap_or_else(|| "E_UNEXPECTED".to_string());
                let message = user_err
                    .map(|e| e.message.clone())
                    .unwrap_or_else(|| err.to_string());
                let details = user_err.and_then(|e| e.details.clone());
                let envelope = envelope_error("status", &code, &message, details);
                let text = serde_json::to_string_pretty(&envelope)?;
                Ok((text, envelope))
            }
        }
    })
    .await
    .context("mcp status handler task join")?
}

async fn call_explain_in_process(args: ExplainArgs) -> anyhow::Result<(String, serde_json::Value)> {
    tokio::task::spawn_blocking(move || {
        #[derive(serde::Serialize)]
        struct ExplainedModule {
            module_id: String,
            module_type: Option<String>,
            layer: Option<String>,
            module_path: Option<String>,
        }

        #[derive(serde::Serialize)]
        struct ExplainedChange {
            op: String,
            target: String,
            path: String,
            path_posix: String,
            modules: Vec<ExplainedModule>,
        }

        #[derive(serde::Serialize)]
        struct ExplainedDrift {
            kind: String,
            target: String,
            path: String,
            path_posix: String,
            expected: Option<String>,
            actual: Option<String>,
            modules: Vec<String>,
        }

        let repo_override = args.common.repo.as_ref().map(std::path::PathBuf::from);
        let profile = args.common.profile.as_deref().unwrap_or("default");
        let target = args.common.target.as_deref().unwrap_or("all");
        let machine_override = args.common.machine.as_deref();

        let command = match args.kind {
            ExplainKindArg::Status => "explain.status",
            ExplainKindArg::Plan | ExplainKindArg::Diff => "explain.plan",
        };

        let result = (|| -> anyhow::Result<(String, serde_json::Value)> {
            let engine = crate::engine::Engine::load(repo_override.as_deref(), machine_override)?;

            match args.kind {
                ExplainKindArg::Plan | ExplainKindArg::Diff => {
                    let targets = crate::cli::util::selected_targets(&engine.manifest, target)?;
                    let render = engine.desired_state(profile, target)?;
                    let desired = render.desired;
                    let mut warnings = render.warnings;
                    let roots = render.roots;

                    let manifest_index = crate::cli::util::load_manifest_module_ids(&roots)?;
                    warnings.extend(manifest_index.warnings);
                    let manifest_index = manifest_index.index;

                    let managed_paths_from_manifest =
                        crate::target_manifest::load_managed_paths_from_manifests(&roots)?;
                    warnings.extend(managed_paths_from_manifest.warnings);
                    let managed_paths_from_manifest = managed_paths_from_manifest.managed_paths;
                    let managed_paths = if !managed_paths_from_manifest.is_empty() {
                        Some(crate::cli::util::filter_managed(
                            managed_paths_from_manifest,
                            target,
                        ))
                    } else {
                        crate::state::latest_snapshot(&engine.home, &["deploy", "rollback"])?
                            .as_ref()
                            .map(crate::deploy::load_managed_paths_from_snapshot)
                            .transpose()?
                            .map(|m| crate::cli::util::filter_managed(m, target))
                    };
                    let plan = crate::deploy::plan(&desired, managed_paths.as_ref())?;

                    let mut explained = Vec::new();
                    for c in &plan.changes {
                        let tp = crate::deploy::TargetPath {
                            target: c.target.clone(),
                            path: std::path::PathBuf::from(&c.path),
                        };

                        let module_ids = match c.op {
                            crate::deploy::Op::Delete => {
                                manifest_index.get(&tp).cloned().unwrap_or_default()
                            }
                            crate::deploy::Op::Create | crate::deploy::Op::Update => desired
                                .get(&tp)
                                .map(|f| f.module_ids.clone())
                                .unwrap_or_default(),
                        };

                        let mut modules = Vec::new();
                        for module_id in module_ids {
                            let module = engine.manifest.modules.iter().find(|m| m.id == module_id);
                            let module_type = module.map(|m| format!("{:?}", m.module_type));
                            let module_path = module.and_then(|m| {
                                crate::cli::util::module_rel_path_for_output(
                                    m, &module_id, &tp, &roots,
                                )
                            });
                            let layer = match (module, module_path.as_deref()) {
                                (Some(m), Some(rel)) => Some(
                                    crate::cli::util::source_layer_for_module_file(
                                        &engine, m, rel,
                                    )?,
                                ),
                                _ => None,
                            };
                            modules.push(ExplainedModule {
                                module_id,
                                module_type,
                                layer,
                                module_path,
                            });
                        }

                        explained.push(ExplainedChange {
                            op: format!("{:?}", c.op).to_lowercase(),
                            target: c.target.clone(),
                            path: c.path.clone(),
                            path_posix: c.path_posix.clone(),
                            modules,
                        });
                    }

                    let mut envelope = crate::output::JsonEnvelope::ok(
                        "explain.plan",
                        serde_json::json!({
                            "profile": profile,
                            "targets": targets,
                            "changes": explained,
                        }),
                    );
                    envelope.warnings = warnings;
                    let text = serde_json::to_string_pretty(&envelope)?;
                    let envelope = serde_json::to_value(&envelope)?;
                    Ok((text, envelope))
                }
                ExplainKindArg::Status => {
                    let targets = crate::cli::util::selected_targets(&engine.manifest, target)?;
                    let render = engine.desired_state(profile, target)?;
                    let desired = render.desired;
                    let mut warnings = render.warnings;
                    let roots = render.roots;

                    let manifest_index = crate::cli::util::load_manifest_module_ids(&roots)?;
                    warnings.extend(manifest_index.warnings);
                    let manifest_index = manifest_index.index;

                    let managed_paths_from_manifest =
                        crate::target_manifest::load_managed_paths_from_manifests(&roots)?;
                    warnings.extend(managed_paths_from_manifest.warnings);
                    let managed_paths_from_manifest = crate::cli::util::filter_managed(
                        managed_paths_from_manifest.managed_paths,
                        target,
                    );

                    let mut drift = Vec::new();
                    if managed_paths_from_manifest.is_empty() {
                        warnings.push(
                            "no target manifests found; drift may be inaccurate (run deploy --apply to write manifests)".to_string(),
                        );
                        for (tp, desired_file) in &desired {
                            let expected = format!(
                                "sha256:{}",
                                crate::hash::sha256_hex(&desired_file.bytes)
                            );
                            match std::fs::read(&tp.path) {
                                Ok(actual_bytes) => {
                                    let actual =
                                        format!("sha256:{}", crate::hash::sha256_hex(&actual_bytes));
                                    if actual != expected {
                                        drift.push(ExplainedDrift {
                                            kind: "modified".to_string(),
                                            target: tp.target.clone(),
                                            path: tp.path.to_string_lossy().to_string(),
                                            path_posix: crate::paths::path_to_posix_string(&tp.path),
                                            expected: Some(expected),
                                            actual: Some(actual),
                                            modules: desired_file.module_ids.clone(),
                                        });
                                    }
                                }
                                Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                                    drift.push(ExplainedDrift {
                                        kind: "missing".to_string(),
                                        target: tp.target.clone(),
                                        path: tp.path.to_string_lossy().to_string(),
                                        path_posix: crate::paths::path_to_posix_string(&tp.path),
                                        expected: Some(expected),
                                        actual: None,
                                        modules: desired_file.module_ids.clone(),
                                    })
                                }
                                Err(err) => return Err(err).context("read deployed file"),
                            }
                        }
                    } else {
                        for tp in &managed_paths_from_manifest {
                            let expected = desired
                                .get(tp)
                                .map(|f| format!("sha256:{}", crate::hash::sha256_hex(&f.bytes)));
                            match std::fs::read(&tp.path) {
                                Ok(actual_bytes) => {
                                    let actual =
                                        format!("sha256:{}", crate::hash::sha256_hex(&actual_bytes));
                                    if let Some(exp) = &expected {
                                        if &actual != exp {
                                            drift.push(ExplainedDrift {
                                                kind: "modified".to_string(),
                                                target: tp.target.clone(),
                                                path: tp.path.to_string_lossy().to_string(),
                                                path_posix: crate::paths::path_to_posix_string(
                                                    &tp.path,
                                                ),
                                                expected: Some(exp.clone()),
                                                actual: Some(actual),
                                                modules: manifest_index
                                                    .get(tp)
                                                    .cloned()
                                                    .unwrap_or_default(),
                                            });
                                        }
                                    } else {
                                        drift.push(ExplainedDrift {
                                            kind: "extra".to_string(),
                                            target: tp.target.clone(),
                                            path: tp.path.to_string_lossy().to_string(),
                                            path_posix: crate::paths::path_to_posix_string(&tp.path),
                                            expected: None,
                                            actual: Some(actual),
                                            modules: manifest_index
                                                .get(tp)
                                                .cloned()
                                                .unwrap_or_default(),
                                        });
                                    }
                                }
                                Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                                    if let Some(exp) = expected {
                                        drift.push(ExplainedDrift {
                                            kind: "missing".to_string(),
                                            target: tp.target.clone(),
                                            path: tp.path.to_string_lossy().to_string(),
                                            path_posix: crate::paths::path_to_posix_string(
                                                &tp.path,
                                            ),
                                            expected: Some(exp),
                                            actual: None,
                                            modules: manifest_index
                                                .get(tp)
                                                .cloned()
                                                .unwrap_or_default(),
                                        });
                                    }
                                }
                                Err(err) => return Err(err).context("read deployed file"),
                            }
                        }
                    }

                    let mut envelope = crate::output::JsonEnvelope::ok(
                        "explain.status",
                        serde_json::json!({
                            "profile": profile,
                            "targets": targets,
                            "drift": drift,
                        }),
                    );
                    envelope.warnings = warnings;
                    let text = serde_json::to_string_pretty(&envelope)?;
                    let envelope = serde_json::to_value(&envelope)?;
                    Ok((text, envelope))
                }
            }
        })();

        match result {
            Ok(v) => Ok(v),
            Err(err) => {
                let user_err = err.chain().find_map(|e| e.downcast_ref::<UserError>());
                let code = user_err
                    .map(|e| e.code.clone())
                    .unwrap_or_else(|| "E_UNEXPECTED".to_string());
                let message = user_err
                    .map(|e| e.message.clone())
                    .unwrap_or_else(|| err.to_string());
                let details = user_err.and_then(|e| e.details.clone());
                let envelope = envelope_error(command, &code, &message, details);
                let text = serde_json::to_string_pretty(&envelope)?;
                Ok((text, envelope))
            }
        }
    })
    .await
    .context("mcp explain handler task join")?
}

async fn call_rollback_in_process(
    args: RollbackArgs,
) -> anyhow::Result<(String, serde_json::Value)> {
    tokio::task::spawn_blocking(move || {
        let home = crate::paths::AgentpackHome::resolve().context("resolve agentpack home")?;

        let RollbackArgs {
            repo: _repo_override,
            to: snapshot_id,
            yes,
        } = args;
        let result = crate::handlers::rollback::rollback(&home, &snapshot_id, true, yes);

        let (text, envelope) = match result {
            Ok(event) => {
                let data = crate::app::rollback_json::rollback_json_data(&snapshot_id, &event.id);
                let envelope = crate::output::JsonEnvelope::ok("rollback", data);
                let text = serde_json::to_string_pretty(&envelope)?;
                let envelope = serde_json::to_value(&envelope)?;
                (text, envelope)
            }
            Err(err) => {
                let user_err = err.chain().find_map(|e| e.downcast_ref::<UserError>());
                let code = user_err
                    .map(|e| e.code.clone())
                    .unwrap_or_else(|| "E_UNEXPECTED".to_string());
                let message = user_err
                    .map(|e| e.message.clone())
                    .unwrap_or_else(|| err.to_string());
                let details = user_err.and_then(|e| e.details.clone());
                let envelope = envelope_error("rollback", &code, &message, details);
                let text = serde_json::to_string_pretty(&envelope)?;
                (text, envelope)
            }
        };

        Ok((text, envelope))
    })
    .await
    .context("mcp rollback handler task join")?
}

async fn call_evolve_restore_in_process(
    args: EvolveRestoreArgs,
) -> anyhow::Result<(String, serde_json::Value)> {
    tokio::task::spawn_blocking(move || {
        let repo_override = args.common.repo.as_ref().map(std::path::PathBuf::from);
        let profile = args.common.profile.as_deref().unwrap_or("default");
        let target = args.common.target.as_deref().unwrap_or("all");
        let machine_override = args.common.machine.as_deref();
        let dry_run = args.common.dry_run.unwrap_or(false);

        let engine = match crate::engine::Engine::load(repo_override.as_deref(), machine_override) {
            Ok(v) => v,
            Err(err) => {
                let user_err = err.chain().find_map(|e| e.downcast_ref::<UserError>());
                let code = user_err
                    .map(|e| e.code.clone())
                    .unwrap_or_else(|| "E_UNEXPECTED".to_string());
                let message = user_err
                    .map(|e| e.message.clone())
                    .unwrap_or_else(|| err.to_string());
                let details = user_err.and_then(|e| e.details.clone());
                let envelope = envelope_error("evolve.restore", &code, &message, details);
                let text = serde_json::to_string_pretty(&envelope)?;
                return Ok((text, envelope));
            }
        };

        let result = crate::handlers::evolve::evolve_restore_in(
            &engine,
            profile,
            target,
            args.module_id.as_deref(),
            dry_run,
            args.yes,
            true,
        );

        let (text, envelope) = match result {
            Ok(crate::handlers::evolve::EvolveRestoreOutcome::Done(report)) => {
                let mut envelope = crate::output::JsonEnvelope::ok(
                    "evolve.restore",
                    serde_json::json!({
                        "restored": report.restored,
                        "summary": report.summary,
                        "reason": report.reason,
                    }),
                );
                envelope.warnings = report.warnings;
                let text = serde_json::to_string_pretty(&envelope)?;
                let envelope = serde_json::to_value(&envelope)?;
                (text, envelope)
            }
            Ok(crate::handlers::evolve::EvolveRestoreOutcome::NeedsConfirmation) => {
                let err = UserError::confirm_required("evolve restore");
                let user_err = err.chain().find_map(|e| e.downcast_ref::<UserError>());
                let code = user_err
                    .map(|e| e.code.clone())
                    .unwrap_or_else(|| "E_UNEXPECTED".to_string());
                let message = user_err
                    .map(|e| e.message.clone())
                    .unwrap_or_else(|| err.to_string());
                let details = user_err.and_then(|e| e.details.clone());
                let envelope = envelope_error("evolve.restore", &code, &message, details);
                let text = serde_json::to_string_pretty(&envelope)?;
                (text, envelope)
            }
            Err(err) => {
                let user_err = err.chain().find_map(|e| e.downcast_ref::<UserError>());
                let code = user_err
                    .map(|e| e.code.clone())
                    .unwrap_or_else(|| "E_UNEXPECTED".to_string());
                let message = user_err
                    .map(|e| e.message.clone())
                    .unwrap_or_else(|| err.to_string());
                let details = user_err.and_then(|e| e.details.clone());
                let envelope = envelope_error("evolve.restore", &code, &message, details);
                let text = serde_json::to_string_pretty(&envelope)?;
                (text, envelope)
            }
        };

        Ok((text, envelope))
    })
    .await
    .context("mcp evolve_restore handler task join")?
}

async fn call_evolve_propose_in_process(
    args: EvolveProposeArgs,
) -> anyhow::Result<(String, serde_json::Value)> {
    tokio::task::spawn_blocking(move || {
        let repo_override = args.common.repo.as_ref().map(std::path::PathBuf::from);
        let profile = args.common.profile.as_deref().unwrap_or("default");
        let target = args.common.target.as_deref().unwrap_or("all");
        let machine_override = args.common.machine.as_deref();
        let dry_run = args.common.dry_run.unwrap_or(false);

        let action_prefix = {
            let mut out = String::from("agentpack");
            if let Some(repo) = args.common.repo.as_deref() {
                out.push_str(&format!(" --repo {repo}"));
            }
            if profile != "default" {
                out.push_str(&format!(" --profile {profile}"));
            }
            if target != "all" {
                out.push_str(&format!(" --target {target}"));
            }
            if let Some(machine) = machine_override {
                out.push_str(&format!(" --machine {machine}"));
            }
            out
        };

        let scope = match args.scope.unwrap_or(EvolveScopeArg::Global) {
            EvolveScopeArg::Global => crate::handlers::evolve::EvolveScope::Global,
            EvolveScopeArg::Machine => crate::handlers::evolve::EvolveScope::Machine,
            EvolveScopeArg::Project => crate::handlers::evolve::EvolveScope::Project,
        };

        let engine = match crate::engine::Engine::load(repo_override.as_deref(), machine_override) {
            Ok(v) => v,
            Err(err) => {
                let user_err = err.chain().find_map(|e| e.downcast_ref::<UserError>());
                let code = user_err
                    .map(|e| e.code.clone())
                    .unwrap_or_else(|| "E_UNEXPECTED".to_string());
                let message = user_err
                    .map(|e| e.message.clone())
                    .unwrap_or_else(|| err.to_string());
                let details = user_err.and_then(|e| e.details.clone());
                let envelope = envelope_error("evolve.propose", &code, &message, details);
                let text = serde_json::to_string_pretty(&envelope)?;
                return Ok((text, envelope));
            }
        };

        let result = crate::handlers::evolve::evolve_propose_in(
            &engine,
            crate::handlers::evolve::EvolveProposeInput {
                profile,
                target_filter: target,
                action_prefix: &action_prefix,
                module_filter: args.module_id.as_deref(),
                scope,
                branch_override: args.branch.as_deref(),
                dry_run,
                confirmed: args.yes,
                json: true,
            },
        );

        let (text, envelope) = match result {
            Ok(crate::handlers::evolve::EvolveProposeOutcome::Noop(report)) => {
                let mut envelope = crate::output::JsonEnvelope::ok(
                    "evolve.propose",
                    serde_json::json!({
                        "created": false,
                        "reason": report.reason,
                        "summary": report.summary,
                        "skipped": report.skipped,
                    }),
                );
                envelope.warnings = report.warnings;
                let text = serde_json::to_string_pretty(&envelope)?;
                let envelope = serde_json::to_value(&envelope)?;
                (text, envelope)
            }
            Ok(crate::handlers::evolve::EvolveProposeOutcome::DryRun(report)) => {
                let mut envelope = crate::output::JsonEnvelope::ok(
                    "evolve.propose",
                    serde_json::json!({
                        "created": false,
                        "reason": "dry_run",
                        "candidates": report.candidates,
                        "skipped": report.skipped,
                        "summary": report.summary,
                    }),
                );
                envelope.warnings = report.warnings;
                let text = serde_json::to_string_pretty(&envelope)?;
                let envelope = serde_json::to_value(&envelope)?;
                (text, envelope)
            }
            Ok(crate::handlers::evolve::EvolveProposeOutcome::Created(report)) => {
                let envelope = crate::output::JsonEnvelope::ok(
                    "evolve.propose",
                    serde_json::json!({
                        "created": true,
                        "branch": report.branch,
                        "scope": report.scope,
                        "files": report.files,
                        "files_posix": report.files_posix,
                        "committed": report.committed,
                    }),
                );
                let text = serde_json::to_string_pretty(&envelope)?;
                let envelope = serde_json::to_value(&envelope)?;
                (text, envelope)
            }
            Ok(crate::handlers::evolve::EvolveProposeOutcome::NeedsConfirmation) => {
                let err = UserError::confirm_required("evolve propose");
                let user_err = err.chain().find_map(|e| e.downcast_ref::<UserError>());
                let code = user_err
                    .map(|e| e.code.clone())
                    .unwrap_or_else(|| "E_UNEXPECTED".to_string());
                let message = user_err
                    .map(|e| e.message.clone())
                    .unwrap_or_else(|| err.to_string());
                let details = user_err.and_then(|e| e.details.clone());
                let envelope = envelope_error("evolve.propose", &code, &message, details);
                let text = serde_json::to_string_pretty(&envelope)?;
                (text, envelope)
            }
            Err(err) => {
                let user_err = err.chain().find_map(|e| e.downcast_ref::<UserError>());
                let code = user_err
                    .map(|e| e.code.clone())
                    .unwrap_or_else(|| "E_UNEXPECTED".to_string());
                let message = user_err
                    .map(|e| e.message.clone())
                    .unwrap_or_else(|| err.to_string());
                let details = user_err.and_then(|e| e.details.clone());
                let envelope = envelope_error("evolve.propose", &code, &message, details);
                let text = serde_json::to_string_pretty(&envelope)?;
                (text, envelope)
            }
        };

        Ok((text, envelope))
    })
    .await
    .context("mcp evolve_propose handler task join")?
}

async fn call_preview_in_process(args: PreviewArgs) -> anyhow::Result<(String, serde_json::Value)> {
    tokio::task::spawn_blocking(move || {
        let repo_override = args.common.repo.as_ref().map(std::path::PathBuf::from);
        let profile = args.common.profile.as_deref().unwrap_or("default");
        let target = args.common.target.as_deref().unwrap_or("all");
        let machine_override = args.common.machine.as_deref();

        let result = crate::handlers::read_only::read_only_context(
            repo_override.as_deref(),
            machine_override,
            profile,
            target,
        );

        let (text, envelope) = match result {
            Ok(crate::handlers::read_only::ReadOnlyContext {
                targets,
                desired,
                plan,
                mut warnings,
                roots,
            }) => {
                let data = crate::app::preview_json::preview_json_data(
                    profile,
                    targets,
                    plan,
                    desired,
                    roots,
                    args.diff,
                    &mut warnings,
                )?;

                let mut envelope = crate::output::JsonEnvelope::ok("preview", data);
                envelope.warnings = warnings;

                let text = serde_json::to_string_pretty(&envelope)?;
                let envelope = serde_json::to_value(&envelope)?;
                (text, envelope)
            }
            Err(err) => {
                let user_err = err.chain().find_map(|e| e.downcast_ref::<UserError>());
                let code = user_err
                    .map(|e| e.code.clone())
                    .unwrap_or_else(|| "E_UNEXPECTED".to_string());
                let message = user_err
                    .map(|e| e.message.clone())
                    .unwrap_or_else(|| err.to_string());
                let details = user_err.and_then(|e| e.details.clone());
                let envelope = envelope_error("preview", &code, &message, details);
                let text = serde_json::to_string_pretty(&envelope)?;
                (text, envelope)
            }
        };

        Ok((text, envelope))
    })
    .await
    .context("mcp preview handler task join")?
}

async fn deploy_plan_envelope_in_process(args: CommonArgs) -> anyhow::Result<serde_json::Value> {
    tokio::task::spawn_blocking(move || {
        let repo_override = args.repo.as_ref().map(std::path::PathBuf::from);
        let profile = args.profile.as_deref().unwrap_or("default");
        let target = args.target.as_deref().unwrap_or("all");
        let machine_override = args.machine.as_deref();

        let result = crate::handlers::read_only::read_only_context(
            repo_override.as_deref(),
            machine_override,
            profile,
            target,
        );

        match result {
            Ok(crate::handlers::read_only::ReadOnlyContext {
                targets,
                plan,
                warnings,
                ..
            }) => {
                let mut envelope = crate::output::JsonEnvelope::ok(
                    "deploy",
                    serde_json::json!({
                        "applied": false,
                        "profile": profile,
                        "targets": targets,
                        "changes": plan.changes,
                        "summary": plan.summary,
                    }),
                );
                envelope.warnings = warnings;
                serde_json::to_value(&envelope).context("serialize deploy envelope")
            }
            Err(err) => {
                let user_err = err.chain().find_map(|e| e.downcast_ref::<UserError>());
                let code = user_err
                    .map(|e| e.code.clone())
                    .unwrap_or_else(|| "E_UNEXPECTED".to_string());
                let message = user_err
                    .map(|e| e.message.clone())
                    .unwrap_or_else(|| err.to_string());
                let details = user_err.and_then(|e| e.details.clone());
                Ok(envelope_error("deploy", &code, &message, details))
            }
        }
    })
    .await
    .context("mcp deploy handler task join")?
}

async fn call_deploy_apply_in_process(
    args: DeployApplyArgs,
) -> anyhow::Result<(String, serde_json::Value)> {
    tokio::task::spawn_blocking(move || {
        let repo_override = args.common.repo.as_ref().map(std::path::PathBuf::from);
        let profile = args.common.profile.as_deref().unwrap_or("default");
        let target = args.common.target.as_deref().unwrap_or("all");
        let machine_override = args.common.machine.as_deref();

        let result = (|| -> anyhow::Result<(String, serde_json::Value)> {
            let engine = crate::engine::Engine::load(repo_override.as_deref(), machine_override)?;
            let crate::handlers::read_only::ReadOnlyContext {
                targets,
                desired,
                plan,
                warnings,
                roots,
            } = crate::handlers::read_only::read_only_context_in(&engine, profile, target)?;

            let will_apply = !args.common.dry_run.unwrap_or(false);
            if !will_apply {
                let mut envelope = crate::output::JsonEnvelope::ok(
                    "deploy",
                    serde_json::json!({
                        "applied": false,
                        "profile": profile,
                        "targets": targets,
                        "changes": plan.changes,
                        "summary": plan.summary,
                    }),
                );
                envelope.warnings = warnings;

                let text = serde_json::to_string_pretty(&envelope)?;
                let envelope = serde_json::to_value(&envelope)?;
                return Ok((text, envelope));
            }

            let adopt = args.adopt.unwrap_or(false);
            let outcome = crate::handlers::deploy::deploy_apply_in(
                &engine,
                &plan,
                &desired,
                &roots,
                adopt,
                args.yes,
                crate::handlers::deploy::ConfirmationStyle::JsonYes {
                    command_id: "deploy --apply",
                },
            )?;

            match outcome {
                crate::handlers::deploy::DeployApplyOutcome::NoChanges => {
                    let mut envelope = crate::output::JsonEnvelope::ok(
                        "deploy",
                        serde_json::json!({
                            "applied": false,
                            "reason": "no_changes",
                            "profile": profile,
                            "targets": targets,
                            "changes": plan.changes,
                            "summary": plan.summary,
                        }),
                    );
                    envelope.warnings = warnings;

                    let text = serde_json::to_string_pretty(&envelope)?;
                    let envelope = serde_json::to_value(&envelope)?;
                    Ok((text, envelope))
                }
                crate::handlers::deploy::DeployApplyOutcome::Applied { snapshot_id } => {
                    let mut envelope = crate::output::JsonEnvelope::ok(
                        "deploy",
                        serde_json::json!({
                            "applied": true,
                            "snapshot_id": snapshot_id,
                            "profile": profile,
                            "targets": targets,
                            "changes": plan.changes,
                            "summary": plan.summary,
                        }),
                    );
                    envelope.warnings = warnings;

                    let text = serde_json::to_string_pretty(&envelope)?;
                    let envelope = serde_json::to_value(&envelope)?;
                    Ok((text, envelope))
                }
                crate::handlers::deploy::DeployApplyOutcome::NeedsConfirmation => {
                    anyhow::bail!(
                        "deploy apply requires confirmation, but confirmation was not provided"
                    )
                }
            }
        })();

        match result {
            Ok(v) => Ok(v),
            Err(err) => {
                let user_err = err.chain().find_map(|e| e.downcast_ref::<UserError>());
                let code = user_err
                    .map(|e| e.code.clone())
                    .unwrap_or_else(|| "E_UNEXPECTED".to_string());
                let message = user_err
                    .map(|e| e.message.clone())
                    .unwrap_or_else(|| err.to_string());
                let details = user_err.and_then(|e| e.details.clone());
                let envelope = envelope_error("deploy", &code, &message, details);
                let text = serde_json::to_string_pretty(&envelope)?;
                Ok((text, envelope))
            }
        }
    })
    .await
    .context("mcp deploy_apply handler task join")?
}

fn tool_result_from_envelope(text: String, envelope: serde_json::Value) -> CallToolResult {
    let ok = envelope
        .get("ok")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    CallToolResult {
        content: vec![Content::text(text)],
        structured_content: Some(envelope),
        is_error: Some(!ok),
        meta: None,
    }
}

fn tool_result_from_user_error(command: &str, err: UserError) -> CallToolResult {
    CallToolResult::structured_error(envelope_error(
        command,
        &err.code,
        &err.message,
        err.details,
    ))
}

fn tool_input_schema<T: schemars::JsonSchema + 'static>() -> Arc<JsonObject> {
    rmcp::handler::server::tool::schema_for_type::<T>()
}

fn tool(
    name: &'static str,
    description: &'static str,
    input_schema: Arc<JsonObject>,
    read_only: bool,
) -> Tool {
    Tool::new(name, description, input_schema).annotate(
        ToolAnnotations::new()
            .read_only(read_only)
            .destructive(!read_only),
    )
}

fn deserialize_args<T: serde::de::DeserializeOwned>(
    args: Option<JsonObject>,
) -> Result<T, McpError> {
    let value = serde_json::Value::Object(args.unwrap_or_default());
    serde_json::from_value(value).map_err(|e| McpError::invalid_params(e.to_string(), None))
}

pub(super) fn tools() -> Vec<Tool> {
    vec![
        tool(
            "plan",
            "Compute plan (returns Agentpack JSON envelope).",
            tool_input_schema::<CommonArgs>(),
            true,
        ),
        tool(
            "diff",
            "Compute diff (returns Agentpack JSON envelope).",
            tool_input_schema::<CommonArgs>(),
            true,
        ),
        tool(
            "preview",
            "Preview plan (optionally include diff; returns Agentpack JSON envelope).",
            tool_input_schema::<PreviewArgs>(),
            true,
        ),
        tool(
            "status",
            "Compute drift/status (returns Agentpack JSON envelope).",
            tool_input_schema::<StatusArgs>(),
            true,
        ),
        tool(
            "doctor",
            "Run doctor checks (returns Agentpack JSON envelope; read-only).",
            tool_input_schema::<DoctorArgs>(),
            true,
        ),
        tool(
            "deploy",
            "Plan+diff (read-only; returns Agentpack JSON envelope).",
            tool_input_schema::<CommonArgs>(),
            true,
        ),
        tool(
            "deploy_apply",
            "Deploy with apply (requires yes=true).",
            tool_input_schema::<DeployApplyArgs>(),
            false,
        ),
        tool(
            "rollback",
            "Rollback to a snapshot id (requires yes=true).",
            tool_input_schema::<RollbackArgs>(),
            false,
        ),
        tool(
            "evolve_propose",
            "Propose overlay updates by capturing drifted outputs (requires yes=true when not dry_run).",
            tool_input_schema::<EvolveProposeArgs>(),
            false,
        ),
        tool(
            "evolve_restore",
            "Restore missing desired outputs (requires yes=true when not dry_run).",
            tool_input_schema::<EvolveRestoreArgs>(),
            false,
        ),
        tool(
            "explain",
            "Explain plan/diff/status provenance (returns Agentpack JSON envelope).",
            tool_input_schema::<ExplainArgs>(),
            true,
        ),
    ]
}

pub(super) async fn call_tool(
    server: &AgentpackMcp,
    request: CallToolRequestParam,
) -> Result<CallToolResult, McpError> {
    match request.name.as_ref() {
        "plan" => {
            let args = deserialize_args::<CommonArgs>(request.arguments)?;
            let (text, envelope) = match call_read_only_in_process("plan", args).await {
                Ok(v) => v,
                Err(err) => {
                    return Ok(CallToolResult::structured_error(envelope_error(
                        "plan",
                        "E_UNEXPECTED",
                        &err.to_string(),
                        None,
                    )));
                }
            };
            Ok(tool_result_from_envelope(text, envelope))
        }
        "diff" => {
            let args = deserialize_args::<CommonArgs>(request.arguments)?;
            let (text, envelope) = match call_read_only_in_process("diff", args).await {
                Ok(v) => v,
                Err(err) => {
                    return Ok(CallToolResult::structured_error(envelope_error(
                        "diff",
                        "E_UNEXPECTED",
                        &err.to_string(),
                        None,
                    )));
                }
            };
            Ok(tool_result_from_envelope(text, envelope))
        }
        "preview" => {
            let args = deserialize_args::<PreviewArgs>(request.arguments)?;
            match call_preview_in_process(args).await {
                Ok((text, envelope)) => Ok(tool_result_from_envelope(text, envelope)),
                Err(err) => Ok(CallToolResult::structured_error(envelope_error(
                    "preview",
                    "E_UNEXPECTED",
                    &err.to_string(),
                    None,
                ))),
            }
        }
        "status" => {
            let args = deserialize_args::<StatusArgs>(request.arguments)?;
            match call_status_in_process(args).await {
                Ok((text, envelope)) => Ok(tool_result_from_envelope(text, envelope)),
                Err(err) => Ok(CallToolResult::structured_error(envelope_error(
                    "status",
                    "E_UNEXPECTED",
                    &err.to_string(),
                    None,
                ))),
            }
        }
        "doctor" => {
            let args = deserialize_args::<DoctorArgs>(request.arguments)?;
            let (text, envelope) = match call_doctor_in_process(args).await {
                Ok(v) => v,
                Err(err) => {
                    return Ok(CallToolResult::structured_error(envelope_error(
                        "doctor",
                        "E_UNEXPECTED",
                        &err.to_string(),
                        None,
                    )));
                }
            };
            Ok(tool_result_from_envelope(text, envelope))
        }
        "deploy" => {
            let args = deserialize_args::<CommonArgs>(request.arguments)?;
            let binding = ConfirmTokenBinding::from(&args);
            match deploy_plan_envelope_in_process(args).await {
                Ok(mut envelope) => {
                    let plan_hash = match confirm::compute_confirm_plan_hash(&binding, &envelope) {
                        Ok(v) => v,
                        Err(err) => {
                            return Ok(CallToolResult::structured_error(envelope_error(
                                "deploy",
                                "E_UNEXPECTED",
                                &err.to_string(),
                                None,
                            )));
                        }
                    };

                    let token = match confirm::generate_confirm_token() {
                        Ok(v) => v,
                        Err(err) => {
                            return Ok(CallToolResult::structured_error(envelope_error(
                                "deploy",
                                "E_UNEXPECTED",
                                &err.to_string(),
                                None,
                            )));
                        }
                    };

                    let now = Instant::now();
                    {
                        let mut store = server
                            .confirm_tokens
                            .lock()
                            .unwrap_or_else(|e| e.into_inner());
                        confirm::insert_token(
                            &mut store,
                            token.clone(),
                            binding,
                            plan_hash.clone(),
                            now,
                        );
                    }

                    let expires_at_utc = time::OffsetDateTime::now_utc()
                        + time::Duration::seconds(
                            i64::try_from(CONFIRM_TOKEN_TTL.as_secs()).unwrap_or(i64::MAX),
                        );
                    let expires_at_utc = match expires_at_utc
                        .format(&time::format_description::well_known::Rfc3339)
                    {
                        Ok(v) => v,
                        Err(err) => {
                            return Ok(CallToolResult::structured_error(envelope_error(
                                "deploy",
                                "E_UNEXPECTED",
                                &err.to_string(),
                                None,
                            )));
                        }
                    };

                    let Some(data) = envelope.get_mut("data").and_then(|v| v.as_object_mut())
                    else {
                        return Ok(CallToolResult::structured_error(envelope_error(
                            "deploy",
                            "E_UNEXPECTED",
                            "agentpack deploy envelope missing data object",
                            None,
                        )));
                    };
                    data.insert(
                        "confirm_token".to_string(),
                        serde_json::Value::String(token),
                    );
                    data.insert(
                        "confirm_plan_hash".to_string(),
                        serde_json::Value::String(plan_hash),
                    );
                    data.insert(
                        "confirm_token_expires_at".to_string(),
                        serde_json::Value::String(expires_at_utc),
                    );

                    let text = match serde_json::to_string_pretty(&envelope) {
                        Ok(v) => v,
                        Err(err) => {
                            return Ok(CallToolResult::structured_error(envelope_error(
                                "deploy",
                                "E_UNEXPECTED",
                                &err.to_string(),
                                None,
                            )));
                        }
                    };
                    Ok(tool_result_from_envelope(text, envelope))
                }
                Err(err) => Ok(CallToolResult::structured_error(envelope_error(
                    "deploy",
                    "E_UNEXPECTED",
                    &err.to_string(),
                    None,
                ))),
            }
        }
        "deploy_apply" => {
            let args = deserialize_args::<DeployApplyArgs>(request.arguments)?;
            if !args.yes || args.common.dry_run.unwrap_or(false) {
                match call_deploy_apply_in_process(args).await {
                    Ok((text, envelope)) => Ok(tool_result_from_envelope(text, envelope)),
                    Err(err) => Ok(CallToolResult::structured_error(envelope_error(
                        "deploy",
                        "E_UNEXPECTED",
                        &err.to_string(),
                        None,
                    ))),
                }
            } else {
                let Some(token) = args
                    .confirm_token
                    .as_deref()
                    .filter(|t| !t.is_empty())
                    .map(ToOwned::to_owned)
                else {
                    return Ok(tool_result_from_user_error(
                        "deploy",
                        UserError::confirm_token_required(),
                    ));
                };

                let binding = ConfirmTokenBinding::from(&args.common);
                let now = Instant::now();
                let stored_plan_hash = {
                    let mut store = server
                        .confirm_tokens
                        .lock()
                        .unwrap_or_else(|e| e.into_inner());
                    match confirm::validate_token(&mut store, token.as_str(), &binding, now) {
                        Ok(v) => v,
                        Err(err) => return Ok(tool_result_from_user_error("deploy", err)),
                    }
                };

                let plan_env = match deploy_plan_envelope_in_process(CommonArgs {
                    repo: args.common.repo.clone(),
                    profile: args.common.profile.clone(),
                    target: args.common.target.clone(),
                    machine: args.common.machine.clone(),
                    dry_run: args.common.dry_run,
                })
                .await
                {
                    Ok(v) => v,
                    Err(err) => {
                        return Ok(CallToolResult::structured_error(envelope_error(
                            "deploy",
                            "E_UNEXPECTED",
                            &err.to_string(),
                            None,
                        )));
                    }
                };
                let current_plan_hash =
                    match confirm::compute_confirm_plan_hash(&binding, &plan_env) {
                        Ok(v) => v,
                        Err(err) => {
                            return Ok(CallToolResult::structured_error(envelope_error(
                                "deploy",
                                "E_UNEXPECTED",
                                &err.to_string(),
                                None,
                            )));
                        }
                    };

                if current_plan_hash != stored_plan_hash {
                    return Ok(tool_result_from_user_error(
                        "deploy",
                        UserError::confirm_token_mismatch().with_details(serde_json::json!({
                            "hint": "Re-run the deploy tool and ensure the apply uses the matching confirm_token.",
                            "confirm_plan_hash": current_plan_hash,
                            "expected_confirm_plan_hash": stored_plan_hash,
                        })),
                    ));
                }

                match call_deploy_apply_in_process(args).await {
                    Ok((text, envelope)) => {
                        if envelope
                            .get("ok")
                            .and_then(serde_json::Value::as_bool)
                            .unwrap_or(false)
                        {
                            let mut store = server
                                .confirm_tokens
                                .lock()
                                .unwrap_or_else(|e| e.into_inner());
                            confirm::consume_token(&mut store, &token);
                        }
                        Ok(tool_result_from_envelope(text, envelope))
                    }
                    Err(err) => Ok(CallToolResult::structured_error(envelope_error(
                        "deploy",
                        "E_UNEXPECTED",
                        &err.to_string(),
                        None,
                    ))),
                }
            }
        }
        "rollback" => {
            let args = deserialize_args::<RollbackArgs>(request.arguments)?;
            match call_rollback_in_process(args).await {
                Ok((text, envelope)) => Ok(tool_result_from_envelope(text, envelope)),
                Err(err) => Ok(CallToolResult::structured_error(envelope_error(
                    "rollback",
                    "E_UNEXPECTED",
                    &err.to_string(),
                    None,
                ))),
            }
        }
        "evolve_propose" => {
            let args = deserialize_args::<EvolveProposeArgs>(request.arguments)?;
            match call_evolve_propose_in_process(args).await {
                Ok((text, envelope)) => Ok(tool_result_from_envelope(text, envelope)),
                Err(err) => Ok(CallToolResult::structured_error(envelope_error(
                    "evolve.propose",
                    "E_UNEXPECTED",
                    &err.to_string(),
                    None,
                ))),
            }
        }
        "evolve_restore" => {
            let args = deserialize_args::<EvolveRestoreArgs>(request.arguments)?;
            match call_evolve_restore_in_process(args).await {
                Ok((text, envelope)) => Ok(tool_result_from_envelope(text, envelope)),
                Err(err) => Ok(CallToolResult::structured_error(envelope_error(
                    "evolve.restore",
                    "E_UNEXPECTED",
                    &err.to_string(),
                    None,
                ))),
            }
        }
        "explain" => {
            let args = deserialize_args::<ExplainArgs>(request.arguments)?;
            match call_explain_in_process(args).await {
                Ok((text, envelope)) => Ok(tool_result_from_envelope(text, envelope)),
                Err(err) => Ok(CallToolResult::structured_error(envelope_error(
                    "explain",
                    "E_UNEXPECTED",
                    &err.to_string(),
                    None,
                ))),
            }
        }
        other => Ok(CallToolResult {
            content: vec![Content::text(format!("unknown tool: {other}"))],
            structured_content: None,
            is_error: Some(true),
            meta: None,
        }),
    }
}
