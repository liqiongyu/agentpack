use std::path::PathBuf;

use anyhow::Context as _;
use clap::{Parser, Subcommand};

use crate::output::{JsonEnvelope, JsonError, print_json};
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

    /// Add a module to agentpack.yaml
    Add,

    /// Remove a module from agentpack.yaml
    Remove,

    /// Generate/update agentpack.lock.json
    Lock,

    /// Fetch sources into store (per lockfile)
    Fetch,

    /// Show planned changes without applying
    Plan,

    /// Show diffs for planned changes
    Diff,

    /// Plan+diff, and optionally apply with --apply
    Deploy {
        /// Apply changes (writes to targets)
        #[arg(long)]
        apply: bool,
    },

    /// Check drift between expected and deployed outputs
    Status,

    /// Rollback to a deployment snapshot
    Rollback {
        /// Snapshot id to rollback to
        #[arg(long)]
        to: String,
    },

    /// Install operator assets for AI self-serve
    Bootstrap,

    /// Manage overlays (v0.1: edit)
    Overlay,
}

pub fn run() -> std::process::ExitCode {
    let cli = Cli::parse();
    match run_with(&cli) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(err) => {
            if cli.json {
                let envelope = JsonEnvelope::<serde_json::Value>::err(
                    cli.command_name(),
                    vec![JsonError {
                        code: "E_UNEXPECTED".to_string(),
                        message: err.to_string(),
                        details: None,
                    }],
                );
                let _ = print_json(&envelope);
            } else {
                eprintln!("{err:#}");
            }

            std::process::ExitCode::from(1)
        }
    }
}

fn run_with(cli: &Cli) -> anyhow::Result<()> {
    let home = AgentpackHome::resolve()?;
    let repo = RepoPaths::resolve(&home, cli.repo.as_deref())?;

    match &cli.command {
        Commands::Init => {
            repo.init_repo_skeleton().context("init repo")?;
            if cli.json {
                let envelope =
                    JsonEnvelope::ok("init", serde_json::json!({ "repo": repo.repo_dir }));
                print_json(&envelope)?;
            } else {
                println!("Initialized agentpack repo at {}", repo.repo_dir.display());
            }
        }
        Commands::Add
        | Commands::Remove
        | Commands::Lock
        | Commands::Fetch
        | Commands::Plan
        | Commands::Diff
        | Commands::Deploy { .. }
        | Commands::Status
        | Commands::Rollback { .. }
        | Commands::Bootstrap
        | Commands::Overlay => anyhow::bail!("command not implemented yet"),
    }

    Ok(())
}

impl Cli {
    fn command_name(&self) -> String {
        match &self.command {
            Commands::Init => "init",
            Commands::Add => "add",
            Commands::Remove => "remove",
            Commands::Lock => "lock",
            Commands::Fetch => "fetch",
            Commands::Plan => "plan",
            Commands::Diff => "diff",
            Commands::Deploy { .. } => "deploy",
            Commands::Status => "status",
            Commands::Rollback { .. } => "rollback",
            Commands::Bootstrap => "bootstrap",
            Commands::Overlay => "overlay",
        }
        .to_string()
    }
}
