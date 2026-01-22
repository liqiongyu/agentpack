use anyhow::Context as _;

use crate::app::next_actions::ordered_next_actions_detailed;

pub(crate) fn status_json_data(
    profile: &str,
    targets: Vec<String>,
    drift: Vec<crate::handlers::status::DriftItem>,
    summary: crate::handlers::status::DriftSummary,
    summary_by_root: Vec<crate::app::status_drift::DriftSummaryByRoot>,
    summary_total_opt: Option<crate::handlers::status::DriftSummary>,
    next_actions: &std::collections::BTreeSet<String>,
) -> anyhow::Result<serde_json::Value> {
    let mut data = serde_json::json!({
        "profile": profile,
        "targets": targets,
        "drift": drift,
        "summary": summary,
        "summary_by_root": summary_by_root,
    });

    if let Some(summary_total) = summary_total_opt {
        data.as_object_mut()
            .context("status json data must be an object")?
            .insert(
                "summary_total".to_string(),
                serde_json::to_value(summary_total).context("serialize summary_total")?,
            );
    }

    if !next_actions.is_empty() {
        let (ordered, detailed) = ordered_next_actions_detailed(next_actions);
        data.as_object_mut()
            .context("status json data must be an object")?
            .insert(
                "next_actions".to_string(),
                serde_json::to_value(&ordered).context("serialize next_actions")?,
            );
        data.as_object_mut()
            .context("status json data must be an object")?
            .insert(
                "next_actions_detailed".to_string(),
                serde_json::to_value(&detailed).context("serialize next_actions_detailed")?,
            );
    }

    Ok(data)
}
