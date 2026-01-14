use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

use crate::config::ModuleType;

#[derive(Parser, Debug)]
#[command(name = "agentpack")]
#[command(about = "AI-first local asset control plane", long_about = None)]
#[command(disable_help_subcommand = true)]
pub struct Cli {
    /// Path to the agentpack config repo (default: $AGENTPACK_HOME/repo)
    #[arg(long, global = true)]
    pub(crate) repo: Option<PathBuf>,

    /// Profile name (default: "default")
    #[arg(long, default_value = "default", global = true)]
    pub(crate) profile: String,

    /// Target name: codex|claude_code|cursor|vscode|all (default: "all")
    #[arg(long, default_value = "all", global = true)]
    pub(crate) target: String,

    /// Machine id for machine overlays (default: auto-detect)
    #[arg(long, global = true)]
    pub(crate) machine: Option<String>,

    /// Machine-readable JSON output
    #[arg(long, global = true)]
    pub(crate) json: bool,

    /// Skip confirmations (dangerous with --apply)
    #[arg(long, global = true)]
    pub(crate) yes: bool,

    /// Force dry-run behavior (do not apply even if --apply is set)
    #[arg(long, global = true)]
    pub(crate) dry_run: bool,

    #[command(subcommand)]
    pub(crate) command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize the agentpack config repo
    Init {
        /// Also initialize the repo as a git repository (idempotent)
        #[arg(long)]
        git: bool,

        /// Also install operator assets after init (equivalent to `agentpack bootstrap`)
        #[arg(long)]
        bootstrap: bool,
    },

    /// Self-describing CLI help (supports --json)
    Help,

    /// Describe the JSON output contract (supports --json)
    Schema,

    /// Composite command: lock and/or fetch (default: fetch; runs lock+fetch if lockfile is missing)
    Update {
        /// Force re-generating the lockfile
        #[arg(long, conflicts_with = "no_lock")]
        lock: bool,

        /// Force running fetch
        #[arg(long, conflicts_with = "no_fetch")]
        fetch: bool,

        /// Skip lockfile generation
        #[arg(long)]
        no_lock: bool,

        /// Skip fetch
        #[arg(long)]
        no_fetch: bool,
    },

    /// Add a module to agentpack.yaml
    Add {
        #[arg(value_enum)]
        module_type: ModuleType,

        /// Source spec: local:... or git:...
        source: String,

        /// Explicit module id (default: derived from type + source)
        #[arg(long)]
        id: Option<String>,

        /// Comma-separated tags (for profiles)
        #[arg(long, value_delimiter = ',')]
        tags: Vec<String>,

        /// Comma-separated target names (codex, claude_code, cursor, vscode). Empty = all.
        #[arg(long, value_delimiter = ',')]
        targets: Vec<String>,
    },

    /// Remove a module from agentpack.yaml
    Remove { module_id: String },

    /// Generate/update agentpack.lock.json
    Lock,

    /// Fetch sources into store (per lockfile)
    Fetch,

    /// Composite command: plan + (optional) diff
    Preview {
        /// Include diffs (human: unified diff; json: diff summary)
        #[arg(long)]
        diff: bool,
    },

    /// Show planned changes without applying
    Plan,

    /// Show diffs for planned changes
    Diff,

    /// Plan+diff, and optionally apply with --apply
    Deploy {
        /// Apply changes (writes to targets)
        #[arg(long)]
        apply: bool,

        /// Allow overwriting existing unmanaged files (adopt updates)
        #[arg(long)]
        adopt: bool,
    },

    /// Check drift between expected and deployed outputs
    Status {
        /// Filter drift items by kind (repeatable or comma-separated)
        #[arg(long, value_enum, value_delimiter = ',')]
        only: Vec<StatusOnly>,
    },

    /// Check local environment and target paths
    Doctor {
        /// Idempotently add `.agentpack.manifest.json` to `.gitignore` for detected repos
        #[arg(long)]
        fix: bool,
    },

    /// Configure git remotes for the agentpack config repo
    Remote {
        #[command(subcommand)]
        command: RemoteCommands,
    },

    /// Sync the agentpack config repo (pull/rebase + push)
    Sync {
        /// Pull with rebase (recommended)
        #[arg(long)]
        rebase: bool,

        /// Remote name (default: origin)
        #[arg(long, default_value = "origin")]
        remote: String,
    },

    /// Record an execution event (reads JSON from stdin and appends to local logs)
    Record,

    /// Score modules based on recorded events
    Score,

    /// Explain why plan/diff/status looks the way it does
    Explain {
        #[command(subcommand)]
        command: ExplainCommands,
    },

    /// Evolution helpers (record/score/propose loop)
    Evolve {
        #[command(subcommand)]
        command: EvolveCommands,
    },

    /// Generate shell completion scripts
    Completions {
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },

    /// Rollback to a deployment snapshot
    Rollback {
        /// Snapshot id to rollback to
        #[arg(long)]
        to: String,
    },

    /// Install operator assets for AI self-serve
    Bootstrap {
        /// Where to install operator assets (default: both)
        #[arg(long, value_enum, default_value = "both")]
        scope: BootstrapScope,
    },

    /// Manage overlays (v0.1: edit)
    Overlay {
        #[command(subcommand)]
        command: OverlayCommands,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum BootstrapScope {
    User,
    Project,
    Both,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OverlayScope {
    Global,
    Machine,
    Project,
}

#[derive(Subcommand, Debug)]
pub enum OverlayCommands {
    /// Create an overlay skeleton and open an editor
    Edit {
        module_id: String,

        /// Overlay scope to write into (default: global)
        #[arg(long, value_enum, default_value = "global")]
        scope: OverlayScope,

        /// Use project overlay (DEPRECATED: use --scope project)
        #[arg(long)]
        project: bool,

        /// Create a sparse overlay (do not copy upstream files)
        #[arg(long, conflicts_with = "materialize")]
        sparse: bool,

        /// Populate upstream files into the overlay without overwriting existing edits
        #[arg(long, conflicts_with = "sparse")]
        materialize: bool,
    },

    /// Rebase an overlay against the current upstream (3-way merge)
    Rebase {
        module_id: String,

        /// Overlay scope to rebase (default: global)
        #[arg(long, value_enum, default_value = "global")]
        scope: OverlayScope,

        /// Remove overlay files that end up identical to upstream after rebasing
        #[arg(long)]
        sparsify: bool,
    },

    /// Print the resolved overlay directory for a module and scope
    Path {
        module_id: String,

        /// Overlay scope to resolve (default: global)
        #[arg(long, value_enum, default_value = "global")]
        scope: OverlayScope,
    },
}

#[derive(Subcommand, Debug)]
pub enum RemoteCommands {
    /// Set a git remote URL for the config repo (creates the remote if missing)
    Set {
        url: String,

        /// Remote name (default: origin)
        #[arg(long, default_value = "origin")]
        name: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum ExplainCommands {
    /// Explain the current plan (module provenance and overlay layers)
    Plan,

    /// Explain the current diff (same as plan, with diffs in `agentpack diff`)
    Diff,

    /// Explain current drift/status (module provenance and overlay layers)
    Status,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum StatusOnly {
    Missing,
    Modified,
    Extra,
}

#[derive(Debug, Clone, Copy, ValueEnum, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvolveScope {
    Global,
    Machine,
    Project,
}

#[derive(Subcommand, Debug)]
pub enum EvolveCommands {
    /// Propose overlay updates by capturing drifted deployed files into overlays (creates a local git branch)
    Propose {
        /// Only propose changes for a single module id
        #[arg(long)]
        module_id: Option<String>,

        /// Overlay scope to write into (default: global)
        #[arg(long, value_enum, default_value = "global")]
        scope: EvolveScope,

        /// Branch name to create (default: evolve/propose-<timestamp>)
        #[arg(long)]
        branch: Option<String>,
    },

    /// Restore missing desired outputs on disk (create-only; no updates/deletes)
    Restore {
        /// Only restore missing outputs attributable to a module id
        #[arg(long)]
        module_id: Option<String>,
    },
}

impl Cli {
    pub(crate) fn command_name(&self) -> String {
        match &self.command {
            Commands::Init { .. } => "init",
            Commands::Help => "help",
            Commands::Schema => "schema",
            Commands::Add { .. } => "add",
            Commands::Remove { .. } => "remove",
            Commands::Lock => "lock",
            Commands::Update { .. } => "update",
            Commands::Fetch => "fetch",
            Commands::Preview { .. } => "preview",
            Commands::Plan => "plan",
            Commands::Diff => "diff",
            Commands::Deploy { .. } => "deploy",
            Commands::Status { .. } => "status",
            Commands::Doctor { .. } => "doctor",
            Commands::Remote { .. } => "remote",
            Commands::Sync { .. } => "sync",
            Commands::Record => "record",
            Commands::Score => "score",
            Commands::Explain { .. } => "explain",
            Commands::Evolve { .. } => "evolve",
            Commands::Completions { .. } => "completions",
            Commands::Rollback { .. } => "rollback",
            Commands::Bootstrap { .. } => "bootstrap",
            Commands::Overlay { .. } => "overlay",
        }
        .to_string()
    }
}
