use std::path::PathBuf;

use anyhow::Context as _;
use clap::{Parser, Subcommand};

use crate::config::{Manifest, Module, ModuleType};
use crate::lockfile::{Lockfile, generate_lockfile, hash_tree};
use crate::output::{JsonEnvelope, JsonError, print_json};
use crate::overlay::ensure_overlay_skeleton;
use crate::paths::{AgentpackHome, RepoPaths};
use crate::project::ProjectContext;
use crate::source::parse_source_spec;
use crate::store::Store;

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

        /// Comma-separated target names (codex, claude_code). Empty = all.
        #[arg(long, value_delimiter = ',')]
        targets: Vec<String>,
    },

    /// Remove a module from agentpack.yaml
    Remove { module_id: String },

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
    Overlay {
        #[command(subcommand)]
        command: OverlayCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum OverlayCommands {
    /// Create an overlay skeleton and open an editor
    Edit {
        module_id: String,

        /// Use project overlay (requires running inside a git repo / project directory)
        #[arg(long)]
        project: bool,
    },
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
        Commands::Add {
            module_type,
            source,
            id,
            tags,
            targets,
        } => {
            let mut manifest = Manifest::load(&repo.manifest_path).context("load manifest")?;
            let parsed_source = parse_source_spec(source).context("parse source")?;
            let module_id = id
                .clone()
                .unwrap_or_else(|| derive_module_id(module_type, source));

            manifest.modules.push(Module {
                id: module_id.clone(),
                module_type: module_type.clone(),
                enabled: true,
                tags: tags.clone(),
                targets: targets.clone(),
                source: parsed_source,
                metadata: Default::default(),
            });

            manifest
                .save(&repo.manifest_path)
                .context("save manifest")?;

            if cli.json {
                let envelope = JsonEnvelope::ok(
                    "add",
                    serde_json::json!({ "module_id": module_id, "manifest": repo.manifest_path }),
                );
                print_json(&envelope)?;
            } else {
                println!("Added module {module_id}");
            }
        }
        Commands::Remove { module_id } => {
            let mut manifest = Manifest::load(&repo.manifest_path).context("load manifest")?;
            let before = manifest.modules.len();
            manifest.modules.retain(|m| m.id != *module_id);
            if manifest.modules.len() == before {
                anyhow::bail!("module not found: {module_id}");
            }
            manifest
                .save(&repo.manifest_path)
                .context("save manifest")?;

            if cli.json {
                let envelope = JsonEnvelope::ok(
                    "remove",
                    serde_json::json!({ "module_id": module_id, "manifest": repo.manifest_path }),
                );
                print_json(&envelope)?;
            } else {
                println!("Removed module {module_id}");
            }
        }
        Commands::Lock => {
            let manifest = Manifest::load(&repo.manifest_path).context("load manifest")?;
            let store = Store::new(&home);
            let lock = generate_lockfile(&repo, &manifest, &store).context("generate lockfile")?;
            lock.save(&repo.lockfile_path).context("write lockfile")?;

            if cli.json {
                let envelope = JsonEnvelope::ok(
                    "lock",
                    serde_json::json!({
                        "lockfile": repo.lockfile_path,
                        "modules": lock.modules.len(),
                    }),
                );
                print_json(&envelope)?;
            } else {
                println!(
                    "Wrote lockfile {} ({} modules)",
                    repo.lockfile_path.display(),
                    lock.modules.len()
                );
            }
        }
        Commands::Fetch => {
            let lock = Lockfile::load(&repo.lockfile_path).context("load lockfile")?;
            let store = Store::new(&home);
            store.ensure_layout()?;

            let mut fetched = 0usize;
            for m in &lock.modules {
                let Some(gs) = &m.resolved_source.git else {
                    continue;
                };

                let src = crate::config::GitSource {
                    url: gs.url.clone(),
                    ref_name: gs.commit.clone(),
                    subdir: gs.subdir.clone(),
                    shallow: false,
                };
                let checkout = store.ensure_git_checkout(&m.id, &src, &gs.commit)?;
                let root = Store::module_root_in_checkout(&checkout, &gs.subdir);
                let (_files, hash) = hash_tree(&root)?;
                if hash != m.sha256 {
                    anyhow::bail!(
                        "store content hash mismatch for {}: expected {}, got {}",
                        m.id,
                        m.sha256,
                        hash
                    );
                }
                fetched += 1;
            }

            if cli.json {
                let envelope = JsonEnvelope::ok(
                    "fetch",
                    serde_json::json!({
                        "store": home.store_dir,
                        "git_modules_fetched": fetched,
                    }),
                );
                print_json(&envelope)?;
            } else {
                println!(
                    "Fetched/verified {fetched} git module(s) into {}",
                    home.store_dir.display()
                );
            }
        }
        Commands::Overlay { command } => match command {
            OverlayCommands::Edit { module_id, project } => {
                let manifest = Manifest::load(&repo.manifest_path).context("load manifest")?;

                let project_ctx = if *project {
                    let cwd = std::env::current_dir().context("get cwd")?;
                    Some(ProjectContext::detect(&cwd).context("detect project")?)
                } else {
                    None
                };

                let skeleton = ensure_overlay_skeleton(
                    &home,
                    &repo,
                    &manifest,
                    module_id,
                    project_ctx.as_ref(),
                )
                .context("ensure overlay")?;

                if let Ok(editor) = std::env::var("EDITOR") {
                    if !editor.trim().is_empty() {
                        let mut cmd = std::process::Command::new(editor);
                        let status = cmd.arg(&skeleton.dir).status().context("launch editor")?;
                        if !status.success() {
                            anyhow::bail!("editor exited with status: {status}");
                        }
                    }
                }

                if cli.json {
                    let envelope = JsonEnvelope::ok(
                        "overlay.edit",
                        serde_json::json!({
                            "module_id": module_id,
                            "overlay_dir": skeleton.dir,
                            "created": skeleton.created,
                            "project": project,
                        }),
                    );
                    print_json(&envelope)?;
                } else if skeleton.created {
                    println!("Created overlay at {}", skeleton.dir.display());
                } else {
                    println!("Overlay already exists at {}", skeleton.dir.display());
                }
            }
        },
        Commands::Plan
        | Commands::Diff
        | Commands::Deploy { .. }
        | Commands::Status
        | Commands::Bootstrap => anyhow::bail!("command not implemented yet"),
        Commands::Rollback { to } => {
            crate::apply::rollback(&home, to).context("rollback")?;
            if cli.json {
                let envelope = JsonEnvelope::ok(
                    "rollback",
                    serde_json::json!({
                        "snapshot_id": to,
                    }),
                );
                print_json(&envelope)?;
            } else {
                println!("Rolled back to snapshot {to}");
            }
        }
    }

    Ok(())
}

impl Cli {
    fn command_name(&self) -> String {
        match &self.command {
            Commands::Init => "init",
            Commands::Add { .. } => "add",
            Commands::Remove { .. } => "remove",
            Commands::Lock => "lock",
            Commands::Fetch => "fetch",
            Commands::Plan => "plan",
            Commands::Diff => "diff",
            Commands::Deploy { .. } => "deploy",
            Commands::Status => "status",
            Commands::Rollback { .. } => "rollback",
            Commands::Bootstrap => "bootstrap",
            Commands::Overlay { .. } => "overlay",
        }
        .to_string()
    }
}

fn derive_module_id(module_type: &ModuleType, source_spec: &str) -> String {
    let prefix = match module_type {
        ModuleType::Instructions => "instructions",
        ModuleType::Skill => "skill",
        ModuleType::Prompt => "prompt",
        ModuleType::Command => "command",
    };

    let name = if let Some(path) = source_spec.strip_prefix("local:") {
        std::path::Path::new(path)
            .file_stem()
            .and_then(|s| s.to_str())
            .or_else(|| {
                std::path::Path::new(path)
                    .file_name()
                    .and_then(|s| s.to_str())
            })
            .unwrap_or("module")
            .to_string()
    } else if let Some(rest) = source_spec.strip_prefix("git:") {
        let (url, query) = rest.split_once('#').unwrap_or((rest, ""));
        if let Some(subdir) = query.split('&').find_map(|kv| {
            kv.split_once('=')
                .filter(|(k, _)| *k == "subdir")
                .map(|(_, v)| v)
        }) {
            std::path::Path::new(subdir)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("module")
                .to_string()
        } else {
            url.rsplit('/')
                .next()
                .unwrap_or("module")
                .trim_end_matches(".git")
                .to_string()
        }
    } else {
        "module".to_string()
    };

    format!("{prefix}:{name}")
}
