use std::path::Path;

use crate::engine::Engine;
use crate::handlers::deploy::{ConfirmationStyle, DeployApplyOutcome, deploy_apply_in};
use crate::handlers::read_only::read_only_context_in;
use crate::user_error::UserError;

#[derive(Debug)]
pub enum ApplyOutcome {
    Applied { snapshot_id: String },
    NoChanges,
}

pub fn apply_from_tui(
    repo_override: Option<&Path>,
    machine_override: Option<&str>,
    profile: &str,
    target_filter: &str,
    adopt: bool,
    confirmed: bool,
) -> anyhow::Result<ApplyOutcome> {
    if !confirmed {
        return Err(anyhow::Error::new(UserError::new(
            "E_CONFIRM_REQUIRED",
            "refusing to apply without explicit confirmation",
        )));
    }

    let engine = Engine::load(repo_override, machine_override)?;
    apply_from_tui_in(&engine, profile, target_filter, adopt, confirmed)
}

pub fn apply_from_tui_in(
    engine: &Engine,
    profile: &str,
    target_filter: &str,
    adopt: bool,
    confirmed: bool,
) -> anyhow::Result<ApplyOutcome> {
    let ctx = read_only_context_in(engine, profile, target_filter)?;
    match deploy_apply_in(
        engine,
        &ctx.plan,
        &ctx.desired,
        &ctx.roots,
        adopt,
        confirmed,
        ConfirmationStyle::Explicit,
    )? {
        DeployApplyOutcome::NoChanges => Ok(ApplyOutcome::NoChanges),
        DeployApplyOutcome::Applied { snapshot_id } => Ok(ApplyOutcome::Applied { snapshot_id }),
        DeployApplyOutcome::NeedsConfirmation => anyhow::bail!(
            "tui apply requires explicit confirmation, but confirmation was not provided"
        ),
    }
}
