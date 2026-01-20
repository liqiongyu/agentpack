use anyhow::Context as _;

use crate::paths::AgentpackHome;
use crate::user_error::UserError;

pub(crate) fn rollback(
    home: &AgentpackHome,
    snapshot_id: &str,
    json: bool,
    yes: bool,
) -> anyhow::Result<crate::state::DeploymentSnapshot> {
    if json && !yes {
        return Err(UserError::confirm_required("rollback"));
    }

    crate::apply::rollback(home, snapshot_id).context("rollback")
}
