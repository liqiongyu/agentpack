use clap::Parser;

use super::args::*;
use super::human::print_user_error_human;
use super::json::print_anyhow_error;

use crate::paths::{AgentpackHome, RepoPaths};

pub fn run() -> std::process::ExitCode {
    let cli = Cli::parse();
    match run_with(&cli) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(err) => {
            if cli.json {
                print_anyhow_error(&cli, &err);
            } else if !print_user_error_human(&err) {
                eprintln!("{err:#}");
            }

            std::process::ExitCode::from(1)
        }
    }
}

fn run_with(cli: &Cli) -> anyhow::Result<()> {
    let home = AgentpackHome::resolve()?;
    let repo = RepoPaths::resolve(&home, cli.repo.as_deref())?;
    let ctx = super::commands::Ctx {
        cli,
        home: &home,
        repo: &repo,
    };

    match &cli.command {
        Commands::Init { git, bootstrap } => {
            super::commands::init::run(&ctx, *git, *bootstrap)?;
        }
        Commands::Help => {
            super::commands::help::run(&ctx)?;
        }
        Commands::Schema => {
            super::commands::schema::run(&ctx)?;
        }
        Commands::Add {
            module_type,
            source,
            id,
            tags,
            targets,
        } => {
            super::commands::add::run(
                &ctx,
                module_type,
                source,
                id,
                tags.as_slice(),
                targets.as_slice(),
            )?;
        }
        Commands::Remove { module_id } => {
            super::commands::remove::run(&ctx, module_id)?;
        }
        Commands::Lock => {
            super::commands::lock::run(&ctx)?;
        }
        Commands::Update {
            lock,
            fetch,
            no_lock,
            no_fetch,
        } => {
            super::commands::update::run(&ctx, *lock, *fetch, *no_lock, *no_fetch)?;
        }
        Commands::Fetch => {
            super::commands::fetch::run(&ctx)?;
        }
        Commands::Preview { diff } => {
            super::commands::preview::run(&ctx, *diff)?;
        }
        Commands::Overlay { command } => {
            super::commands::overlay::run(&ctx, command)?;
        }
        Commands::Plan => {
            super::commands::plan::run(&ctx)?;
        }
        Commands::Diff => {
            super::commands::diff::run(&ctx)?;
        }
        Commands::Deploy { apply, adopt } => {
            super::commands::deploy::run(&ctx, *apply, *adopt)?;
        }
        Commands::Status { only } => {
            super::commands::status::run(&ctx, only)?;
        }
        #[cfg(feature = "tui")]
        Commands::Tui => {
            super::commands::tui::run(&ctx)?;
        }
        Commands::Doctor { fix } => {
            super::commands::doctor::run(&ctx, *fix)?;
        }
        Commands::Remote { command } => {
            super::commands::remote::run(&ctx, command)?;
        }
        Commands::Sync { rebase, remote } => {
            super::commands::sync::run(&ctx, *rebase, remote)?;
        }
        Commands::Record => {
            super::commands::record::run(&ctx)?;
        }
        Commands::Score => {
            super::commands::score::run(&ctx)?;
        }
        Commands::Explain { command } => {
            super::commands::explain::run(&ctx, command)?;
        }
        Commands::Evolve { command } => {
            super::commands::evolve::run(&ctx, command)?;
        }
        Commands::Completions { shell } => {
            super::commands::completions::run(&ctx, *shell)?;
        }
        Commands::Bootstrap { scope } => {
            super::commands::bootstrap::run(&ctx, *scope)?;
        }
        Commands::Rollback { to } => {
            super::commands::rollback::run(&ctx, to)?;
        }
    }

    Ok(())
}
