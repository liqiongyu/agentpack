use crate::app::preview_diff::preview_diff_files;

pub(crate) fn preview_json_data(
    profile: &str,
    targets: Vec<String>,
    plan: crate::deploy::PlanResult,
    desired: crate::deploy::DesiredState,
    roots: Vec<crate::targets::TargetRoot>,
    diff: bool,
    warnings: &mut Vec<String>,
) -> anyhow::Result<serde_json::Value> {
    if !diff {
        return Ok(serde_json::json!({
            "profile": profile,
            "targets": targets,
            "plan": {
                "changes": plan.changes,
                "summary": plan.summary,
            },
        }));
    }

    let plan_changes = plan.changes.clone();
    let plan_summary = plan.summary.clone();

    let files = preview_diff_files(&plan, &desired, &roots, warnings)?;

    let mut data = serde_json::json!({
        "profile": profile,
        "targets": targets,
        "plan": {
            "changes": plan_changes,
            "summary": plan_summary,
        },
    });

    data["diff"] = serde_json::json!({
        "changes": plan.changes,
        "summary": plan.summary,
        "files": files,
    });

    Ok(data)
}
