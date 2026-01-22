pub(crate) fn ordered_next_actions(actions: &std::collections::BTreeSet<String>) -> Vec<String> {
    let mut out: Vec<String> = actions.iter().cloned().collect();
    out.sort_by(|a, b| {
        next_action_priority(a)
            .cmp(&next_action_priority(b))
            .then_with(|| a.cmp(b))
    });
    out
}

pub(crate) fn next_action_code(action: &str) -> &'static str {
    match next_action_subcommand(action) {
        Some("bootstrap") => "bootstrap",
        Some("doctor") => "doctor",
        Some("update") => "update",
        Some("preview") => {
            if action.contains(" --diff") {
                "preview_diff"
            } else {
                "preview"
            }
        }
        Some("diff") => "diff",
        Some("plan") => "plan",
        Some("deploy") => {
            if action.contains(" --apply") {
                "deploy_apply"
            } else {
                "deploy"
            }
        }
        Some("status") => "status",
        Some("evolve") => {
            if action.contains(" propose") {
                "evolve_propose"
            } else if action.contains(" restore") {
                "evolve_restore"
            } else {
                "evolve"
            }
        }
        Some("rollback") => "rollback",
        _ => "other",
    }
}

fn next_action_priority(action: &str) -> u8 {
    match next_action_subcommand(action) {
        Some("bootstrap") => 0,
        Some("doctor") => 10,
        Some("update") => 20,
        Some("preview") => 30,
        Some("diff") => 40,
        Some("plan") => 50,
        Some("deploy") => 60,
        Some("status") => 70,
        Some("evolve") => {
            if action.contains(" propose") {
                80
            } else {
                81
            }
        }
        Some("rollback") => 90,
        _ => 100,
    }
}

fn next_action_subcommand(action: &str) -> Option<&str> {
    let mut iter = action.split_whitespace();
    // Skip program name ("agentpack") and global flags (and their args).
    let _ = iter.next()?;

    while let Some(tok) = iter.next() {
        if !tok.starts_with("--") {
            return Some(tok);
        }

        // Skip flag value for the flags we know to take an argument.
        if matches!(tok, "--repo" | "--profile" | "--target" | "--machine") {
            let _ = iter.next();
        }
    }

    None
}
