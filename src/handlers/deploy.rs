use crate::engine::Engine;
use crate::targets::TargetRoot;
use crate::user_error::UserError;

#[derive(Debug, Clone, Copy)]
pub(crate) enum ConfirmationStyle {
    /// `--json` mode: mutating operations require `--yes`.
    JsonYes { command_id: &'static str },
    /// Interactive callers can prompt the user when changes exist.
    Interactive,
    /// Non-interactive callers require an explicit `confirmed=true` input.
    Explicit,
}

#[derive(Debug)]
pub(crate) enum DeployApplyOutcome {
    NoChanges,
    NeedsConfirmation,
    Applied { snapshot_id: String },
}

pub(crate) fn deploy_apply_in(
    engine: &Engine,
    plan: &crate::deploy::PlanResult,
    desired: &crate::deploy::DesiredState,
    roots: &[TargetRoot],
    adopt: bool,
    confirmed: bool,
    confirmation_style: ConfirmationStyle,
) -> anyhow::Result<DeployApplyOutcome> {
    match confirmation_style {
        ConfirmationStyle::JsonYes { command_id } => {
            if !confirmed {
                return Err(UserError::confirm_required(command_id));
            }
        }
        ConfirmationStyle::Explicit => {
            if !confirmed {
                return Err(anyhow::Error::new(UserError::new(
                    "E_CONFIRM_REQUIRED",
                    "refusing to apply without explicit confirmation",
                )));
            }
        }
        ConfirmationStyle::Interactive => {}
    }

    ensure_adopt_ok(plan, adopt)?;

    let needs_manifests = crate::target_manifest::manifests_missing_for_desired(roots, desired);
    if plan.changes.is_empty() && !needs_manifests {
        return Ok(DeployApplyOutcome::NoChanges);
    }

    if !confirmed {
        return Ok(DeployApplyOutcome::NeedsConfirmation);
    }

    let lockfile_path = engine
        .repo
        .lockfile_path
        .exists()
        .then_some(engine.repo.lockfile_path.as_path());
    let snapshot =
        crate::apply::apply_plan(&engine.home, "deploy", plan, desired, lockfile_path, roots)?;

    Ok(DeployApplyOutcome::Applied {
        snapshot_id: snapshot.id,
    })
}

fn ensure_adopt_ok(plan: &crate::deploy::PlanResult, adopt: bool) -> anyhow::Result<()> {
    let adopt_updates: Vec<&crate::deploy::PlanChange> = plan
        .changes
        .iter()
        .filter(|c| matches!(c.update_kind, Some(crate::deploy::UpdateKind::AdoptUpdate)))
        .collect();
    if adopt_updates.is_empty() || adopt {
        return Ok(());
    }

    let mut sample_paths: Vec<String> = adopt_updates.iter().map(|c| c.path.clone()).collect();
    sample_paths.sort();
    sample_paths.truncate(20);

    Err(anyhow::Error::new(
        UserError::new(
            "E_ADOPT_CONFIRM_REQUIRED",
            "refusing to overwrite unmanaged existing files without --adopt",
        )
        .with_details(serde_json::json!({
            "flag": "--adopt",
            "adopt_updates": adopt_updates.len(),
            "sample_paths": sample_paths,
        })),
    ))
}
