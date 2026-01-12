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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub module_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub success: Option<bool>,
    pub event: serde_json::Value,
}

pub struct ReadEventsResult {
    pub events: Vec<RecordedEvent>,
    pub warnings: Vec<String>,
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

pub fn event_module_id(event: &serde_json::Value) -> Option<String> {
    event
        .get("module_id")
        .or_else(|| event.get("moduleId"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

pub fn event_success(event: &serde_json::Value) -> Option<bool> {
    event.get("success").and_then(|v| v.as_bool()).or_else(|| {
        event
            .get("ok")
            .and_then(|v| v.as_bool())
            .or_else(|| event.get("successful").and_then(|v| v.as_bool()))
    })
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
        module_id: event_module_id(&event),
        success: event_success(&event),
        event,
    })
}

pub fn read_events(home: &AgentpackHome) -> anyhow::Result<Vec<RecordedEvent>> {
    Ok(read_events_with_warnings(home)?.events)
}

pub fn read_events_with_warnings(home: &AgentpackHome) -> anyhow::Result<ReadEventsResult> {
    let path = home.logs_dir.join(EVENTS_LOG_FILENAME);
    if !path.exists() {
        return Ok(ReadEventsResult {
            events: Vec::new(),
            warnings: Vec::new(),
        });
    }

    let f = std::fs::File::open(&path).with_context(|| format!("open {}", path.display()))?;
    let mut events = Vec::new();
    let mut warnings = Vec::new();

    use std::io::BufRead as _;
    let reader = std::io::BufReader::new(f);
    for (i, line) in reader.lines().enumerate() {
        let line_no = i + 1;
        let line = match line {
            Ok(s) => s,
            Err(err) => {
                warnings.push(format!(
                    "events.jsonl: failed to read line {line_no}: {err}"
                ));
                continue;
            }
        };

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let evt: RecordedEvent = match serde_json::from_str(trimmed) {
            Ok(v) => v,
            Err(err) => {
                warnings.push(format!(
                    "events.jsonl: skipped malformed JSON at line {line_no}: {err}"
                ));
                continue;
            }
        };

        if evt.schema_version != EVENTS_SCHEMA_VERSION {
            warnings.push(format!(
                "events.jsonl: skipped unsupported schema_version {} at line {line_no}",
                evt.schema_version
            ));
            continue;
        }

        events.push(evt);
    }

    Ok(ReadEventsResult { events, warnings })
}
