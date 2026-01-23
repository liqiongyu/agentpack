use rmcp::{
    ErrorData as McpError,
    model::{CallToolRequestParam, CallToolResult, Tool},
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
mod router;
mod status;
mod tool_registry;
mod tool_schema;

pub(super) use args::{
    CommonArgs, DeployApplyArgs, DoctorArgs, EvolveProposeArgs, EvolveRestoreArgs, EvolveScopeArg,
    ExplainArgs, ExplainKindArg, PreviewArgs, RollbackArgs, StatusArgs, StatusOnly,
};

use deploy_plan::deploy_plan_envelope_in_process;
use envelope::{
    envelope_from_anyhow_error, tool_result_from_envelope, tool_result_from_user_error,
    tool_result_unexpected,
};
use tool_schema::{tool, tool_input_schema};

pub(super) const TOOLS_INSTRUCTIONS: &str = "Agentpack MCP server (stdio). Tools: plan, diff, preview, status, doctor, deploy, deploy_apply, rollback, evolve_propose, evolve_restore, explain.";

pub(super) fn tools() -> Vec<Tool> {
    tool_registry::tools()
}

pub(super) async fn call_tool(
    server: &AgentpackMcp,
    request: CallToolRequestParam,
) -> Result<CallToolResult, McpError> {
    router::call_tool(server, request).await
}
