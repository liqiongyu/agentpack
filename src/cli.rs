use std::path::PathBuf;

use anyhow::Context as _;
use clap::{Parser, Subcommand};

use crate::paths::{AgentpackHome, RepoPaths};

#[derive(Parser, Debug)]
#[command(name = "agentpack")]
#[command(about = "AI-first local asset control plane", long_about = None)]
pub struct Cli {
    /// Path to the agentpack config repo (default: $AGENTPACK_HOME/repo)
    #[arg(long)]
    repo: Option<PathBuf>,

    /// Profile name (default: "default")
    #[arg(long, default_value = "default")]
    profile: String,

    /// Target name: codex|claude_code|all (default: "all")
    #[arg(long, default_value = "all")]
    target: String,

    /// Machine-readable JSON output
    #[arg(long)]
    json: bool,

    /// Skip confirmations (dangerous with --apply)
    #[arg(long)]
    yes: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize the agentpack config repo
    Init,
}

pub fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let home = AgentpackHome::resolve()?;
    let repo = RepoPaths::resolve(&home, cli.repo.as_deref())?;

    match cli.command {
        Commands::Init => {
            repo.init_repo_skeleton().context("init repo")?;
            if cli.json {
                println!(
                    "{}",
                    serde_json::json!({
                        "ok": true,
                        "command": "init",
                        "version": env!("CARGO_PKG_VERSION"),
                        "data": {"repo": repo.repo_dir},
                        "warnings": [],
                        "errors": [],
                    })
                );
            } else {
                println!("Initialized agentpack repo at {}", repo.repo_dir.display());
            }
        }
    }

    Ok(())
}
