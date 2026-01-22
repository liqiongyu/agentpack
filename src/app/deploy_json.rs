pub(crate) fn deploy_json_data_dry_run(
    profile: &str,
    targets: Vec<String>,
    plan: crate::deploy::PlanResult,
) -> serde_json::Value {
    serde_json::json!({
        "applied": false,
        "profile": profile,
        "targets": targets,
        "changes": plan.changes,
        "summary": plan.summary,
    })
}

pub(crate) fn deploy_json_data_no_changes(
    profile: &str,
    targets: Vec<String>,
    plan: crate::deploy::PlanResult,
) -> serde_json::Value {
    serde_json::json!({
        "applied": false,
        "reason": "no_changes",
        "profile": profile,
        "targets": targets,
        "changes": plan.changes,
        "summary": plan.summary,
    })
}

pub(crate) fn deploy_json_data_applied(
    profile: &str,
    targets: Vec<String>,
    plan: crate::deploy::PlanResult,
    snapshot_id: String,
) -> serde_json::Value {
    serde_json::json!({
        "applied": true,
        "snapshot_id": snapshot_id,
        "profile": profile,
        "targets": targets,
        "changes": plan.changes,
        "summary": plan.summary,
    })
}
