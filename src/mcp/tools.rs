use rmcp::{
    ErrorData as McpError,
    model::{CallToolRequestParam, CallToolResult, Content, Tool},
};

use super::confirm::{CONFIRM_TOKEN_TTL, ConfirmTokenBinding};
use super::{AgentpackMcp, confirm};

mod args;
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
mod tool_registry;
mod tool_schema;

pub(super) use args::{
    CommonArgs, DeployApplyArgs, DoctorArgs, EvolveProposeArgs, EvolveRestoreArgs, EvolveScopeArg,
    ExplainArgs, ExplainKindArg, PreviewArgs, RollbackArgs, StatusArgs, StatusOnly,
};

use deploy_plan::deploy_plan_envelope_in_process;
use envelope::{envelope_error, tool_result_from_envelope, tool_result_from_user_error};
use tool_schema::{deserialize_args, tool, tool_input_schema};

pub(super) const TOOLS_INSTRUCTIONS: &str = "Agentpack MCP server (stdio). Tools: plan, diff, preview, status, doctor, deploy, deploy_apply, rollback, evolve_propose, evolve_restore, explain.";

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
    tool_registry::tools()
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
