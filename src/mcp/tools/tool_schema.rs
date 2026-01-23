use std::sync::Arc;

use rmcp::ErrorData as McpError;
use rmcp::model::{JsonObject, Tool, ToolAnnotations};

pub(super) fn tool_input_schema<T: schemars::JsonSchema + 'static>() -> Arc<JsonObject> {
    rmcp::handler::server::tool::schema_for_type::<T>()
}

pub(super) fn tool(
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

pub(super) fn deserialize_args<T: serde::de::DeserializeOwned>(
    args: Option<JsonObject>,
) -> Result<T, McpError> {
    let value = serde_json::Value::Object(args.unwrap_or_default());
    serde_json::from_value(value).map_err(|e| McpError::invalid_params(e.to_string(), None))
}
