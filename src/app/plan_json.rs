pub(crate) fn plan_json_data(
    profile: &str,
    targets: Vec<String>,
    plan: crate::deploy::PlanResult,
) -> serde_json::Value {
    serde_json::json!({
        "profile": profile,
        "targets": targets,
        "changes": plan.changes,
        "summary": plan.summary,
    })
}
