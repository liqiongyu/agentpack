use std::future::Future;
use std::sync::Arc;

use rmcp::{
    ErrorData as McpError, RoleServer, ServerHandler,
    model::{
        CallToolRequestParam, CallToolResult, Content, Implementation, JsonObject, ListToolsResult,
        PaginatedRequestParam, ProtocolVersion, ServerCapabilities, ServerInfo, Tool,
        ToolAnnotations,
    },
    service::RequestContext,
};

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
pub struct CommonArgs {
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
pub struct StatusArgs {
    #[serde(flatten)]
    pub common: CommonArgs,
    #[serde(default)]
    pub only: Option<Vec<StatusOnly>>,
}

#[derive(Debug, Clone, Copy, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum StatusOnly {
    Missing,
    Modified,
    Extra,
}

#[derive(Debug, Default, serde::Deserialize, schemars::JsonSchema)]
pub struct DoctorArgs {
    #[serde(default)]
    pub repo: Option<String>,
    #[serde(default)]
    pub target: Option<String>,
}

#[derive(Debug, Default, serde::Deserialize, schemars::JsonSchema)]
pub struct DeployApplyArgs {
    #[serde(flatten)]
    pub common: CommonArgs,
    #[serde(default)]
    pub adopt: Option<bool>,
    #[serde(default)]
    pub yes: bool,
}

#[derive(Debug, Default, serde::Deserialize, schemars::JsonSchema)]
pub struct RollbackArgs {
    #[serde(default)]
    pub repo: Option<String>,
    pub to: String,
    #[serde(default)]
    pub yes: bool,
}

#[derive(Clone, Default)]
pub struct AgentpackMcp;

impl AgentpackMcp {
    pub fn new() -> Self {
        Self
    }
}

fn tool_input_schema<T: schemars::JsonSchema + 'static>() -> Arc<JsonObject> {
    Arc::new(rmcp::handler::server::tool::schema_for_type::<T>())
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

fn deserialize_args<T: serde::de::DeserializeOwned + Default>(
    args: Option<JsonObject>,
) -> Result<T, McpError> {
    let value = serde_json::Value::Object(args.unwrap_or_default());
    serde_json::from_value(value).map_err(|e| McpError::invalid_params(e.to_string(), None))
}

impl ServerHandler for AgentpackMcp {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2025_06_18,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: env!("CARGO_CRATE_NAME").to_string(),
                title: None,
                version: env!("CARGO_PKG_VERSION").to_string(),
                icons: None,
                website_url: None,
            },
            instructions: Some(
                "Agentpack MCP server (stdio). Tools: plan, diff, status, doctor, deploy_apply, rollback."
                    .to_string(),
            ),
        }
    }

    fn list_tools(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ListToolsResult, McpError>> + Send + '_ {
        std::future::ready(Ok(ListToolsResult {
            tools: vec![
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
            ],
            next_cursor: None,
            meta: None,
        }))
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        match request.name.as_ref() {
            "plan" => Ok(CallToolResult::structured_error(envelope_error(
                "plan",
                "E_UNEXPECTED",
                "mcp tool not implemented yet",
                None,
            ))),
            "diff" => Ok(CallToolResult::structured_error(envelope_error(
                "diff",
                "E_UNEXPECTED",
                "mcp tool not implemented yet",
                None,
            ))),
            "status" => Ok(CallToolResult::structured_error(envelope_error(
                "status",
                "E_UNEXPECTED",
                "mcp tool not implemented yet",
                None,
            ))),
            "doctor" => Ok(CallToolResult::structured_error(envelope_error(
                "doctor",
                "E_UNEXPECTED",
                "mcp tool not implemented yet",
                None,
            ))),
            "deploy_apply" => {
                let args = deserialize_args::<DeployApplyArgs>(request.arguments)?;
                if !args.yes {
                    return Ok(CallToolResult::structured_error(envelope_error(
                        "deploy",
                        "E_CONFIRM_REQUIRED",
                        "approval required: pass yes=true",
                        Some(serde_json::json!({ "tool": "deploy_apply" })),
                    )));
                }
                Ok(CallToolResult::structured_error(envelope_error(
                    "deploy",
                    "E_UNEXPECTED",
                    "mcp tool not implemented yet",
                    None,
                )))
            }
            "rollback" => {
                let args = deserialize_args::<RollbackArgs>(request.arguments)?;
                if !args.yes {
                    return Ok(CallToolResult::structured_error(envelope_error(
                        "rollback",
                        "E_CONFIRM_REQUIRED",
                        "approval required: pass yes=true",
                        Some(serde_json::json!({ "tool": "rollback", "to": args.to })),
                    )));
                }
                Ok(CallToolResult::structured_error(envelope_error(
                    "rollback",
                    "E_UNEXPECTED",
                    "mcp tool not implemented yet",
                    None,
                )))
            }
            other => Ok(CallToolResult {
                content: vec![Content::text(format!("unknown tool: {other}"))],
                structured_content: None,
                is_error: Some(true),
                meta: None,
            }),
        }
    }
}
