use rmcp::{
    ErrorData as McpError,
    model::{CallToolRequestParam, CallToolResult, Content},
};

use super::tool_schema::deserialize_args;
use super::{tool_result_from_envelope, tool_result_unexpected};

pub(super) async fn call_tool(
    server: &super::AgentpackMcp,
    request: CallToolRequestParam,
) -> Result<CallToolResult, McpError> {
    match request.name.as_ref() {
        "plan" => {
            let args = deserialize_args::<super::CommonArgs>(request.arguments)?;
            let (text, envelope) =
                match super::read_only::call_read_only_in_process("plan", args).await {
                    Ok(v) => v,
                    Err(err) => {
                        return Ok(tool_result_unexpected("plan", &err));
                    }
                };
            Ok(tool_result_from_envelope(text, envelope))
        }
        "diff" => {
            let args = deserialize_args::<super::CommonArgs>(request.arguments)?;
            let (text, envelope) =
                match super::read_only::call_read_only_in_process("diff", args).await {
                    Ok(v) => v,
                    Err(err) => {
                        return Ok(tool_result_unexpected("diff", &err));
                    }
                };
            Ok(tool_result_from_envelope(text, envelope))
        }
        "preview" => {
            let args = deserialize_args::<super::PreviewArgs>(request.arguments)?;
            match super::preview::call_preview_in_process(args).await {
                Ok((text, envelope)) => Ok(tool_result_from_envelope(text, envelope)),
                Err(err) => Ok(tool_result_unexpected("preview", &err)),
            }
        }
        "status" => {
            let args = deserialize_args::<super::StatusArgs>(request.arguments)?;
            match super::status::call_status_in_process(args).await {
                Ok((text, envelope)) => Ok(tool_result_from_envelope(text, envelope)),
                Err(err) => Ok(tool_result_unexpected("status", &err)),
            }
        }
        "doctor" => {
            let args = deserialize_args::<super::DoctorArgs>(request.arguments)?;
            let (text, envelope) = match super::doctor::call_doctor_in_process(args).await {
                Ok(v) => v,
                Err(err) => {
                    return Ok(tool_result_unexpected("doctor", &err));
                }
            };
            Ok(tool_result_from_envelope(text, envelope))
        }
        "deploy" => {
            let args = deserialize_args::<super::CommonArgs>(request.arguments)?;
            Ok(super::deploy::call_deploy_tool(server, args).await)
        }
        "deploy_apply" => {
            let args = deserialize_args::<super::DeployApplyArgs>(request.arguments)?;
            Ok(super::deploy_apply::call_deploy_apply_tool(server, args).await)
        }
        "rollback" => {
            let args = deserialize_args::<super::RollbackArgs>(request.arguments)?;
            match super::rollback::call_rollback_in_process(args).await {
                Ok((text, envelope)) => Ok(tool_result_from_envelope(text, envelope)),
                Err(err) => Ok(tool_result_unexpected("rollback", &err)),
            }
        }
        "evolve_propose" => {
            let args = deserialize_args::<super::EvolveProposeArgs>(request.arguments)?;
            match super::evolve_propose::call_evolve_propose_in_process(args).await {
                Ok((text, envelope)) => Ok(tool_result_from_envelope(text, envelope)),
                Err(err) => Ok(tool_result_unexpected("evolve.propose", &err)),
            }
        }
        "evolve_restore" => {
            let args = deserialize_args::<super::EvolveRestoreArgs>(request.arguments)?;
            match super::evolve_restore::call_evolve_restore_in_process(args).await {
                Ok((text, envelope)) => Ok(tool_result_from_envelope(text, envelope)),
                Err(err) => Ok(tool_result_unexpected("evolve.restore", &err)),
            }
        }
        "explain" => {
            let args = deserialize_args::<super::ExplainArgs>(request.arguments)?;
            match super::explain::call_explain_in_process(args).await {
                Ok((text, envelope)) => Ok(tool_result_from_envelope(text, envelope)),
                Err(err) => Ok(tool_result_unexpected("explain", &err)),
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
