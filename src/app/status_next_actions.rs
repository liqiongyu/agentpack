#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum StatusNextAction {
    PreviewDiff,
    DeployApply,
    EvolvePropose,
}

impl StatusNextAction {
    pub(crate) fn human_command(self, prefix: &str) -> String {
        format!("{prefix} {}", self.human_suffix())
    }

    pub(crate) fn json_command(self, prefix: &str) -> String {
        format!("{prefix} {}", self.json_suffix())
    }

    fn human_suffix(self) -> &'static str {
        match self {
            StatusNextAction::PreviewDiff => "preview --diff",
            StatusNextAction::DeployApply => "deploy --apply",
            StatusNextAction::EvolvePropose => "evolve propose",
        }
    }

    fn json_suffix(self) -> &'static str {
        match self {
            StatusNextAction::PreviewDiff => "preview --diff --json",
            StatusNextAction::DeployApply => "deploy --apply --yes --json",
            StatusNextAction::EvolvePropose => "evolve propose --yes --json",
        }
    }
}

pub(crate) fn status_next_actions(
    summary_modified: u64,
    summary_missing: u64,
    summary_extra: u64,
    any_manifest: bool,
    needs_deploy_apply: bool,
) -> std::collections::BTreeSet<StatusNextAction> {
    let mut out = std::collections::BTreeSet::new();

    if needs_deploy_apply {
        out.insert(StatusNextAction::DeployApply);
    }

    if summary_modified > 0 || summary_missing > 0 {
        out.insert(StatusNextAction::PreviewDiff);
        out.insert(StatusNextAction::DeployApply);

        // Only suggest evolve when there is a reliable baseline (a previous deploy wrote manifests).
        if any_manifest {
            out.insert(StatusNextAction::EvolvePropose);
        }
    } else if summary_extra > 0 {
        out.insert(StatusNextAction::PreviewDiff);
    }

    out
}
