use std::collections::BTreeSet;

use crate::handlers::doctor::DoctorRootCheck;

#[derive(Default)]
pub(crate) struct DoctorNextActions {
    pub human: BTreeSet<String>,
    pub json: BTreeSet<String>,
}

pub(crate) fn doctor_next_actions(
    roots: &[DoctorRootCheck],
    needs_gitignore_fix: bool,
    fix: bool,
    prefix: &str,
) -> DoctorNextActions {
    let mut out = DoctorNextActions::default();

    for c in roots {
        if let Some(suggestion) = &c.suggestion {
            if let Some((_, cmd)) = suggestion.split_once(':') {
                let cmd = cmd.trim();
                if !cmd.is_empty() {
                    out.human.insert(cmd.to_string());
                    out.json.insert(cmd.to_string());
                }
            }
        }
    }

    if needs_gitignore_fix && !fix {
        out.human.insert(format!("{prefix} doctor --fix"));
        out.json
            .insert(format!("{prefix} doctor --fix --yes --json"));
    }

    out
}
