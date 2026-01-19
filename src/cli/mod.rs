mod args;
mod commands;
mod dispatch;
mod human;
mod json;
pub(crate) mod util;

pub fn clap_command() -> clap::Command {
    use clap::CommandFactory as _;
    args::Cli::command()
}

pub use dispatch::run;
