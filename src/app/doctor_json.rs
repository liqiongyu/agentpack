use anyhow::Context as _;

use crate::app::next_actions::ordered_next_actions;

pub(crate) fn doctor_json_data(
    machine_id: String,
    roots: Vec<crate::handlers::doctor::DoctorRootCheck>,
    gitignore_fixes: Vec<crate::handlers::doctor::DoctorGitignoreFix>,
    next_actions: &std::collections::BTreeSet<String>,
) -> anyhow::Result<serde_json::Value> {
    let mut data = serde_json::json!({
        "machine_id": machine_id,
        "roots": roots,
        "gitignore_fixes": gitignore_fixes,
    });

    if !next_actions.is_empty() {
        let ordered = ordered_next_actions(next_actions);
        data.as_object_mut()
            .context("doctor json data must be an object")?
            .insert(
                "next_actions".to_string(),
                serde_json::to_value(&ordered).context("serialize next_actions")?,
            );
    }

    Ok(data)
}
