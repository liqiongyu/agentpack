#[derive(serde::Serialize)]
pub(crate) struct ExplainedModule {
    pub(crate) module_id: String,
    pub(crate) module_type: Option<String>,
    pub(crate) layer: Option<String>,
    pub(crate) module_path: Option<String>,
}

#[derive(serde::Serialize)]
pub(crate) struct ExplainedChange {
    pub(crate) op: String,
    pub(crate) target: String,
    pub(crate) path: String,
    pub(crate) path_posix: String,
    pub(crate) modules: Vec<ExplainedModule>,
}

#[derive(serde::Serialize)]
pub(crate) struct ExplainedDrift {
    pub(crate) kind: String,
    pub(crate) target: String,
    pub(crate) path: String,
    pub(crate) path_posix: String,
    pub(crate) expected: Option<String>,
    pub(crate) actual: Option<String>,
    pub(crate) modules: Vec<String>,
}

pub(crate) fn explain_plan_json_data(
    profile: &str,
    targets: Vec<String>,
    changes: Vec<ExplainedChange>,
) -> serde_json::Value {
    serde_json::json!({
        "profile": profile,
        "targets": targets,
        "changes": changes,
    })
}

pub(crate) fn explain_status_json_data(
    profile: &str,
    targets: Vec<String>,
    drift: Vec<ExplainedDrift>,
) -> serde_json::Value {
    serde_json::json!({
        "profile": profile,
        "targets": targets,
        "drift": drift,
    })
}
