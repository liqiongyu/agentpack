use crate::paths::{AgentpackHome, RepoPaths};

use super::args::Cli;

pub(crate) mod add;
pub(crate) mod bootstrap;
pub(crate) mod completions;
pub(crate) mod deploy;
pub(crate) mod diff;
pub(crate) mod doctor;
pub(crate) mod evolve;
pub(crate) mod explain;
pub(crate) mod fetch;
pub(crate) mod help;
pub(crate) mod import;
pub(crate) mod init;
pub(crate) mod lock;
pub(crate) mod mcp;
pub(crate) mod overlay;
pub(crate) mod plan;
pub(crate) mod policy;
pub(crate) mod preview;
pub(crate) mod record;
pub(crate) mod remote;
pub(crate) mod remove;
pub(crate) mod rollback;
pub(crate) mod schema;
pub(crate) mod score;
pub(crate) mod status;
pub(crate) mod sync;
#[cfg(feature = "tui")]
pub(crate) mod tui;
pub(crate) mod update;

pub(crate) struct Ctx<'a> {
    pub(crate) cli: &'a Cli,
    pub(crate) home: &'a AgentpackHome,
    pub(crate) repo: &'a RepoPaths,
}
