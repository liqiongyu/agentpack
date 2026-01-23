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
            let command_path = ["plan"];
            let meta = super::CommandMeta {
                command: "plan",
                command_id: "plan",
                command_path: &command_path,
            };
            let args = deserialize_args::<super::CommonArgs>(request.arguments)?;
            let (text, envelope) =
                match super::read_only::call_read_only_in_process("plan", args).await {
                    Ok(v) => v,
                    Err(err) => {
                        return Ok(tool_result_unexpected(meta, &err));
                    }
                };
            Ok(tool_result_from_envelope(text, envelope))
        }
        "diff" => {
            let command_path = ["diff"];
            let meta = super::CommandMeta {
                command: "diff",
                command_id: "diff",
                command_path: &command_path,
            };
            let args = deserialize_args::<super::CommonArgs>(request.arguments)?;
            let (text, envelope) =
                match super::read_only::call_read_only_in_process("diff", args).await {
                    Ok(v) => v,
                    Err(err) => {
                        return Ok(tool_result_unexpected(meta, &err));
                    }
                };
            Ok(tool_result_from_envelope(text, envelope))
        }
        "preview" => {
            let command_path = ["preview"];
            let meta = super::CommandMeta {
                command: "preview",
                command_id: "preview",
                command_path: &command_path,
            };
            let args = deserialize_args::<super::PreviewArgs>(request.arguments)?;
            match super::preview::call_preview_in_process(args).await {
                Ok((text, envelope)) => Ok(tool_result_from_envelope(text, envelope)),
                Err(err) => Ok(tool_result_unexpected(meta, &err)),
            }
        }
        "status" => {
            let command_path = ["status"];
            let meta = super::CommandMeta {
                command: "status",
                command_id: "status",
                command_path: &command_path,
            };
            let args = deserialize_args::<super::StatusArgs>(request.arguments)?;
            match super::status::call_status_in_process(args).await {
                Ok((text, envelope)) => Ok(tool_result_from_envelope(text, envelope)),
                Err(err) => Ok(tool_result_unexpected(meta, &err)),
            }
        }
        "doctor" => {
            let command_path = ["doctor"];
            let meta = super::CommandMeta {
                command: "doctor",
                command_id: "doctor",
                command_path: &command_path,
            };
            let args = deserialize_args::<super::DoctorArgs>(request.arguments)?;
            let (text, envelope) = match super::doctor::call_doctor_in_process(args).await {
                Ok(v) => v,
                Err(err) => {
                    return Ok(tool_result_unexpected(meta, &err));
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
            let command_path = ["rollback"];
            let meta = super::CommandMeta {
                command: "rollback",
                command_id: "rollback",
                command_path: &command_path,
            };
            let args = deserialize_args::<super::RollbackArgs>(request.arguments)?;
            match super::rollback::call_rollback_in_process(args).await {
                Ok((text, envelope)) => Ok(tool_result_from_envelope(text, envelope)),
                Err(err) => Ok(tool_result_unexpected(meta, &err)),
            }
        }
        "evolve_propose" => {
            let command_path = ["evolve", "propose"];
            let meta = super::CommandMeta {
                command: "evolve.propose",
                command_id: "evolve propose",
                command_path: &command_path,
            };
            let args = deserialize_args::<super::EvolveProposeArgs>(request.arguments)?;
            match super::evolve_propose::call_evolve_propose_in_process(args).await {
                Ok((text, envelope)) => Ok(tool_result_from_envelope(text, envelope)),
                Err(err) => Ok(tool_result_unexpected(meta, &err)),
            }
        }
        "evolve_restore" => {
            let command_path = ["evolve", "restore"];
            let meta = super::CommandMeta {
                command: "evolve.restore",
                command_id: "evolve restore",
                command_path: &command_path,
            };
            let args = deserialize_args::<super::EvolveRestoreArgs>(request.arguments)?;
            match super::evolve_restore::call_evolve_restore_in_process(args).await {
                Ok((text, envelope)) => Ok(tool_result_from_envelope(text, envelope)),
                Err(err) => Ok(tool_result_unexpected(meta, &err)),
            }
        }
        "explain" => {
            let args = deserialize_args::<super::ExplainArgs>(request.arguments)?;
            let (command, command_id, command_path) = match args.kind {
                super::ExplainKindArg::Plan => {
                    ("explain.plan", "explain plan", ["explain", "plan"])
                }
                super::ExplainKindArg::Diff => {
                    ("explain.plan", "explain diff", ["explain", "diff"])
                }
                super::ExplainKindArg::Status => {
                    ("explain.status", "explain status", ["explain", "status"])
                }
            };
            let meta = super::CommandMeta {
                command,
                command_id,
                command_path: &command_path,
            };
            match super::explain::call_explain_in_process(args).await {
                Ok((text, envelope)) => Ok(tool_result_from_envelope(text, envelope)),
                Err(err) => Ok(tool_result_unexpected(meta, &err)),
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
