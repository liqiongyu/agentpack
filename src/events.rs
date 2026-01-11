use std::io::Read as _;
use std::path::PathBuf;

use anyhow::Context as _;
use serde::{Deserialize, Serialize};

use crate::paths::AgentpackHome;

pub const EVENTS_LOG_FILENAME: &str = "events.jsonl";
const EVENTS_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordedEvent {
    pub schema_version: u32,
    pub recorded_at: String,
    pub machine_id: String,
    pub event: serde_json::Value,
}

pub fn read_stdin_event() -> anyhow::Result<serde_json::Value> {
    let mut input = String::new();
    std::io::stdin()
        .read_to_string(&mut input)
        .context("read stdin")?;
    let trimmed = input.trim();
    if trimmed.is_empty() {
        anyhow::bail!("no input on stdin (expected a JSON event)");
    }
    let event: serde_json::Value = serde_json::from_str(trimmed).context("parse JSON event")?;
    Ok(event)
}

pub fn append_event(home: &AgentpackHome, recorded: &RecordedEvent) -> anyhow::Result<PathBuf> {
    std::fs::create_dir_all(&home.logs_dir).context("create logs dir")?;
    let path = home.logs_dir.join(EVENTS_LOG_FILENAME);

    let mut line = serde_json::to_string(recorded).context("serialize event")?;
    line.push('\n');

    use std::io::Write as _;
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .with_context(|| format!("open {}", path.display()))?;
    f.write_all(line.as_bytes()).context("append event")?;

    Ok(path)
}

pub fn new_record(machine_id: String, event: serde_json::Value) -> anyhow::Result<RecordedEvent> {
    let recorded_at = time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .context("format timestamp")?;
    Ok(RecordedEvent {
        schema_version: EVENTS_SCHEMA_VERSION,
        recorded_at,
        machine_id,
        event,
    })
}

pub fn read_events(home: &AgentpackHome) -> anyhow::Result<Vec<RecordedEvent>> {
    let path = home.logs_dir.join(EVENTS_LOG_FILENAME);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let mut out = Vec::new();
    for (i, line) in raw.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let evt: RecordedEvent = serde_json::from_str(trimmed)
            .with_context(|| format!("parse {} line {}", path.display(), i + 1))?;
        if evt.schema_version != EVENTS_SCHEMA_VERSION {
            anyhow::bail!("unsupported events schema_version: {}", evt.schema_version);
        }
        out.push(evt);
    }
    Ok(out)
}
