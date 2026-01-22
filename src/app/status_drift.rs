#[derive(serde::Serialize)]
pub(crate) struct DriftSummaryByRoot {
    pub(crate) target: String,
    pub(crate) root: String,
    pub(crate) root_posix: String,
    pub(crate) summary: crate::handlers::status::DriftSummary,
}

pub(crate) fn drift_summary(
    drift: &[crate::handlers::status::DriftItem],
) -> crate::handlers::status::DriftSummary {
    let mut summary = crate::handlers::status::DriftSummary::default();
    for d in drift {
        match d.kind.as_str() {
            "modified" => summary.modified += 1,
            "missing" => summary.missing += 1,
            "extra" => summary.extra += 1,
            _ => {}
        }
    }
    summary
}

pub(crate) fn drift_summary_by_root(
    drift: &[crate::handlers::status::DriftItem],
) -> Vec<DriftSummaryByRoot> {
    let mut by_root: std::collections::BTreeMap<(String, String), DriftSummaryByRoot> =
        std::collections::BTreeMap::new();

    for d in drift {
        let root = d.root.as_deref().unwrap_or("<unknown>").to_string();
        let root_posix = d.root_posix.as_deref().unwrap_or("<unknown>").to_string();
        let key = (d.target.clone(), root_posix.clone());
        let entry = by_root.entry(key).or_insert_with(|| DriftSummaryByRoot {
            target: d.target.clone(),
            root,
            root_posix,
            summary: crate::handlers::status::DriftSummary::default(),
        });
        match d.kind.as_str() {
            "modified" => entry.summary.modified += 1,
            "missing" => entry.summary.missing += 1,
            "extra" => entry.summary.extra += 1,
            _ => {}
        }
    }

    by_root.into_values().collect()
}
