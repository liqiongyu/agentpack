pub(crate) fn evolve_propose_json_data_noop(
    reason: &'static str,
    summary: crate::handlers::evolve::EvolveProposeSummary,
    skipped: Vec<crate::handlers::evolve::EvolveProposeSkippedItem>,
) -> serde_json::Value {
    serde_json::json!({
        "created": false,
        "reason": reason,
        "summary": summary,
        "skipped": skipped,
    })
}

pub(crate) fn evolve_propose_json_data_dry_run(
    candidates: Vec<crate::handlers::evolve::EvolveProposeItem>,
    skipped: Vec<crate::handlers::evolve::EvolveProposeSkippedItem>,
    summary: crate::handlers::evolve::EvolveProposeSummary,
) -> serde_json::Value {
    serde_json::json!({
        "created": false,
        "reason": "dry_run",
        "candidates": candidates,
        "skipped": skipped,
        "summary": summary,
    })
}

pub(crate) fn evolve_propose_json_data_created(
    branch: String,
    scope: crate::handlers::evolve::EvolveScope,
    files: Vec<String>,
    files_posix: Vec<String>,
    committed: bool,
) -> serde_json::Value {
    serde_json::json!({
        "created": true,
        "branch": branch,
        "scope": scope,
        "files": files,
        "files_posix": files_posix,
        "committed": committed,
    })
}
