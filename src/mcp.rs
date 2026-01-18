use std::collections::HashMap;
use std::future::Future;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use anyhow::Context as _;
use rmcp::{
    ErrorData as McpError, RoleServer, ServerHandler,
    model::{
        CallToolRequestParam, CallToolResult, Content, Implementation, JsonObject, ListToolsResult,
        PaginatedRequestParam, ProtocolVersion, ServerCapabilities, ServerInfo, Tool,
        ToolAnnotations,
    },
    service::RequestContext,
};
use sha2::Digest as _;

use crate::user_error::UserError;

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

const CONFIRM_TOKEN_TTL: Duration = Duration::from_secs(10 * 60);
const CONFIRM_TOKEN_LEN_BYTES: usize = 32;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
struct ConfirmTokenBinding {
    repo: Option<String>,
    profile: Option<String>,
    target: Option<String>,
    machine: Option<String>,
}

impl From<&CommonArgs> for ConfirmTokenBinding {
    fn from(value: &CommonArgs) -> Self {
        Self {
            repo: value.repo.clone(),
            profile: value.profile.clone(),
            target: value.target.clone(),
            machine: value.machine.clone(),
        }
    }
}

#[derive(Debug, Clone)]
struct ConfirmTokenEntry {
    binding: ConfirmTokenBinding,
    plan_hash: String,
    expires_at: Instant,
}

#[derive(Debug, Default)]
struct ConfirmTokenStore {
    tokens: HashMap<String, ConfirmTokenEntry>,
}

impl ConfirmTokenStore {
    fn cleanup_expired(&mut self, now: Instant) {
        // Keep recently expired tokens for a short grace period so callers can get a more
        // actionable `E_CONFIRM_TOKEN_EXPIRED` instead of an unknown-token mismatch.
        self.tokens
            .retain(|_, entry| entry.expires_at + CONFIRM_TOKEN_TTL > now);
    }
}

#[derive(Debug, Default, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
pub struct DoctorArgs {
    #[serde(default)]
    pub repo: Option<String>,
    #[serde(default)]
    pub target: Option<String>,
}

#[derive(Debug, Default, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DeployApplyArgs {
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
pub struct RollbackArgs {
    #[serde(default)]
    pub repo: Option<String>,
    pub to: String,
    #[serde(default)]
    pub yes: bool,
}

#[derive(Debug, Default, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PreviewArgs {
    #[serde(flatten)]
    pub common: CommonArgs,
    #[serde(default)]
    pub diff: bool,
}

#[derive(Debug, Clone, Copy, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EvolveScopeArg {
    Global,
    Machine,
    Project,
}

#[derive(Debug, Default, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EvolveProposeArgs {
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
pub struct EvolveRestoreArgs {
    #[serde(flatten)]
    pub common: CommonArgs,
    #[serde(default)]
    pub module_id: Option<String>,
    #[serde(default)]
    pub yes: bool,
}

#[derive(Debug, Clone, Copy, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExplainKindArg {
    Plan,
    Diff,
    Status,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ExplainArgs {
    #[serde(flatten)]
    pub common: CommonArgs,
    pub kind: ExplainKindArg,
}

#[derive(Clone)]
pub struct AgentpackMcp {
    confirm_tokens: Arc<Mutex<ConfirmTokenStore>>,
}

impl AgentpackMcp {
    pub fn new() -> Self {
        Self {
            confirm_tokens: Arc::new(Mutex::new(ConfirmTokenStore::default())),
        }
    }
}

impl Default for AgentpackMcp {
    fn default() -> Self {
        Self::new()
    }
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

fn cli_args_for_plan(common: &CommonArgs) -> Vec<String> {
    let mut args = vec!["--json".to_string()];
    append_common_flags(&mut args, common);
    args.push("plan".to_string());
    args
}

fn cli_args_for_diff(common: &CommonArgs) -> Vec<String> {
    let mut args = vec!["--json".to_string()];
    append_common_flags(&mut args, common);
    args.push("diff".to_string());
    args
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

fn cli_args_for_deploy(common: &CommonArgs) -> Vec<String> {
    let mut args = vec!["--json".to_string()];
    append_common_flags(&mut args, common);
    args.push("deploy".to_string());
    args
}

fn cli_args_for_doctor(doctor: &DoctorArgs) -> Vec<String> {
    let mut args = vec!["--json".to_string()];
    if let Some(repo) = &doctor.repo {
        args.push("--repo".to_string());
        args.push(repo.clone());
    }
    if let Some(target) = &doctor.target {
        args.push("--target".to_string());
        args.push(target.clone());
    }
    args.push("doctor".to_string());
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

fn cli_args_for_rollback(rollback: &RollbackArgs) -> Vec<String> {
    let mut args = vec!["--json".to_string()];
    if rollback.yes {
        args.push("--yes".to_string());
    }
    if let Some(repo) = &rollback.repo {
        args.push("--repo".to_string());
        args.push(repo.clone());
    }
    args.push("rollback".to_string());
    args.push("--to".to_string());
    args.push(rollback.to.clone());
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

fn cli_args_for_evolve_restore(evolve: &EvolveRestoreArgs) -> Vec<String> {
    let mut args = vec!["--json".to_string()];
    if evolve.yes {
        args.push("--yes".to_string());
    }
    append_common_flags(&mut args, &evolve.common);
    args.push("evolve".to_string());
    args.push("restore".to_string());

    if let Some(module_id) = &evolve.module_id {
        args.push("--module-id".to_string());
        args.push(module_id.clone());
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

fn compute_confirm_plan_hash(
    binding: &ConfirmTokenBinding,
    envelope: &serde_json::Value,
) -> anyhow::Result<String> {
    let data = envelope
        .get("data")
        .cloned()
        .unwrap_or_else(|| serde_json::Value::Object(serde_json::Map::new()));
    let hash_input = serde_json::json!({
        "binding": binding,
        "data": data,
    });
    let bytes = serde_json::to_vec(&hash_input).context("serialize confirm_plan_hash input")?;
    Ok(hex::encode(sha2::Sha256::digest(bytes)))
}

fn generate_confirm_token() -> anyhow::Result<String> {
    let mut bytes = [0u8; CONFIRM_TOKEN_LEN_BYTES];
    getrandom::fill(&mut bytes).map_err(|e| anyhow::anyhow!("generate confirm_token: {e}"))?;
    Ok(hex::encode(bytes))
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

fn deserialize_args<T: serde::de::DeserializeOwned>(
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
                "Agentpack MCP server (stdio). Tools: plan, diff, preview, status, doctor, deploy, deploy_apply, rollback, evolve_propose, evolve_restore, explain."
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
            "plan" => {
                let args = deserialize_args::<CommonArgs>(request.arguments)?;
                match call_agentpack_json(cli_args_for_plan(&args)).await {
                    Ok((text, envelope)) => Ok(tool_result_from_envelope(text, envelope)),
                    Err(err) => Ok(CallToolResult::structured_error(envelope_error(
                        "plan",
                        "E_UNEXPECTED",
                        &err.to_string(),
                        None,
                    ))),
                }
            }
            "diff" => {
                let args = deserialize_args::<CommonArgs>(request.arguments)?;
                match call_agentpack_json(cli_args_for_diff(&args)).await {
                    Ok((text, envelope)) => Ok(tool_result_from_envelope(text, envelope)),
                    Err(err) => Ok(CallToolResult::structured_error(envelope_error(
                        "diff",
                        "E_UNEXPECTED",
                        &err.to_string(),
                        None,
                    ))),
                }
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
                match call_agentpack_json(cli_args_for_doctor(&args)).await {
                    Ok((text, envelope)) => Ok(tool_result_from_envelope(text, envelope)),
                    Err(err) => Ok(CallToolResult::structured_error(envelope_error(
                        "doctor",
                        "E_UNEXPECTED",
                        &err.to_string(),
                        None,
                    ))),
                }
            }
            "deploy" => {
                let args = deserialize_args::<CommonArgs>(request.arguments)?;
                match call_agentpack_json(cli_args_for_deploy(&args)).await {
                    Ok((_text, mut envelope)) => {
                        let binding = ConfirmTokenBinding::from(&args);
                        let plan_hash = match compute_confirm_plan_hash(&binding, &envelope) {
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

                        let token = match generate_confirm_token() {
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
                        let expires_at = now + CONFIRM_TOKEN_TTL;

                        {
                            let mut store = self
                                .confirm_tokens
                                .lock()
                                .unwrap_or_else(|e| e.into_inner());
                            store.cleanup_expired(now);
                            store.tokens.insert(
                                token.clone(),
                                ConfirmTokenEntry {
                                    binding: binding.clone(),
                                    plan_hash: plan_hash.clone(),
                                    expires_at,
                                },
                            );
                        }

                        let expires_at = time::OffsetDateTime::now_utc()
                            + time::Duration::seconds(
                                i64::try_from(CONFIRM_TOKEN_TTL.as_secs()).unwrap_or(i64::MAX),
                            );
                        let expires_at = match expires_at
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
                            serde_json::Value::String(expires_at),
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
                    let Some(token) = args.confirm_token.as_deref().filter(|t| !t.is_empty())
                    else {
                        return Ok(tool_result_from_user_error(
                            "deploy",
                            UserError::confirm_token_required(),
                        ));
                    };

                    let binding = ConfirmTokenBinding::from(&args.common);
                    let now = Instant::now();
                    let stored_plan_hash = {
                        let mut store = self
                            .confirm_tokens
                            .lock()
                            .unwrap_or_else(|e| e.into_inner());
                        let Some(entry) = store.tokens.get(token).cloned() else {
                            store.cleanup_expired(now);
                            return Ok(tool_result_from_user_error(
                                "deploy",
                                UserError::confirm_token_mismatch(),
                            ));
                        };
                        if entry.expires_at <= now {
                            store.tokens.remove(token);
                            store.cleanup_expired(now);
                            return Ok(tool_result_from_user_error(
                                "deploy",
                                UserError::confirm_token_expired(),
                            ));
                        }
                        if entry.binding != binding {
                            store.cleanup_expired(now);
                            return Ok(tool_result_from_user_error(
                                "deploy",
                                UserError::confirm_token_mismatch(),
                            ));
                        }

                        store.cleanup_expired(now);
                        entry.plan_hash
                    };

                    let (_plan_text, plan_env) =
                        match call_agentpack_json(cli_args_for_deploy(&args.common)).await {
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
                    let current_plan_hash = match compute_confirm_plan_hash(&binding, &plan_env) {
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
                                let mut store = self
                                    .confirm_tokens
                                    .lock()
                                    .unwrap_or_else(|e| e.into_inner());
                                store.tokens.remove(token);
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
                match call_agentpack_json(cli_args_for_rollback(&args)).await {
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
                match call_agentpack_json(cli_args_for_evolve_restore(&args)).await {
                    Ok((text, envelope)) => Ok(tool_result_from_envelope(text, envelope)),
                    Err(err) => Ok(CallToolResult::structured_error(envelope_error(
                        "evolve",
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
}
