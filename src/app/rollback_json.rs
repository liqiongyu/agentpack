pub(crate) fn rollback_json_data(
    rolled_back_to: &str,
    event_snapshot_id: &str,
) -> serde_json::Value {
    serde_json::json!({
        "rolled_back_to": rolled_back_to,
        "event_snapshot_id": event_snapshot_id,
    })
}
