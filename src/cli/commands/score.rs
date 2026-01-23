use crate::config::Manifest;
use crate::output::{JsonEnvelope, print_json};

use super::Ctx;

pub(crate) fn run(ctx: &Ctx<'_>) -> anyhow::Result<()> {
    #[derive(Debug, Clone, serde::Serialize)]
    struct ModuleScore {
        module_id: String,
        total: u64,
        failures: u64,
        failure_rate: Option<f64>,
        last_seen_at: Option<String>,
    }

    let mut scores: std::collections::BTreeMap<
        String,
        (u64, u64, Option<String>), // total, failures, last_seen_at
    > = std::collections::BTreeMap::new();

    let read = crate::events::read_events_with_warnings(ctx.home)?;
    let crate::events::ReadEventsResult {
        events,
        warnings,
        stats,
    } = read;

    for evt in events {
        let Some(module_id) = evt
            .module_id
            .clone()
            .or_else(|| crate::events::event_module_id(&evt.event))
        else {
            continue;
        };
        let success = evt
            .success
            .or_else(|| crate::events::event_success(&evt.event))
            .unwrap_or(true);

        let entry = scores.entry(module_id).or_insert((0, 0, None));
        entry.0 += 1;
        if !success {
            entry.1 += 1;
        }
        let update_last = match &entry.2 {
            None => true,
            Some(prev) => prev < &evt.recorded_at,
        };
        if update_last {
            entry.2 = Some(evt.recorded_at);
        }
    }

    if let Ok(manifest) = Manifest::load(&ctx.repo.manifest_path) {
        for m in manifest.modules {
            scores.entry(m.id).or_insert((0, 0, None));
        }
    }

    let mut out: Vec<ModuleScore> = scores
        .into_iter()
        .map(|(module_id, (total, failures, last_seen_at))| ModuleScore {
            module_id,
            total,
            failures,
            failure_rate: if total == 0 {
                None
            } else {
                Some((failures as f64) / (total as f64))
            },
            last_seen_at,
        })
        .collect();
    out.sort_by(|a, b| {
        cmp_failure_rate(a.failures, a.total, b.failures, b.total)
            .then_with(|| a.module_id.cmp(&b.module_id))
    });

    if ctx.cli.json {
        let mut envelope = JsonEnvelope::ok(
            "score",
            serde_json::json!({
                "modules": out,
                "read_stats": stats,
            }),
        )
        .with_command_meta(ctx.cli.command_id(), ctx.cli.command_path());
        envelope.warnings = warnings;
        print_json(&envelope)?;
    } else if out.is_empty() {
        for w in &warnings {
            eprintln!("Warning: {w}");
        }
        if stats.skipped_total > 0 {
            eprintln!(
                "Warning: events.jsonl skipped lines: total={} malformed_json={} unsupported_schema_version={} read_errors={}",
                stats.skipped_total,
                stats.skipped_malformed_json,
                stats.skipped_unsupported_schema_version,
                stats.skipped_io_errors
            );
        }
        println!("No events recorded yet");
    } else {
        for w in &warnings {
            eprintln!("Warning: {w}");
        }
        if stats.skipped_total > 0 {
            eprintln!(
                "Warning: events.jsonl skipped lines: total={} malformed_json={} unsupported_schema_version={} read_errors={}",
                stats.skipped_total,
                stats.skipped_malformed_json,
                stats.skipped_unsupported_schema_version,
                stats.skipped_io_errors
            );
        }
        for s in out {
            let rate = s
                .failure_rate
                .map(|r| format!("{:.1}%", r * 100.0))
                .unwrap_or_else(|| "-".to_string());
            println!(
                "- {} failures={}/{} rate={} last_seen={}",
                s.module_id,
                s.failures,
                s.total,
                rate,
                s.last_seen_at.as_deref().unwrap_or("-")
            );
        }
    }

    Ok(())
}

fn cmp_failure_rate(a_fail: u64, a_total: u64, b_fail: u64, b_total: u64) -> std::cmp::Ordering {
    match (a_total == 0, b_total == 0) {
        (true, true) => std::cmp::Ordering::Equal,
        (true, false) => std::cmp::Ordering::Greater,
        (false, true) => std::cmp::Ordering::Less,
        (false, false) => {
            let left = (a_fail as u128) * (b_total as u128);
            let right = (b_fail as u128) * (a_total as u128);
            right.cmp(&left)
        }
    }
}
