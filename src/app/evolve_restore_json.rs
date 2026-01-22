pub(crate) fn evolve_restore_json_data(
    restored: Vec<crate::handlers::evolve::EvolveRestoreItem>,
    summary: crate::handlers::evolve::EvolveRestoreSummary,
    reason: &'static str,
) -> serde_json::Value {
    serde_json::json!({
        "restored": restored,
        "summary": summary,
        "reason": reason,
    })
}
