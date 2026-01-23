use rmcp::{
    ErrorData as McpError,
    model::{CallToolRequestParam, CallToolResult, Content, Tool},
};

use super::confirm::{CONFIRM_TOKEN_TTL, ConfirmTokenBinding};
use super::{AgentpackMcp, confirm};

mod deploy;
mod deploy_apply;
mod deploy_plan;
mod doctor;
mod envelope;
mod evolve_propose;
mod evolve_restore;
mod explain;
mod preview;
mod read_only;
mod rollback;
mod status;
mod tool_schema;

use deploy_plan::deploy_plan_envelope_in_process;
use envelope::{envelope_error, tool_result_from_envelope, tool_result_from_user_error};
use tool_schema::{deserialize_args, tool, tool_input_schema};

pub(super) const TOOLS_INSTRUCTIONS: &str = "Agentpack MCP server (stdio). Tools: plan, diff, preview, status, doctor, deploy, deploy_apply, rollback, evolve_propose, evolve_restore, explain.";

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

async fn call_doctor_in_process(args: DoctorArgs) -> anyhow::Result<(String, serde_json::Value)> {
    doctor::call_doctor_in_process(args).await
}

async fn call_status_in_process(args: StatusArgs) -> anyhow::Result<(String, serde_json::Value)> {
    status::call_status_in_process(args).await
}

async fn call_explain_in_process(args: ExplainArgs) -> anyhow::Result<(String, serde_json::Value)> {
    explain::call_explain_in_process(args).await
}

async fn call_rollback_in_process(
    args: RollbackArgs,
) -> anyhow::Result<(String, serde_json::Value)> {
    rollback::call_rollback_in_process(args).await
}

async fn call_evolve_restore_in_process(
    args: EvolveRestoreArgs,
) -> anyhow::Result<(String, serde_json::Value)> {
    evolve_restore::call_evolve_restore_in_process(args).await
}

async fn call_evolve_propose_in_process(
    args: EvolveProposeArgs,
) -> anyhow::Result<(String, serde_json::Value)> {
    evolve_propose::call_evolve_propose_in_process(args).await
}

async fn call_preview_in_process(args: PreviewArgs) -> anyhow::Result<(String, serde_json::Value)> {
    preview::call_preview_in_process(args).await
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
            let (text, envelope) = match read_only::call_read_only_in_process("plan", args).await {
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
            let (text, envelope) = match read_only::call_read_only_in_process("diff", args).await {
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
            Ok(deploy::call_deploy_tool(server, args).await)
        }
        "deploy_apply" => {
            let args = deserialize_args::<DeployApplyArgs>(request.arguments)?;
            Ok(deploy_apply::call_deploy_apply_tool(server, args).await)
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
