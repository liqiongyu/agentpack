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
    pub command_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub git_rev: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub module_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub snapshot_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub success: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub targets: Option<Vec<String>>,
    pub event: serde_json::Value,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ReadEventsStats {
    pub lines_total: u64,
    pub lines_empty: u64,
    pub records_ok: u64,
    pub skipped_total: u64,
    pub skipped_io_errors: u64,
    pub skipped_malformed_json: u64,
    pub skipped_unsupported_schema_version: u64,
}

pub struct ReadEventsResult {
    pub events: Vec<RecordedEvent>,
    pub warnings: Vec<String>,
    pub stats: ReadEventsStats,
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

fn event_string_field(event: &serde_json::Value, keys: &[&str]) -> Option<String> {
    keys.iter()
        .find_map(|k| event.get(*k))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

fn event_u64_field(event: &serde_json::Value, keys: &[&str]) -> Option<u64> {
    keys.iter()
        .find_map(|k| event.get(*k))
        .and_then(|v| v.as_u64())
}

fn event_targets_field(event: &serde_json::Value) -> Option<Vec<String>> {
    if let Some(values) = event.get("targets").and_then(|v| v.as_array()) {
        let mut out: Vec<String> = values
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();
        out.sort();
        out.dedup();
        if !out.is_empty() {
            return Some(out);
        }
    }

    event
        .get("target")
        .and_then(|v| v.as_str())
        .map(|s| vec![s.to_string()])
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
        command_id: event_string_field(&event, &["command_id", "commandId"]),
        duration_ms: event_u64_field(&event, &["duration_ms", "durationMs"]),
        git_rev: event_string_field(&event, &["git_rev", "gitRev"]),
        module_id: event_module_id(&event),
        snapshot_id: event_string_field(&event, &["snapshot_id", "snapshotId"]),
        success: event_success(&event),
        targets: event_targets_field(&event),
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
            stats: ReadEventsStats::default(),
        });
    }

    let f = std::fs::File::open(&path).with_context(|| format!("open {}", path.display()))?;
    let mut events = Vec::new();
    let mut warnings = Vec::new();
    let mut stats = ReadEventsStats::default();

    use std::io::BufRead as _;
    let reader = std::io::BufReader::new(f);
    for (i, line) in reader.lines().enumerate() {
        let line_no = i + 1;
        stats.lines_total += 1;
        let line = match line {
            Ok(s) => s,
            Err(err) => {
                stats.skipped_total += 1;
                stats.skipped_io_errors += 1;
                warnings.push(format!(
                    "events.jsonl: failed to read line {line_no}: {err}"
                ));
                continue;
            }
        };

        let trimmed = line.trim();
        if trimmed.is_empty() {
            stats.lines_empty += 1;
            continue;
        }

        let evt: RecordedEvent = match serde_json::from_str(trimmed) {
            Ok(v) => v,
            Err(err) => {
                stats.skipped_total += 1;
                stats.skipped_malformed_json += 1;
                warnings.push(format!(
                    "events.jsonl: skipped malformed JSON at line {line_no}: {err}"
                ));
                continue;
            }
        };

        if evt.schema_version != EVENTS_SCHEMA_VERSION {
            stats.skipped_total += 1;
            stats.skipped_unsupported_schema_version += 1;
            warnings.push(format!(
                "events.jsonl: skipped unsupported schema_version {} at line {line_no}",
                evt.schema_version
            ));
            continue;
        }

        stats.records_ok += 1;
        events.push(evt);
    }

    Ok(ReadEventsResult {
        events,
        warnings,
        stats,
    })
}
