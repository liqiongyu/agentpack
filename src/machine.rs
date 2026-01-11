use std::process::Command;

use anyhow::Context as _;

pub fn detect_machine_id() -> anyhow::Result<String> {
    if let Ok(val) = std::env::var("AGENTPACK_MACHINE_ID") {
        let id = normalize_machine_id(&val);
        if !id.is_empty() {
            return Ok(id);
        }
    }

    if let Ok(val) = std::env::var("HOSTNAME") {
        let id = normalize_machine_id(&val);
        if !id.is_empty() {
            return Ok(id);
        }
    }

    if let Ok(val) = std::env::var("COMPUTERNAME") {
        let id = normalize_machine_id(&val);
        if !id.is_empty() {
            return Ok(id);
        }
    }

    let out = Command::new("hostname").output().context("run hostname")?;
    if out.status.success() {
        let raw = String::from_utf8_lossy(&out.stdout);
        let id = normalize_machine_id(&raw);
        if !id.is_empty() {
            return Ok(id);
        }
    }

    Ok("unknown".to_string())
}

pub fn normalize_machine_id(s: &str) -> String {
    let raw = s.trim().to_lowercase();
    let mut out = String::new();
    let mut last_dash = false;
    for ch in raw.chars() {
        let ok = ch.is_ascii_alphanumeric() || ch == '-' || ch == '_';
        let normalized = if ok {
            ch
        } else if !last_dash {
            '-'
        } else {
            continue;
        };

        last_dash = normalized == '-';
        out.push(normalized);
    }
    out.trim_matches('-').to_string()
}
