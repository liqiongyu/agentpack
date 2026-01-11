use serde::Serialize;

const JSON_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Serialize)]
pub struct JsonError {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct JsonEnvelope<T>
where
    T: Serialize,
{
    pub schema_version: u32,
    pub ok: bool,
    pub command: String,
    pub version: String,
    pub data: T,
    pub warnings: Vec<String>,
    pub errors: Vec<JsonError>,
}

impl<T> JsonEnvelope<T>
where
    T: Serialize,
{
    pub fn ok(command: impl Into<String>, data: T) -> Self {
        Self {
            schema_version: JSON_SCHEMA_VERSION,
            ok: true,
            command: command.into(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            data,
            warnings: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn err(command: impl Into<String>, errors: Vec<JsonError>) -> Self
    where
        T: Default,
    {
        Self {
            schema_version: JSON_SCHEMA_VERSION,
            ok: false,
            command: command.into(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            data: T::default(),
            warnings: Vec::new(),
            errors,
        }
    }
}

pub fn print_json<T>(envelope: &JsonEnvelope<T>) -> anyhow::Result<()>
where
    T: Serialize,
{
    println!("{}", serde_json::to_string_pretty(envelope)?);
    Ok(())
}
