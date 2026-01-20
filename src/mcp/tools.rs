use std::sync::Arc;
use std::time::Instant;

use anyhow::Context as _;
use rmcp::{
    ErrorData as McpError,
    model::{CallToolRequestParam, CallToolResult, Content, JsonObject, Tool, ToolAnnotations},
};

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

fn append_common_flags(args: &mut Vec<String>, common: &CommonArgs) {
    if let Some(repo) = &common.repo {
        args.push("--repo".to_string());
        args.push(repo.clone());
    }
    if let Some(profile) = &common.profile {
        args.push("--profile".to_string());
        args.push(profile.clone());
    }
    if let Some(target) = &common.target {
        args.push("--target".to_string());
        args.push(target.clone());
    }
    if let Some(machine) = &common.machine {
        args.push("--machine".to_string());
        args.push(machine.clone());
    }
    if common.dry_run.unwrap_or(false) {
        args.push("--dry-run".to_string());
    }
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
                let mut envelope = crate::output::JsonEnvelope::ok(
                    command,
                    serde_json::json!({
                        "profile": profile,
                        "targets": targets,
                        "changes": plan.changes,
                        "summary": plan.summary,
                    }),
                );
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

        #[derive(Default)]
        struct NextActions {
            json: std::collections::BTreeSet<String>,
        }

        let mut next_actions = NextActions::default();
        for c in &report.roots {
            if let Some(suggestion) = &c.suggestion {
                if let Some((_, cmd)) = suggestion.split_once(':') {
                    let cmd = cmd.trim();
                    if !cmd.is_empty() {
                        next_actions.json.insert(cmd.to_string());
                    }
                }
            }
        }

        if report.needs_gitignore_fix {
            let prefix = action_prefix(args.repo.as_deref(), target);
            next_actions
                .json
                .insert(format!("{prefix} doctor --fix --yes --json"));
        }

        let crate::handlers::doctor::DoctorReport {
            machine_id,
            roots,
            gitignore_fixes,
            warnings,
            ..
        } = report;

        let mut data = serde_json::json!({
            "machine_id": machine_id,
            "roots": roots,
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

        let mut envelope = crate::output::JsonEnvelope::ok("doctor", data);
        envelope.warnings = warnings;

        let text = serde_json::to_string_pretty(&envelope)?;
        let envelope = serde_json::to_value(&envelope)?;
        Ok((text, envelope))
    })
    .await
    .context("mcp doctor handler task join")?
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
                let envelope = crate::output::JsonEnvelope::ok(
                    "rollback",
                    serde_json::json!({
                        "rolled_back_to": snapshot_id,
                        "event_snapshot_id": event.id,
                    }),
                );
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

fn cli_args_for_status(status: &StatusArgs) -> Vec<String> {
    let mut args = vec!["--json".to_string()];
    append_common_flags(&mut args, &status.common);
    args.push("status".to_string());
    if let Some(only) = &status.only {
        let only = only
            .iter()
            .map(|k| match k {
                StatusOnly::Missing => "missing",
                StatusOnly::Modified => "modified",
                StatusOnly::Extra => "extra",
            })
            .collect::<Vec<_>>()
            .join(",");
        if !only.is_empty() {
            args.push("--only".to_string());
            args.push(only);
        }
    }
    args
}

fn cli_args_for_preview(preview: &PreviewArgs) -> Vec<String> {
    let mut args = vec!["--json".to_string()];
    append_common_flags(&mut args, &preview.common);
    args.push("preview".to_string());
    if preview.diff {
        args.push("--diff".to_string());
    }
    args
}

fn cli_args_for_deploy_apply(deploy: &DeployApplyArgs) -> Vec<String> {
    let mut args = vec!["--json".to_string()];
    if deploy.yes {
        args.push("--yes".to_string());
    }
    append_common_flags(&mut args, &deploy.common);
    args.push("deploy".to_string());
    args.push("--apply".to_string());
    if deploy.adopt.unwrap_or(false) {
        args.push("--adopt".to_string());
    }
    args
}

fn cli_args_for_evolve_propose(evolve: &EvolveProposeArgs) -> Vec<String> {
    let mut args = vec!["--json".to_string()];
    if evolve.yes {
        args.push("--yes".to_string());
    }
    append_common_flags(&mut args, &evolve.common);
    args.push("evolve".to_string());
    args.push("propose".to_string());

    if let Some(module_id) = &evolve.module_id {
        args.push("--module-id".to_string());
        args.push(module_id.clone());
    }
    if let Some(scope) = evolve.scope {
        args.push("--scope".to_string());
        args.push(
            match scope {
                EvolveScopeArg::Global => "global",
                EvolveScopeArg::Machine => "machine",
                EvolveScopeArg::Project => "project",
            }
            .to_string(),
        );
    }
    if let Some(branch) = &evolve.branch {
        args.push("--branch".to_string());
        args.push(branch.clone());
    }

    args
}

fn cli_args_for_explain(explain: &ExplainArgs) -> Vec<String> {
    let mut args = vec!["--json".to_string()];
    append_common_flags(&mut args, &explain.common);
    args.push("explain".to_string());
    args.push(
        match explain.kind {
            ExplainKindArg::Plan => "plan",
            ExplainKindArg::Diff => "diff",
            ExplainKindArg::Status => "status",
        }
        .to_string(),
    );
    args
}

async fn call_agentpack_json(args: Vec<String>) -> anyhow::Result<(String, serde_json::Value)> {
    tokio::task::spawn_blocking(move || {
        let exe = std::env::current_exe().context("resolve agentpack executable path")?;
        let output = std::process::Command::new(exe)
            .args(&args)
            .output()
            .with_context(|| format!("run agentpack {}", args.join(" ")))?;

        let stdout = String::from_utf8(output.stdout)
            .with_context(|| format!("agentpack {} stdout is not utf-8", args.join(" ")))?;
        let envelope: serde_json::Value = serde_json::from_str(&stdout).with_context(|| {
            format!(
                "parse agentpack {} stdout as json (exit={})",
                args.join(" "),
                output.status
            )
        })?;
        Ok((stdout, envelope))
    })
    .await
    .context("agentpack subprocess join")?
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
            match call_agentpack_json(cli_args_for_preview(&args)).await {
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
            match call_agentpack_json(cli_args_for_status(&args)).await {
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
                match call_agentpack_json(cli_args_for_deploy_apply(&args)).await {
                    Ok((text, envelope)) => Ok(tool_result_from_envelope(text, envelope)),
                    Err(err) => Ok(CallToolResult::structured_error(envelope_error(
                        "deploy",
                        "E_UNEXPECTED",
                        &err.to_string(),
                        None,
                    ))),
                }
            } else {
                let Some(token) = args.confirm_token.as_deref().filter(|t| !t.is_empty()) else {
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
                    match confirm::validate_token(&mut store, token, &binding, now) {
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

                match call_agentpack_json(cli_args_for_deploy_apply(&args)).await {
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
                            confirm::consume_token(&mut store, token);
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
            match call_agentpack_json(cli_args_for_evolve_propose(&args)).await {
                Ok((text, envelope)) => Ok(tool_result_from_envelope(text, envelope)),
                Err(err) => Ok(CallToolResult::structured_error(envelope_error(
                    "evolve",
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
            match call_agentpack_json(cli_args_for_explain(&args)).await {
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
