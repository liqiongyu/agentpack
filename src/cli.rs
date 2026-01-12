use std::path::PathBuf;

use anyhow::Context as _;
use clap::{CommandFactory, Parser, Subcommand, ValueEnum};

use crate::config::{Manifest, Module, ModuleType, TargetScope};
use crate::deploy::{TargetPath, load_managed_paths_from_snapshot, plan as compute_plan};
use crate::diff::unified_diff;
use crate::engine::Engine;
use crate::fs::write_atomic;
use crate::hash::sha256_hex;
use crate::lockfile::{Lockfile, generate_lockfile, hash_tree};
use crate::output::{JsonEnvelope, JsonError, print_json};
use crate::overlay::{ensure_overlay_skeleton, resolve_upstream_module_root};
use crate::paths::{AgentpackHome, RepoPaths};
use crate::source::parse_source_spec;
use crate::state::latest_snapshot;
use crate::store::Store;
use crate::user_error::UserError;

const UNIFIED_DIFF_MAX_BYTES: usize = 100 * 1024;

const TEMPLATE_CODEX_OPERATOR_SKILL: &str =
    include_str!("../templates/codex/skills/agentpack-operator/SKILL.md");
const TEMPLATE_CLAUDE_AP_DOCTOR: &str = include_str!("../templates/claude/commands/ap-doctor.md");
const TEMPLATE_CLAUDE_AP_UPDATE: &str = include_str!("../templates/claude/commands/ap-update.md");
const TEMPLATE_CLAUDE_AP_PREVIEW: &str = include_str!("../templates/claude/commands/ap-preview.md");
const TEMPLATE_CLAUDE_AP_PLAN: &str = include_str!("../templates/claude/commands/ap-plan.md");
const TEMPLATE_CLAUDE_AP_DEPLOY: &str = include_str!("../templates/claude/commands/ap-deploy.md");
const TEMPLATE_CLAUDE_AP_STATUS: &str = include_str!("../templates/claude/commands/ap-status.md");
const TEMPLATE_CLAUDE_AP_DIFF: &str = include_str!("../templates/claude/commands/ap-diff.md");
const TEMPLATE_CLAUDE_AP_EXPLAIN: &str = include_str!("../templates/claude/commands/ap-explain.md");
const TEMPLATE_CLAUDE_AP_EVOLVE: &str = include_str!("../templates/claude/commands/ap-evolve.md");

fn render_operator_template_bytes(template: &str) -> Vec<u8> {
    template
        .replace("{{AGENTPACK_VERSION}}", env!("CARGO_PKG_VERSION"))
        .into_bytes()
}

const MUTATING_COMMAND_IDS: &[&str] = &[
    "init",
    "add",
    "remove",
    "lock",
    "fetch",
    "update",
    "deploy --apply",
    "rollback",
    "bootstrap",
    "doctor --fix",
    "overlay edit",
    "remote set",
    "sync",
    "record",
    "evolve propose",
];

#[derive(Parser, Debug)]
#[command(name = "agentpack")]
#[command(about = "AI-first local asset control plane", long_about = None)]
#[command(disable_help_subcommand = true)]
pub struct Cli {
    /// Path to the agentpack config repo (default: $AGENTPACK_HOME/repo)
    #[arg(long, global = true)]
    repo: Option<PathBuf>,

    /// Profile name (default: "default")
    #[arg(long, default_value = "default", global = true)]
    profile: String,

    /// Target name: codex|claude_code|all (default: "all")
    #[arg(long, default_value = "all", global = true)]
    target: String,

    /// Machine id for machine overlays (default: auto-detect)
    #[arg(long, global = true)]
    machine: Option<String>,

    /// Machine-readable JSON output
    #[arg(long, global = true)]
    json: bool,

    /// Skip confirmations (dangerous with --apply)
    #[arg(long, global = true)]
    yes: bool,

    /// Force dry-run behavior (do not apply even if --apply is set)
    #[arg(long, global = true)]
    dry_run: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize the agentpack config repo
    Init,

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
    },

    /// Check drift between expected and deployed outputs
    Status,

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
}

pub fn run() -> std::process::ExitCode {
    let cli = Cli::parse();
    match run_with(&cli) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(err) => {
            if cli.json {
                let user_err = err.chain().find_map(|e| e.downcast_ref::<UserError>());
                let envelope = JsonEnvelope::<serde_json::Value>::err(
                    cli.command_name(),
                    vec![JsonError {
                        code: user_err
                            .map(|e| e.code.clone())
                            .unwrap_or_else(|| "E_UNEXPECTED".to_string()),
                        message: user_err
                            .map(|e| e.message.clone())
                            .unwrap_or_else(|| err.to_string()),
                        details: user_err.and_then(|e| e.details.clone()),
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

fn require_yes_for_json_mutation(cli: &Cli, command_id: &'static str) -> anyhow::Result<()> {
    debug_assert!(
        MUTATING_COMMAND_IDS.contains(&command_id),
        "mutating command id must be registered in MUTATING_COMMAND_IDS: {command_id}"
    );
    if cli.json && !cli.yes {
        return Err(UserError::confirm_required(command_id));
    }
    Ok(())
}

fn run_with(cli: &Cli) -> anyhow::Result<()> {
    let home = AgentpackHome::resolve()?;
    let repo = RepoPaths::resolve(&home, cli.repo.as_deref())?;

    match &cli.command {
        Commands::Init => {
            require_yes_for_json_mutation(cli, "init")?;
            repo.init_repo_skeleton().context("init repo")?;
            if cli.json {
                let envelope =
                    JsonEnvelope::ok("init", serde_json::json!({ "repo": repo.repo_dir }));
                print_json(&envelope)?;
            } else {
                println!("Initialized agentpack repo at {}", repo.repo_dir.display());
            }
        }
        Commands::Help => {
            if cli.json {
                let commands = serde_json::json!([
                    {"id":"init","path":["init"],"mutating":true},
                    {"id":"help","path":["help"],"mutating":false},
                    {"id":"schema","path":["schema"],"mutating":false},
                    {"id":"update","path":["update"],"mutating":true},
                    {"id":"add","path":["add"],"mutating":true},
                    {"id":"remove","path":["remove"],"mutating":true},
                    {"id":"lock","path":["lock"],"mutating":true},
                    {"id":"fetch","path":["fetch"],"mutating":true},
                    {"id":"preview","path":["preview"],"mutating":false},
                    {"id":"plan","path":["plan"],"mutating":false},
                    {"id":"diff","path":["diff"],"mutating":false},
                    {"id":"deploy","path":["deploy"],"mutating":false},
                    {"id":"status","path":["status"],"mutating":false},
                    {"id":"doctor","path":["doctor"],"mutating":false},
                    {"id":"doctor --fix","path":["doctor"],"mutating":true},
                    {"id":"remote set","path":["remote","set"],"mutating":true},
                    {"id":"sync","path":["sync"],"mutating":true},
                    {"id":"record","path":["record"],"mutating":true},
                    {"id":"score","path":["score"],"mutating":false},
                    {"id":"explain plan","path":["explain","plan"],"mutating":false},
                    {"id":"explain diff","path":["explain","diff"],"mutating":false},
                    {"id":"explain status","path":["explain","status"],"mutating":false},
                    {"id":"evolve propose","path":["evolve","propose"],"mutating":true},
                    {"id":"completions","path":["completions"],"mutating":false},
                    {"id":"rollback","path":["rollback"],"mutating":true},
                    {"id":"bootstrap","path":["bootstrap"],"mutating":true},
                    {"id":"overlay edit","path":["overlay","edit"],"mutating":true},
                    {"id":"overlay path","path":["overlay","path"],"mutating":false}
                ]);

                let envelope = JsonEnvelope::ok(
                    "help",
                    serde_json::json!({
                        "commands": commands,
                        "mutating_commands": MUTATING_COMMAND_IDS,
                        "notes": [
                            "recommended: doctor -> update -> preview -> deploy --apply",
                            "recommended: status -> evolve propose -> review -> deploy --apply",
                            "in --json mode, mutating commands require --yes"
                        ]
                    }),
                );
                print_json(&envelope)?;
            } else {
                let mut cmd = Cli::command();
                cmd.print_long_help()?;
                println!();
            }
        }
        Commands::Schema => {
            if cli.json {
                let envelope = JsonEnvelope::ok(
                    "schema",
                    serde_json::json!({
                        "envelope": {
                            "schema_version": 1,
                            "fields": {
                                "schema_version": "number",
                                "ok": "boolean",
                                "command": "string",
                                "version": "string",
                                "data": "object",
                                "warnings": "array[string]",
                                "errors": "array[{code,message,details?}]",
                            },
                            "error_item": {
                                "code": "string",
                                "message": "string",
                                "details": "object|null"
                            }
                        },
                        "commands": {
                            "plan": { "data_fields": ["profile","targets","changes","summary"] },
                            "diff": { "data_fields": ["profile","targets","changes","summary"] },
                            "preview": { "data_fields": ["profile","targets","plan","diff?"] },
                            "status": { "data_fields": ["profile","targets","drift","summary"] }
                        }
                    }),
                );
                print_json(&envelope)?;
            } else {
                println!("JSON envelope schema_version=1");
                println!("- keys: schema_version, ok, command, version, data, warnings, errors");
                println!("- key commands: plan/diff/preview/status (use --json to inspect)");
            }
        }
        Commands::Add {
            module_type,
            source,
            id,
            tags,
            targets,
        } => {
            require_yes_for_json_mutation(cli, "add")?;
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
            require_yes_for_json_mutation(cli, "remove")?;
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
            require_yes_for_json_mutation(cli, "lock")?;
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
        Commands::Update {
            lock,
            fetch,
            no_lock,
            no_fetch,
        } => {
            #[derive(Debug, Clone, serde::Serialize)]
            struct UpdateStep {
                name: String,
                ok: bool,
                detail: serde_json::Value,
            }

            let lockfile_exists = repo.lockfile_path.exists();
            let mut do_lock = !lockfile_exists;
            let mut do_fetch = true;

            if *lock {
                do_lock = true;
            }
            if *fetch {
                do_fetch = true;
            }
            if *no_lock {
                do_lock = false;
            }
            if *no_fetch {
                do_fetch = false;
            }

            if do_fetch && !do_lock && !lockfile_exists {
                anyhow::bail!(
                    "lockfile missing: {}; run `agentpack lock` first or omit --no-lock",
                    repo.lockfile_path.display()
                );
            }

            let will_write = do_lock || do_fetch;
            if cli.json && will_write && !cli.yes {
                return Err(UserError::confirm_required("update"));
            }

            let mut steps: Vec<UpdateStep> = Vec::new();
            let store = Store::new(&home);

            let lock = if do_lock {
                let manifest = Manifest::load(&repo.manifest_path).context("load manifest")?;
                let lock =
                    generate_lockfile(&repo, &manifest, &store).context("generate lockfile")?;
                lock.save(&repo.lockfile_path).context("write lockfile")?;
                steps.push(UpdateStep {
                    name: "lock".to_string(),
                    ok: true,
                    detail: serde_json::json!({
                        "lockfile": repo.lockfile_path.clone(),
                        "modules": lock.modules.len(),
                    }),
                });
                Some(lock)
            } else {
                None
            };

            let mut fetched = 0usize;
            if do_fetch {
                let lock = match lock {
                    Some(l) => l,
                    None => Lockfile::load(&repo.lockfile_path).context("load lockfile")?,
                };
                store.ensure_layout()?;

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

                steps.push(UpdateStep {
                    name: "fetch".to_string(),
                    ok: true,
                    detail: serde_json::json!({
                        "store": home.cache_dir.clone(),
                        "git_modules_fetched": fetched,
                    }),
                });
            }

            if cli.json {
                let envelope = JsonEnvelope::ok(
                    "update",
                    serde_json::json!({
                        "lockfile": repo.lockfile_path.clone(),
                        "store": home.cache_dir.clone(),
                        "steps": steps,
                        "git_modules_fetched": fetched,
                    }),
                );
                print_json(&envelope)?;
            } else if steps.is_empty() {
                println!("No steps to run");
            } else {
                for s in &steps {
                    println!("- {}", s.name);
                }
            }
        }
        Commands::Fetch => {
            require_yes_for_json_mutation(cli, "fetch")?;
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
                        "store": home.cache_dir,
                        "git_modules_fetched": fetched,
                    }),
                );
                print_json(&envelope)?;
            } else {
                println!(
                    "Fetched/verified {fetched} git module(s) into {}",
                    home.cache_dir.display()
                );
            }
        }
        Commands::Preview { diff } => {
            let engine = Engine::load(cli.repo.as_deref(), cli.machine.as_deref())?;
            let targets = selected_targets(&engine.manifest, &cli.target)?;
            let render = engine.desired_state(&cli.profile, &cli.target)?;
            let desired = render.desired;
            let mut warnings = render.warnings;
            let roots = render.roots;
            let managed_paths_from_manifest =
                crate::target_manifest::load_managed_paths_from_manifests(&roots)?;
            let managed_paths = if !managed_paths_from_manifest.is_empty() {
                Some(filter_managed(managed_paths_from_manifest, &cli.target))
            } else {
                latest_snapshot(&engine.home, &["deploy", "rollback"])?
                    .as_ref()
                    .map(load_managed_paths_from_snapshot)
                    .transpose()?
                    .map(|m| filter_managed(m, &cli.target))
            };

            let plan = compute_plan(&desired, managed_paths.as_ref())?;

            if cli.json {
                let plan_changes = plan.changes.clone();
                let plan_summary = plan.summary.clone();
                let mut data = serde_json::json!({
                    "profile": cli.profile,
                    "targets": targets,
                    "plan": {
                        "changes": plan_changes,
                        "summary": plan_summary,
                    },
                });
                if *diff {
                    let files = preview_diff_files(&plan, &desired, &roots, &mut warnings)?;
                    data["diff"] = serde_json::json!({
                        "changes": plan.changes,
                        "summary": plan.summary,
                        "files": files,
                    });
                }

                let mut envelope = JsonEnvelope::ok("preview", data);
                envelope.warnings = warnings;
                print_json(&envelope)?;
            } else {
                for w in warnings {
                    eprintln!("Warning: {w}");
                }
                println!(
                    "Plan: +{} ~{} -{}",
                    plan.summary.create, plan.summary.update, plan.summary.delete
                );
                if *diff {
                    print_diff(&plan, &desired)?;
                } else {
                    for c in &plan.changes {
                        println!("{:?} {} {}", c.op, c.target, c.path);
                    }
                }
            }
        }
        Commands::Overlay { command } => match command {
            OverlayCommands::Edit {
                module_id,
                scope,
                project,
            } => {
                require_yes_for_json_mutation(cli, "overlay edit")?;
                let engine = Engine::load(cli.repo.as_deref(), cli.machine.as_deref())?;
                let mut warnings: Vec<String> = Vec::new();
                let module_id_str = module_id.as_str();

                let mut effective_scope = *scope;
                if *project {
                    if *scope != OverlayScope::Global {
                        warnings.push(
                            "--project is deprecated; ignoring --scope and using project scope"
                                .to_string(),
                        );
                    }
                    effective_scope = OverlayScope::Project;
                }

                let overlay_dir = overlay_dir_for_scope(&engine, module_id_str, effective_scope);

                let skeleton = ensure_overlay_skeleton(
                    &engine.home,
                    &engine.repo,
                    &engine.manifest,
                    module_id_str,
                    &overlay_dir,
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
                    let mut envelope = JsonEnvelope::ok(
                        "overlay.edit",
                        serde_json::json!({
                            "module_id": module_id,
                            "scope": effective_scope,
                            "overlay_dir": skeleton.dir,
                            "created": skeleton.created,
                            "project": effective_scope == OverlayScope::Project,
                            "machine_id": if matches!(effective_scope, OverlayScope::Machine) { Some(engine.machine_id.clone()) } else { None },
                            "project_id": if matches!(effective_scope, OverlayScope::Project) { Some(engine.project.project_id.clone()) } else { None },
                        }),
                    );
                    envelope.warnings = warnings;
                    print_json(&envelope)?;
                } else {
                    for w in warnings {
                        eprintln!("Warning: {w}");
                    }
                    let status = if skeleton.created {
                        "Created"
                    } else {
                        "Overlay already exists at"
                    };
                    println!("{status} {}", skeleton.dir.display());
                }
            }
            OverlayCommands::Path { module_id, scope } => {
                let engine = Engine::load(cli.repo.as_deref(), cli.machine.as_deref())?;
                let module_id_str = module_id.as_str();

                let overlay_dir = overlay_dir_for_scope(&engine, module_id_str, *scope);

                if cli.json {
                    let envelope = JsonEnvelope::ok(
                        "overlay.path",
                        serde_json::json!({
                            "module_id": module_id,
                            "scope": scope,
                            "overlay_dir": overlay_dir,
                        }),
                    );
                    print_json(&envelope)?;
                } else {
                    println!("{}", overlay_dir.display());
                }
            }
        },
        Commands::Plan => {
            let engine = Engine::load(cli.repo.as_deref(), cli.machine.as_deref())?;
            let targets = selected_targets(&engine.manifest, &cli.target)?;
            let render = engine.desired_state(&cli.profile, &cli.target)?;
            let desired = render.desired;
            let warnings = render.warnings;
            let roots = render.roots;
            let managed_paths_from_manifest =
                crate::target_manifest::load_managed_paths_from_manifests(&roots)?;
            let managed_paths = if !managed_paths_from_manifest.is_empty() {
                Some(filter_managed(managed_paths_from_manifest, &cli.target))
            } else {
                latest_snapshot(&engine.home, &["deploy", "rollback"])?
                    .as_ref()
                    .map(load_managed_paths_from_snapshot)
                    .transpose()?
                    .map(|m| filter_managed(m, &cli.target))
            };

            let plan = compute_plan(&desired, managed_paths.as_ref())?;

            if cli.json {
                let mut envelope = JsonEnvelope::ok(
                    "plan",
                    serde_json::json!({
                        "profile": cli.profile,
                        "targets": targets,
                        "changes": plan.changes,
                        "summary": plan.summary,
                    }),
                );
                envelope.warnings = warnings;
                print_json(&envelope)?;
            } else {
                for w in warnings {
                    eprintln!("Warning: {w}");
                }
                println!(
                    "Plan: +{} ~{} -{}",
                    plan.summary.create, plan.summary.update, plan.summary.delete
                );
                for c in &plan.changes {
                    println!("{:?} {} {}", c.op, c.target, c.path);
                }
            }
        }
        Commands::Diff => {
            let engine = Engine::load(cli.repo.as_deref(), cli.machine.as_deref())?;
            let targets = selected_targets(&engine.manifest, &cli.target)?;
            let render = engine.desired_state(&cli.profile, &cli.target)?;
            let desired = render.desired;
            let warnings = render.warnings;
            let roots = render.roots;
            let managed_paths_from_manifest =
                crate::target_manifest::load_managed_paths_from_manifests(&roots)?;
            let managed_paths = if !managed_paths_from_manifest.is_empty() {
                Some(filter_managed(managed_paths_from_manifest, &cli.target))
            } else {
                latest_snapshot(&engine.home, &["deploy", "rollback"])?
                    .as_ref()
                    .map(load_managed_paths_from_snapshot)
                    .transpose()?
                    .map(|m| filter_managed(m, &cli.target))
            };
            let plan = compute_plan(&desired, managed_paths.as_ref())?;

            if cli.json {
                let mut envelope = JsonEnvelope::ok(
                    "diff",
                    serde_json::json!({
                        "profile": cli.profile,
                        "targets": targets,
                        "changes": plan.changes,
                        "summary": plan.summary,
                    }),
                );
                envelope.warnings = warnings;
                print_json(&envelope)?;
                return Ok(());
            }

            for w in warnings {
                eprintln!("Warning: {w}");
            }
            print_diff(&plan, &desired)?;
        }
        Commands::Deploy { apply } => {
            let engine = Engine::load(cli.repo.as_deref(), cli.machine.as_deref())?;
            let targets = selected_targets(&engine.manifest, &cli.target)?;
            let render = engine.desired_state(&cli.profile, &cli.target)?;
            let desired = render.desired;
            let warnings = render.warnings;
            let roots = render.roots;
            let managed_paths_from_manifest =
                crate::target_manifest::load_managed_paths_from_manifests(&roots)?;
            let managed_paths = if !managed_paths_from_manifest.is_empty() {
                Some(filter_managed(managed_paths_from_manifest, &cli.target))
            } else {
                latest_snapshot(&engine.home, &["deploy", "rollback"])?
                    .as_ref()
                    .map(load_managed_paths_from_snapshot)
                    .transpose()?
                    .map(|m| filter_managed(m, &cli.target))
            };
            let plan = compute_plan(&desired, managed_paths.as_ref())?;

            let will_apply = *apply && !cli.dry_run;

            if !cli.json {
                for w in &warnings {
                    eprintln!("Warning: {w}");
                }
                println!(
                    "Plan: +{} ~{} -{}",
                    plan.summary.create, plan.summary.update, plan.summary.delete
                );
                print_diff(&plan, &desired)?;
            }

            if !will_apply {
                if cli.json {
                    let mut envelope = JsonEnvelope::ok(
                        "deploy",
                        serde_json::json!({
                            "applied": false,
                            "profile": cli.profile,
                            "targets": targets,
                            "changes": plan.changes,
                            "summary": plan.summary,
                        }),
                    );
                    envelope.warnings = warnings;
                    print_json(&envelope)?;
                }
                return Ok(());
            }

            require_yes_for_json_mutation(cli, "deploy --apply")?;

            let needs_manifests = manifests_missing_for_desired(&roots, &desired);

            if plan.changes.is_empty() && !needs_manifests {
                if cli.json {
                    let mut envelope = JsonEnvelope::ok(
                        "deploy",
                        serde_json::json!({
                            "applied": false,
                            "reason": "no_changes",
                            "profile": cli.profile,
                            "targets": targets,
                            "changes": plan.changes,
                            "summary": plan.summary,
                        }),
                    );
                    envelope.warnings = warnings;
                    print_json(&envelope)?;
                } else {
                    println!("No changes");
                }
                return Ok(());
            }

            if !cli.yes && !cli.json && !confirm("Apply changes?")? {
                println!("Aborted");
                return Ok(());
            }

            let lockfile_path = if engine.repo.lockfile_path.exists() {
                Some(engine.repo.lockfile_path.as_path())
            } else {
                None
            };
            let snapshot = crate::apply::apply_plan(
                &engine.home,
                "deploy",
                &plan,
                &desired,
                lockfile_path,
                &roots,
            )?;

            if cli.json {
                let mut envelope = JsonEnvelope::ok(
                    "deploy",
                    serde_json::json!({
                        "applied": true,
                        "snapshot_id": snapshot.id,
                        "profile": cli.profile,
                        "targets": targets,
                        "changes": plan.changes,
                        "summary": plan.summary,
                    }),
                );
                envelope.warnings = warnings;
                print_json(&envelope)?;
            } else {
                println!("Applied. Snapshot: {}", snapshot.id);
            }
        }
        Commands::Status => {
            #[derive(serde::Serialize)]
            struct DriftItem {
                target: String,
                path: String,
                expected: Option<String>,
                actual: Option<String>,
                kind: String,
            }

            let engine = Engine::load(cli.repo.as_deref(), cli.machine.as_deref())?;
            let targets = selected_targets(&engine.manifest, &cli.target)?;
            let render = engine.desired_state(&cli.profile, &cli.target)?;
            let desired = render.desired;
            let mut warnings = render.warnings;
            let roots = render.roots;
            warn_operator_assets_if_outdated(&engine, &targets, &mut warnings)?;
            let managed_paths_from_manifest =
                crate::target_manifest::load_managed_paths_from_manifests(&roots)?;
            let managed_paths_from_manifest =
                filter_managed(managed_paths_from_manifest, &cli.target);

            let mut drift = Vec::new();
            if managed_paths_from_manifest.is_empty() {
                warnings.push("no target manifests found; drift may be inaccurate (run deploy --apply to write manifests)".to_string());
                for (tp, desired_file) in &desired {
                    let expected = format!("sha256:{}", sha256_hex(&desired_file.bytes));
                    match std::fs::read(&tp.path) {
                        Ok(actual_bytes) => {
                            let actual = format!("sha256:{}", sha256_hex(&actual_bytes));
                            if actual != expected {
                                drift.push(DriftItem {
                                    target: tp.target.clone(),
                                    path: tp.path.to_string_lossy().to_string(),
                                    expected: Some(expected),
                                    actual: Some(actual),
                                    kind: "modified".to_string(),
                                });
                            }
                        }
                        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                            drift.push(DriftItem {
                                target: tp.target.clone(),
                                path: tp.path.to_string_lossy().to_string(),
                                expected: Some(expected),
                                actual: None,
                                kind: "missing".to_string(),
                            })
                        }
                        Err(err) => return Err(err).context("read deployed file"),
                    }
                }
            } else {
                for tp in &managed_paths_from_manifest {
                    let expected = desired
                        .get(tp)
                        .map(|f| format!("sha256:{}", sha256_hex(&f.bytes)));
                    match std::fs::read(&tp.path) {
                        Ok(actual_bytes) => {
                            let actual = format!("sha256:{}", sha256_hex(&actual_bytes));
                            if let Some(exp) = &expected {
                                if &actual != exp {
                                    drift.push(DriftItem {
                                        target: tp.target.clone(),
                                        path: tp.path.to_string_lossy().to_string(),
                                        expected: Some(exp.clone()),
                                        actual: Some(actual),
                                        kind: "modified".to_string(),
                                    });
                                }
                            } else {
                                drift.push(DriftItem {
                                    target: tp.target.clone(),
                                    path: tp.path.to_string_lossy().to_string(),
                                    expected: None,
                                    actual: Some(actual),
                                    kind: "extra".to_string(),
                                });
                            }
                        }
                        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                            if let Some(exp) = expected {
                                drift.push(DriftItem {
                                    target: tp.target.clone(),
                                    path: tp.path.to_string_lossy().to_string(),
                                    expected: Some(exp),
                                    actual: None,
                                    kind: "missing".to_string(),
                                });
                            }
                        }
                        Err(err) => return Err(err).context("read deployed file"),
                    }
                }

                for root in &roots {
                    if !root.scan_extras {
                        continue;
                    }
                    if !root.root.exists() {
                        continue;
                    }

                    let mut files = crate::fs::list_files(&root.root)?;
                    files.sort();
                    for path in files {
                        if path.file_name().and_then(|s| s.to_str())
                            == Some(crate::target_manifest::TARGET_MANIFEST_FILENAME)
                        {
                            continue;
                        }

                        let tp = TargetPath {
                            target: root.target.clone(),
                            path: path.clone(),
                        };
                        if managed_paths_from_manifest.contains(&tp) {
                            continue;
                        }

                        drift.push(DriftItem {
                            target: tp.target.clone(),
                            path: tp.path.to_string_lossy().to_string(),
                            expected: None,
                            actual: Some(format!(
                                "sha256:{}",
                                sha256_hex(&std::fs::read(&tp.path)?)
                            )),
                            kind: "extra".to_string(),
                        });
                    }
                }
            }

            if cli.json {
                let mut envelope = JsonEnvelope::ok(
                    "status",
                    serde_json::json!({
                        "profile": cli.profile,
                        "targets": targets,
                        "drift": drift,
                    }),
                );
                envelope.warnings = warnings;
                print_json(&envelope)?;
            } else if drift.is_empty() {
                for w in warnings {
                    eprintln!("Warning: {w}");
                }
                println!("No drift");
            } else {
                for w in warnings {
                    eprintln!("Warning: {w}");
                }
                println!("Drift ({}):", drift.len());
                for d in drift {
                    println!("{} {} {}", d.kind, d.target, d.path);
                }
            }
        }
        Commands::Doctor { fix } => {
            #[derive(serde::Serialize)]
            struct DoctorRootCheck {
                target: String,
                root: String,
                exists: bool,
                writable: bool,
                scan_extras: bool,
                issues: Vec<String>,
                suggestion: Option<String>,
            }

            #[derive(Debug, Clone, serde::Serialize)]
            struct DoctorGitignoreFix {
                repo_root: String,
                gitignore_path: String,
                updated: bool,
            }

            fn git_repo_root(dir: &std::path::Path) -> Option<std::path::PathBuf> {
                let out = std::process::Command::new("git")
                    .current_dir(dir)
                    .args(["rev-parse", "--show-toplevel"])
                    .output()
                    .ok()?;
                if !out.status.success() {
                    return None;
                }
                let root = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if root.is_empty() {
                    None
                } else {
                    Some(std::path::PathBuf::from(root))
                }
            }

            fn git_is_ignored(repo_root: &std::path::Path, rel: &std::path::Path) -> bool {
                let rel = rel.to_string_lossy().replace('\\', "/");
                let out = std::process::Command::new("git")
                    .current_dir(repo_root)
                    .args(["check-ignore", "-q", rel.as_str()])
                    .output();
                match out {
                    Ok(out) if out.status.success() => true,
                    Ok(out) if out.status.code() == Some(1) => false,
                    _ => false,
                }
            }

            fn ensure_gitignore_contains(
                repo_root: &std::path::Path,
                line: &str,
            ) -> anyhow::Result<bool> {
                let gitignore_path = repo_root.join(".gitignore");
                let mut contents = std::fs::read_to_string(&gitignore_path).unwrap_or_default();
                let already = contents.lines().any(|l| l.trim() == line);
                if already {
                    return Ok(false);
                }

                if !contents.is_empty() && !contents.ends_with('\n') {
                    contents.push('\n');
                }
                contents.push_str(line);
                contents.push('\n');
                write_atomic(&gitignore_path, contents.as_bytes())
                    .with_context(|| format!("write {}", gitignore_path.display()))?;
                Ok(true)
            }

            fn overlay_layout_warnings(engine: &Engine) -> Vec<String> {
                fn rel(repo_dir: &std::path::Path, path: &std::path::Path) -> String {
                    path.strip_prefix(repo_dir)
                        .unwrap_or(path)
                        .to_string_lossy()
                        .to_string()
                }

                fn dirs_for_scope(
                    engine: &Engine,
                    module_id: &str,
                    scope: OverlayScope,
                ) -> (
                    std::path::PathBuf,
                    Option<std::path::PathBuf>,
                    Option<std::path::PathBuf>,
                ) {
                    let bounded = crate::ids::module_fs_key(module_id);
                    let canonical = match scope {
                        OverlayScope::Global => {
                            engine.repo.repo_dir.join("overlays").join(&bounded)
                        }
                        OverlayScope::Machine => engine
                            .repo
                            .repo_dir
                            .join("overlays/machines")
                            .join(&engine.machine_id)
                            .join(&bounded),
                        OverlayScope::Project => engine
                            .repo
                            .repo_dir
                            .join("projects")
                            .join(&engine.project.project_id)
                            .join("overlays")
                            .join(&bounded),
                    };

                    let legacy_fs_key = crate::ids::module_fs_key_unbounded(module_id);
                    let legacy_fs_key = (legacy_fs_key != bounded).then(|| match scope {
                        OverlayScope::Global => {
                            engine.repo.repo_dir.join("overlays").join(&legacy_fs_key)
                        }
                        OverlayScope::Machine => engine
                            .repo
                            .repo_dir
                            .join("overlays/machines")
                            .join(&engine.machine_id)
                            .join(&legacy_fs_key),
                        OverlayScope::Project => engine
                            .repo
                            .repo_dir
                            .join("projects")
                            .join(&engine.project.project_id)
                            .join("overlays")
                            .join(&legacy_fs_key),
                    });

                    let legacy =
                        crate::ids::is_safe_legacy_path_component(module_id).then(|| match scope {
                            OverlayScope::Global => {
                                engine.repo.repo_dir.join("overlays").join(module_id)
                            }
                            OverlayScope::Machine => engine
                                .repo
                                .repo_dir
                                .join("overlays/machines")
                                .join(&engine.machine_id)
                                .join(module_id),
                            OverlayScope::Project => engine
                                .repo
                                .repo_dir
                                .join("projects")
                                .join(&engine.project.project_id)
                                .join("overlays")
                                .join(module_id),
                        });

                    (canonical, legacy_fs_key, legacy)
                }

                let mut warnings = Vec::new();
                for module in &engine.manifest.modules {
                    let module_id = module.id.as_str();
                    for scope in [
                        OverlayScope::Global,
                        OverlayScope::Machine,
                        OverlayScope::Project,
                    ] {
                        let scope_name = match scope {
                            OverlayScope::Global => "global",
                            OverlayScope::Machine => "machine",
                            OverlayScope::Project => "project",
                        };

                        let (canonical, legacy_fs_key, legacy) =
                            dirs_for_scope(engine, module_id, scope);

                        let mut existing: Vec<std::path::PathBuf> = Vec::new();
                        if canonical.exists() {
                            existing.push(canonical.clone());
                        }
                        if let Some(p) = legacy_fs_key.clone() {
                            if p.exists() {
                                existing.push(p);
                            }
                        }
                        if let Some(p) = legacy.clone() {
                            if p.exists() {
                                existing.push(p);
                            }
                        }

                        if existing.len() >= 2 {
                            let found = existing
                                .iter()
                                .map(|p| rel(&engine.repo.repo_dir, p))
                                .collect::<Vec<_>>()
                                .join(", ");
                            let selected = overlay_dir_for_scope(engine, module_id, scope);
                            warnings.push(format!(
                                "overlay layout ({scope_name}) module {module_id}: multiple overlay dirs exist: {found}; agentpack will use {} (consider migrating/removing legacy dirs)",
                                rel(&engine.repo.repo_dir, &selected)
                            ));
                        }

                        for dir in existing {
                            if !dir.is_dir() {
                                warnings.push(format!(
                                    "overlay layout ({scope_name}) module {module_id}: {} exists but is not a directory",
                                    rel(&engine.repo.repo_dir, &dir)
                                ));
                                continue;
                            }

                            let meta_path = dir.join(".agentpack").join("module_id");
                            if !meta_path.exists() {
                                continue;
                            }
                            let raw = match std::fs::read_to_string(&meta_path) {
                                Ok(s) => s,
                                Err(err) => {
                                    warnings.push(format!(
                                        "overlay metadata ({scope_name}) module {module_id}: failed to read {}: {err}",
                                        rel(&engine.repo.repo_dir, &meta_path)
                                    ));
                                    continue;
                                }
                            };
                            let got = raw.trim_end();
                            if got != module_id {
                                warnings.push(format!(
                                    "overlay metadata ({scope_name}) module {module_id}: {} contains {:?} (expected {:?})",
                                    rel(&engine.repo.repo_dir, &meta_path),
                                    got,
                                    module_id
                                ));
                            }
                        }
                    }
                }

                warnings
            }

            let engine = Engine::load(cli.repo.as_deref(), cli.machine.as_deref())?;
            let render = engine.desired_state(&cli.profile, &cli.target)?;
            let mut warnings = render.warnings;
            warnings.extend(overlay_layout_warnings(&engine));

            let mut checks = Vec::new();
            let mut repos_to_fix: std::collections::BTreeSet<std::path::PathBuf> =
                std::collections::BTreeSet::new();
            for root in render.roots {
                let mut issues = Vec::new();
                let exists = root.root.exists();
                let is_dir = root.root.is_dir();

                if !exists {
                    issues.push("missing".to_string());
                } else if !is_dir {
                    issues.push("not_a_directory".to_string());
                }

                let writable = exists && is_dir && dir_is_writable(&root.root);
                if exists && is_dir && !writable {
                    issues.push("not_writable".to_string());
                }

                let suggestion = if !exists {
                    Some(format!(
                        "create directory: mkdir -p {}",
                        root.root.display()
                    ))
                } else if exists && is_dir && !writable {
                    Some("fix permissions (directory not writable)".to_string())
                } else {
                    None
                };

                if exists && is_dir {
                    if let Some(repo_root) = git_repo_root(&root.root) {
                        let manifest_path = root.root.join(".agentpack.manifest.json");
                        let rel = manifest_path
                            .strip_prefix(&repo_root)
                            .unwrap_or(manifest_path.as_path());
                        let ignored = git_is_ignored(&repo_root, rel);
                        if !ignored {
                            warnings.push(format!(
                                "target root is in a git repo and `.agentpack.manifest.json` is not ignored: root={} repo={}; consider adding it to .gitignore (or run `agentpack doctor --fix`)",
                                root.root.display(),
                                repo_root.display(),
                            ));
                            repos_to_fix.insert(repo_root);
                        }
                    }
                }

                checks.push(DoctorRootCheck {
                    target: root.target,
                    root: root.root.to_string_lossy().to_string(),
                    exists,
                    writable,
                    scan_extras: root.scan_extras,
                    issues,
                    suggestion,
                });
            }

            let mut gitignore_fixes: Vec<DoctorGitignoreFix> = Vec::new();
            if *fix && !repos_to_fix.is_empty() {
                if cli.json && !cli.yes {
                    return Err(UserError::confirm_required("doctor --fix"));
                }
                for repo_root in &repos_to_fix {
                    let updated = ensure_gitignore_contains(repo_root, ".agentpack.manifest.json")
                        .context("update .gitignore")?;
                    gitignore_fixes.push(DoctorGitignoreFix {
                        repo_root: repo_root.display().to_string(),
                        gitignore_path: repo_root.join(".gitignore").display().to_string(),
                        updated,
                    });
                }
            }

            if cli.json {
                let mut envelope = JsonEnvelope::ok(
                    "doctor",
                    serde_json::json!({
                        "machine_id": engine.machine_id,
                        "roots": checks,
                        "gitignore_fixes": gitignore_fixes,
                    }),
                );
                envelope.warnings = warnings;
                print_json(&envelope)?;
            } else {
                for w in warnings.drain(..) {
                    eprintln!("Warning: {w}");
                }
                println!("Machine ID: {}", engine.machine_id);
                if *fix {
                    for f in &gitignore_fixes {
                        if f.updated {
                            println!(
                                "Updated {} (added .agentpack.manifest.json)",
                                f.gitignore_path
                            );
                        }
                    }
                }
                for c in checks {
                    let status = if c.issues.is_empty() { "ok" } else { "issues" };
                    println!("- {} {} ({status})", c.target, c.root,);
                    for issue in c.issues {
                        println!("  - issue: {issue}");
                    }
                    if let Some(s) = c.suggestion {
                        println!("  - suggestion: {s}");
                    }
                }
            }
        }
        Commands::Remote { command } => match command {
            RemoteCommands::Set { url, name } => {
                require_yes_for_json_mutation(cli, "remote set")?;
                let repo_dir = repo.repo_dir.as_path();
                if !repo_dir.join(".git").exists() {
                    let _ = crate::git::git_in(repo_dir, &["init"])?;
                }

                let has_remote =
                    crate::git::git_in(repo_dir, &["remote", "get-url", name.as_str()]).is_ok();
                if has_remote {
                    let _ = crate::git::git_in(
                        repo_dir,
                        &["remote", "set-url", name.as_str(), url.as_str()],
                    )?;
                } else {
                    let _ = crate::git::git_in(
                        repo_dir,
                        &["remote", "add", name.as_str(), url.as_str()],
                    )?;
                }

                if cli.json {
                    let envelope = JsonEnvelope::ok(
                        "remote.set",
                        serde_json::json!({
                            "repo": repo_dir.display().to_string(),
                            "remote": name,
                            "url": url,
                        }),
                    );
                    print_json(&envelope)?;
                } else {
                    println!("Set remote {} -> {}", name, url);
                }
            }
        },
        Commands::Sync { rebase, remote } => {
            require_yes_for_json_mutation(cli, "sync")?;
            let repo_dir = repo.repo_dir.as_path();
            if !repo_dir.join(".git").exists() {
                anyhow::bail!(
                    "config repo is not a git repository: {}",
                    repo_dir.display()
                );
            }

            let status = crate::git::git_in(repo_dir, &["status", "--porcelain"])?;
            if !status.trim().is_empty() {
                anyhow::bail!("refusing to sync with a dirty working tree (commit or stash first)");
            }

            let branch = crate::git::git_in(repo_dir, &["rev-parse", "--abbrev-ref", "HEAD"])?;
            let branch = branch.trim();
            if branch == "HEAD" {
                anyhow::bail!("refusing to sync on detached HEAD");
            }

            // Ensure remote exists.
            let _ = crate::git::git_in(repo_dir, &["remote", "get-url", remote.as_str()])?;

            let mut ran = Vec::new();
            if *rebase {
                ran.push(format!("git pull --rebase {} {}", remote, branch));
                let _ =
                    crate::git::git_in(repo_dir, &["pull", "--rebase", remote.as_str(), branch])?;
            } else {
                ran.push(format!("git pull {} {}", remote, branch));
                let _ = crate::git::git_in(repo_dir, &["pull", remote.as_str(), branch])?;
            }

            ran.push(format!("git push {} {}", remote, branch));
            let _ = crate::git::git_in(repo_dir, &["push", remote.as_str(), branch])?;

            if cli.json {
                let envelope = JsonEnvelope::ok(
                    "sync",
                    serde_json::json!({
                        "repo": repo_dir.display().to_string(),
                        "remote": remote,
                        "branch": branch,
                        "rebase": rebase,
                        "commands": ran,
                    }),
                );
                print_json(&envelope)?;
            } else {
                println!("Synced {} ({} {})", repo_dir.display(), remote, branch);
            }
        }
        Commands::Record => {
            require_yes_for_json_mutation(cli, "record")?;
            let event = crate::events::read_stdin_event()?;
            let machine_id = crate::machine::detect_machine_id()?;
            let record = crate::events::new_record(machine_id.clone(), event)?;
            let path = crate::events::append_event(&home, &record)?;

            if cli.json {
                let envelope = JsonEnvelope::ok(
                    "record",
                    serde_json::json!({
                        "path": path,
                        "recorded_at": record.recorded_at,
                        "machine_id": record.machine_id,
                    }),
                );
                print_json(&envelope)?;
            } else {
                println!("Recorded event to {}", path.display());
            }
        }
        Commands::Score => {
            #[derive(Debug, Clone, serde::Serialize)]
            struct ModuleScore {
                module_id: String,
                total: u64,
                failures: u64,
                failure_rate: Option<f64>,
                last_seen_at: Option<String>,
            }

            let mut scores: std::collections::BTreeMap<
                String,
                (u64, u64, Option<String>), // total, failures, last_seen_at
            > = std::collections::BTreeMap::new();

            let read = crate::events::read_events_with_warnings(&home)?;
            for evt in read.events {
                let Some(module_id) = evt
                    .module_id
                    .clone()
                    .or_else(|| crate::events::event_module_id(&evt.event))
                else {
                    continue;
                };
                let success = evt
                    .success
                    .or_else(|| crate::events::event_success(&evt.event))
                    .unwrap_or(true);

                let entry = scores.entry(module_id).or_insert((0, 0, None));
                entry.0 += 1;
                if !success {
                    entry.1 += 1;
                }
                let update_last = match &entry.2 {
                    None => true,
                    Some(prev) => prev < &evt.recorded_at,
                };
                if update_last {
                    entry.2 = Some(evt.recorded_at);
                }
            }

            if let Ok(manifest) = Manifest::load(&repo.manifest_path) {
                for m in manifest.modules {
                    scores.entry(m.id).or_insert((0, 0, None));
                }
            }

            let mut out: Vec<ModuleScore> = scores
                .into_iter()
                .map(|(module_id, (total, failures, last_seen_at))| ModuleScore {
                    module_id,
                    total,
                    failures,
                    failure_rate: if total == 0 {
                        None
                    } else {
                        Some((failures as f64) / (total as f64))
                    },
                    last_seen_at,
                })
                .collect();
            out.sort_by(|a, b| {
                cmp_failure_rate(a.failures, a.total, b.failures, b.total)
                    .then_with(|| a.module_id.cmp(&b.module_id))
            });

            if cli.json {
                let mut envelope = JsonEnvelope::ok("score", serde_json::json!({ "modules": out }));
                envelope.warnings = read.warnings;
                print_json(&envelope)?;
            } else if out.is_empty() {
                for w in read.warnings {
                    eprintln!("Warning: {w}");
                }
                println!("No events recorded yet");
            } else {
                for w in read.warnings {
                    eprintln!("Warning: {w}");
                }
                for s in out {
                    let rate = s
                        .failure_rate
                        .map(|r| format!("{:.1}%", r * 100.0))
                        .unwrap_or_else(|| "-".to_string());
                    println!(
                        "- {} failures={}/{} rate={} last_seen={}",
                        s.module_id,
                        s.failures,
                        s.total,
                        rate,
                        s.last_seen_at.as_deref().unwrap_or("-")
                    );
                }
            }
        }
        Commands::Explain { command } => {
            let engine = Engine::load(cli.repo.as_deref(), cli.machine.as_deref())?;
            match command {
                ExplainCommands::Plan => explain_plan(cli, &engine)?,
                ExplainCommands::Diff => explain_plan(cli, &engine)?,
                ExplainCommands::Status => explain_status(cli, &engine)?,
            }
        }
        Commands::Evolve { command } => {
            let engine = Engine::load(cli.repo.as_deref(), cli.machine.as_deref())?;
            match command {
                EvolveCommands::Propose {
                    module_id,
                    scope,
                    branch,
                } => evolve_propose(
                    cli,
                    &engine,
                    module_id.as_deref(),
                    *scope,
                    branch.as_deref(),
                )?,
            }
        }
        Commands::Completions { shell } => {
            if cli.json {
                anyhow::bail!("completions does not support --json");
            }
            let mut cmd = Cli::command();
            clap_complete::generate(*shell, &mut cmd, "agentpack", &mut std::io::stdout());
        }
        Commands::Bootstrap { scope } => {
            let engine = Engine::load(cli.repo.as_deref(), cli.machine.as_deref())?;
            let targets = selected_targets(&engine.manifest, &cli.target)?;
            let (allow_user, allow_project) = bootstrap_scope_flags(*scope);
            let scope_str = bootstrap_scope_str(*scope);

            let mut desired = crate::deploy::DesiredState::new();
            let mut roots: Vec<crate::targets::TargetRoot> = Vec::new();

            if targets.iter().any(|t| t == "codex") {
                let codex_home = codex_home_for_manifest(&engine.manifest)?;
                let bytes = render_operator_template_bytes(TEMPLATE_CODEX_OPERATOR_SKILL);

                if allow_user {
                    desired.insert(
                        TargetPath {
                            target: "codex".to_string(),
                            path: codex_home.join("skills/agentpack-operator/SKILL.md"),
                        },
                        crate::deploy::DesiredFile {
                            bytes: bytes.clone(),
                            module_ids: vec!["skill:agentpack-operator".to_string()],
                        },
                    );
                    roots.push(crate::targets::TargetRoot {
                        target: "codex".to_string(),
                        root: codex_home.join("skills"),
                        scan_extras: true,
                    });
                }
                if allow_project {
                    desired.insert(
                        TargetPath {
                            target: "codex".to_string(),
                            path: engine
                                .project
                                .project_root
                                .join(".codex/skills/agentpack-operator/SKILL.md"),
                        },
                        crate::deploy::DesiredFile {
                            bytes: bytes.clone(),
                            module_ids: vec!["skill:agentpack-operator".to_string()],
                        },
                    );
                    roots.push(crate::targets::TargetRoot {
                        target: "codex".to_string(),
                        root: engine.project.project_root.join(".codex/skills"),
                        scan_extras: true,
                    });
                }
            }

            if targets.iter().any(|t| t == "claude_code") {
                let bytes_doctor = render_operator_template_bytes(TEMPLATE_CLAUDE_AP_DOCTOR);
                let bytes_update = render_operator_template_bytes(TEMPLATE_CLAUDE_AP_UPDATE);
                let bytes_preview = render_operator_template_bytes(TEMPLATE_CLAUDE_AP_PREVIEW);
                let bytes_plan = render_operator_template_bytes(TEMPLATE_CLAUDE_AP_PLAN);
                let bytes_deploy = render_operator_template_bytes(TEMPLATE_CLAUDE_AP_DEPLOY);
                let bytes_status = render_operator_template_bytes(TEMPLATE_CLAUDE_AP_STATUS);
                let bytes_diff = render_operator_template_bytes(TEMPLATE_CLAUDE_AP_DIFF);
                let bytes_explain = render_operator_template_bytes(TEMPLATE_CLAUDE_AP_EXPLAIN);
                let bytes_evolve = render_operator_template_bytes(TEMPLATE_CLAUDE_AP_EVOLVE);

                if allow_user {
                    let user_dir = expand_tilde("~/.claude/commands")?;
                    roots.push(crate::targets::TargetRoot {
                        target: "claude_code".to_string(),
                        root: user_dir.clone(),
                        scan_extras: true,
                    });
                    desired.insert(
                        TargetPath {
                            target: "claude_code".to_string(),
                            path: user_dir.join("ap-doctor.md"),
                        },
                        crate::deploy::DesiredFile {
                            bytes: bytes_doctor.clone(),
                            module_ids: vec!["command:ap-doctor".to_string()],
                        },
                    );
                    desired.insert(
                        TargetPath {
                            target: "claude_code".to_string(),
                            path: user_dir.join("ap-update.md"),
                        },
                        crate::deploy::DesiredFile {
                            bytes: bytes_update.clone(),
                            module_ids: vec!["command:ap-update".to_string()],
                        },
                    );
                    desired.insert(
                        TargetPath {
                            target: "claude_code".to_string(),
                            path: user_dir.join("ap-preview.md"),
                        },
                        crate::deploy::DesiredFile {
                            bytes: bytes_preview.clone(),
                            module_ids: vec!["command:ap-preview".to_string()],
                        },
                    );
                    desired.insert(
                        TargetPath {
                            target: "claude_code".to_string(),
                            path: user_dir.join("ap-plan.md"),
                        },
                        crate::deploy::DesiredFile {
                            bytes: bytes_plan.clone(),
                            module_ids: vec!["command:ap-plan".to_string()],
                        },
                    );
                    desired.insert(
                        TargetPath {
                            target: "claude_code".to_string(),
                            path: user_dir.join("ap-deploy.md"),
                        },
                        crate::deploy::DesiredFile {
                            bytes: bytes_deploy.clone(),
                            module_ids: vec!["command:ap-deploy".to_string()],
                        },
                    );
                    desired.insert(
                        TargetPath {
                            target: "claude_code".to_string(),
                            path: user_dir.join("ap-status.md"),
                        },
                        crate::deploy::DesiredFile {
                            bytes: bytes_status.clone(),
                            module_ids: vec!["command:ap-status".to_string()],
                        },
                    );
                    desired.insert(
                        TargetPath {
                            target: "claude_code".to_string(),
                            path: user_dir.join("ap-diff.md"),
                        },
                        crate::deploy::DesiredFile {
                            bytes: bytes_diff.clone(),
                            module_ids: vec!["command:ap-diff".to_string()],
                        },
                    );
                    desired.insert(
                        TargetPath {
                            target: "claude_code".to_string(),
                            path: user_dir.join("ap-explain.md"),
                        },
                        crate::deploy::DesiredFile {
                            bytes: bytes_explain.clone(),
                            module_ids: vec!["command:ap-explain".to_string()],
                        },
                    );
                    desired.insert(
                        TargetPath {
                            target: "claude_code".to_string(),
                            path: user_dir.join("ap-evolve.md"),
                        },
                        crate::deploy::DesiredFile {
                            bytes: bytes_evolve.clone(),
                            module_ids: vec!["command:ap-evolve".to_string()],
                        },
                    );
                }

                if allow_project {
                    let repo_dir = engine.project.project_root.join(".claude/commands");
                    roots.push(crate::targets::TargetRoot {
                        target: "claude_code".to_string(),
                        root: repo_dir.clone(),
                        scan_extras: true,
                    });
                    desired.insert(
                        TargetPath {
                            target: "claude_code".to_string(),
                            path: repo_dir.join("ap-doctor.md"),
                        },
                        crate::deploy::DesiredFile {
                            bytes: bytes_doctor,
                            module_ids: vec!["command:ap-doctor".to_string()],
                        },
                    );
                    desired.insert(
                        TargetPath {
                            target: "claude_code".to_string(),
                            path: repo_dir.join("ap-update.md"),
                        },
                        crate::deploy::DesiredFile {
                            bytes: bytes_update,
                            module_ids: vec!["command:ap-update".to_string()],
                        },
                    );
                    desired.insert(
                        TargetPath {
                            target: "claude_code".to_string(),
                            path: repo_dir.join("ap-preview.md"),
                        },
                        crate::deploy::DesiredFile {
                            bytes: bytes_preview,
                            module_ids: vec!["command:ap-preview".to_string()],
                        },
                    );
                    desired.insert(
                        TargetPath {
                            target: "claude_code".to_string(),
                            path: repo_dir.join("ap-plan.md"),
                        },
                        crate::deploy::DesiredFile {
                            bytes: bytes_plan,
                            module_ids: vec!["command:ap-plan".to_string()],
                        },
                    );
                    desired.insert(
                        TargetPath {
                            target: "claude_code".to_string(),
                            path: repo_dir.join("ap-deploy.md"),
                        },
                        crate::deploy::DesiredFile {
                            bytes: bytes_deploy,
                            module_ids: vec!["command:ap-deploy".to_string()],
                        },
                    );
                    desired.insert(
                        TargetPath {
                            target: "claude_code".to_string(),
                            path: repo_dir.join("ap-status.md"),
                        },
                        crate::deploy::DesiredFile {
                            bytes: bytes_status,
                            module_ids: vec!["command:ap-status".to_string()],
                        },
                    );
                    desired.insert(
                        TargetPath {
                            target: "claude_code".to_string(),
                            path: repo_dir.join("ap-diff.md"),
                        },
                        crate::deploy::DesiredFile {
                            bytes: bytes_diff,
                            module_ids: vec!["command:ap-diff".to_string()],
                        },
                    );
                    desired.insert(
                        TargetPath {
                            target: "claude_code".to_string(),
                            path: repo_dir.join("ap-explain.md"),
                        },
                        crate::deploy::DesiredFile {
                            bytes: bytes_explain,
                            module_ids: vec!["command:ap-explain".to_string()],
                        },
                    );
                    desired.insert(
                        TargetPath {
                            target: "claude_code".to_string(),
                            path: repo_dir.join("ap-evolve.md"),
                        },
                        crate::deploy::DesiredFile {
                            bytes: bytes_evolve,
                            module_ids: vec!["command:ap-evolve".to_string()],
                        },
                    );
                }
            }

            let plan = compute_plan(&desired, None)?;

            if !cli.json {
                println!(
                    "Plan: +{} ~{} -{}",
                    plan.summary.create, plan.summary.update, plan.summary.delete
                );
                print_diff(&plan, &desired)?;
            }

            if cli.dry_run {
                if cli.json {
                    let envelope = JsonEnvelope::ok(
                        "bootstrap",
                        serde_json::json!({
                            "applied": false,
                            "reason": "dry_run",
                            "targets": targets,
                            "scope": scope_str,
                            "changes": plan.changes,
                            "summary": plan.summary,
                        }),
                    );
                    print_json(&envelope)?;
                }
                return Ok(());
            }

            if plan.changes.is_empty() {
                if cli.json {
                    let envelope = JsonEnvelope::ok(
                        "bootstrap",
                        serde_json::json!({
                            "applied": false,
                            "reason": "no_changes",
                            "targets": targets,
                            "scope": scope_str,
                            "changes": plan.changes,
                            "summary": plan.summary,
                        }),
                    );
                    print_json(&envelope)?;
                } else {
                    println!("No changes");
                }
                return Ok(());
            }

            if cli.json && !cli.yes {
                return Err(UserError::confirm_required("bootstrap"));
            }

            if !cli.yes && !cli.json && !confirm("Apply bootstrap changes?")? {
                println!("Aborted");
                return Ok(());
            }

            let snapshot =
                crate::apply::apply_plan(&engine.home, "bootstrap", &plan, &desired, None, &roots)?;
            if cli.json {
                let envelope = JsonEnvelope::ok(
                    "bootstrap",
                    serde_json::json!({
                        "applied": true,
                        "snapshot_id": snapshot.id,
                        "targets": targets,
                        "scope": scope_str,
                        "changes": plan.changes,
                        "summary": plan.summary,
                    }),
                );
                print_json(&envelope)?;
            } else {
                println!("Bootstrapped. Snapshot: {}", snapshot.id);
            }
        }
        Commands::Rollback { to } => {
            require_yes_for_json_mutation(cli, "rollback")?;
            let event = crate::apply::rollback(&home, to).context("rollback")?;
            if cli.json {
                let envelope = JsonEnvelope::ok(
                    "rollback",
                    serde_json::json!({
                        "rolled_back_to": to,
                        "event_snapshot_id": event.id,
                    }),
                );
                print_json(&envelope)?;
            } else {
                println!("Rolled back to snapshot {to}. Event: {}", event.id);
            }
        }
    }

    Ok(())
}

#[derive(serde::Serialize)]
struct PreviewDiffFile {
    target: String,
    root: String,
    path: String,
    op: crate::deploy::Op,
    before_hash: Option<String>,
    after_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    unified: Option<String>,
}

fn preview_diff_files(
    plan: &crate::deploy::PlanResult,
    desired: &crate::deploy::DesiredState,
    roots: &[crate::targets::TargetRoot],
    warnings: &mut Vec<String>,
) -> anyhow::Result<Vec<PreviewDiffFile>> {
    let mut out = Vec::new();

    for c in &plan.changes {
        let abs_path = std::path::PathBuf::from(&c.path);
        let root_idx = best_root_idx(roots, &c.target, &abs_path);
        let root = root_idx
            .and_then(|idx| roots.get(idx))
            .map(|r| r.root.display().to_string())
            .unwrap_or_else(|| "<unknown>".to_string());

        let rel_path = root_idx
            .and_then(|idx| roots.get(idx))
            .and_then(|r| abs_path.strip_prefix(&r.root).ok())
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| c.path.clone());

        let before_hash = c.before_sha256.as_ref().map(|h| format!("sha256:{h}"));
        let after_hash = c.after_sha256.as_ref().map(|h| format!("sha256:{h}"));

        let mut unified: Option<String> = None;
        if matches!(c.op, crate::deploy::Op::Create | crate::deploy::Op::Update) {
            let before_bytes = std::fs::read(&abs_path).unwrap_or_default();
            let tp = TargetPath {
                target: c.target.clone(),
                path: abs_path.clone(),
            };
            if let Some(df) = desired.get(&tp) {
                match (
                    std::str::from_utf8(&before_bytes).ok(),
                    std::str::from_utf8(&df.bytes).ok(),
                ) {
                    (Some(from), Some(to)) => {
                        let from_name = format!("a/{rel_path}");
                        let to_name = format!("b/{rel_path}");
                        let diff = unified_diff(from, to, &from_name, &to_name);
                        if diff.len() > UNIFIED_DIFF_MAX_BYTES {
                            warnings.push(format!(
                                "preview diff omitted for {} {} (over {} bytes)",
                                c.target, rel_path, UNIFIED_DIFF_MAX_BYTES
                            ));
                        } else {
                            unified = Some(diff);
                        }
                    }
                    _ => {
                        warnings.push(format!(
                            "preview diff omitted for {} {} (binary or non-utf8)",
                            c.target, rel_path
                        ));
                    }
                }
            }
        }

        out.push(PreviewDiffFile {
            target: c.target.clone(),
            root,
            path: rel_path,
            op: c.op.clone(),
            before_hash,
            after_hash,
            unified,
        });
    }

    Ok(out)
}

fn filter_managed(
    managed: crate::deploy::ManagedPaths,
    target_filter: &str,
) -> crate::deploy::ManagedPaths {
    managed
        .into_iter()
        .filter(|tp| target_filter == "all" || tp.target == target_filter)
        .collect()
}

fn manifests_missing_for_desired(
    roots: &[crate::targets::TargetRoot],
    desired: &crate::deploy::DesiredState,
) -> bool {
    if roots.is_empty() {
        return false;
    }

    let mut used: Vec<bool> = vec![false; roots.len()];
    for tp in desired.keys() {
        if let Some(idx) = best_root_idx(roots, &tp.target, &tp.path) {
            used[idx] = true;
        }
    }

    for (idx, root) in roots.iter().enumerate() {
        if used[idx] && !crate::target_manifest::manifest_path(&root.root).exists() {
            return true;
        }
    }

    false
}

fn best_root_idx(
    roots: &[crate::targets::TargetRoot],
    target: &str,
    path: &std::path::Path,
) -> Option<usize> {
    roots
        .iter()
        .enumerate()
        .filter(|(_, r)| r.target == target)
        .filter(|(_, r)| path.strip_prefix(&r.root).is_ok())
        .max_by_key(|(_, r)| r.root.components().count())
        .map(|(idx, _)| idx)
}

fn cmp_failure_rate(a_fail: u64, a_total: u64, b_fail: u64, b_total: u64) -> std::cmp::Ordering {
    match (a_total == 0, b_total == 0) {
        (true, true) => std::cmp::Ordering::Equal,
        (true, false) => std::cmp::Ordering::Greater,
        (false, true) => std::cmp::Ordering::Less,
        (false, false) => {
            let left = (a_fail as u128) * (b_total as u128);
            let right = (b_fail as u128) * (a_total as u128);
            right.cmp(&left)
        }
    }
}

fn load_manifest_module_ids(
    roots: &[crate::targets::TargetRoot],
) -> anyhow::Result<std::collections::BTreeMap<TargetPath, Vec<String>>> {
    let mut out = std::collections::BTreeMap::new();
    for root in roots {
        let path = crate::target_manifest::manifest_path(&root.root);
        if !path.exists() {
            continue;
        }
        let manifest = crate::target_manifest::TargetManifest::load(&path)?;
        for f in manifest.managed_files {
            if std::path::Path::new(&f.path).is_absolute() {
                continue;
            }
            if std::path::Path::new(&f.path)
                .components()
                .any(|c| matches!(c, std::path::Component::ParentDir))
            {
                continue;
            }
            out.insert(
                TargetPath {
                    target: root.target.clone(),
                    path: root.root.join(&f.path),
                },
                f.module_ids,
            );
        }
    }
    Ok(out)
}

fn module_name_from_id(module_id: &str) -> String {
    module_id
        .split_once(':')
        .map(|(_, name)| name.to_string())
        .unwrap_or_else(|| crate::store::sanitize_module_id(module_id))
}

fn module_rel_path_for_output(
    module: &Module,
    module_id: &str,
    output: &TargetPath,
    roots: &[crate::targets::TargetRoot],
) -> Option<String> {
    match module.module_type {
        ModuleType::Instructions => Some("AGENTS.md".to_string()),
        ModuleType::Prompt | ModuleType::Command => output
            .path
            .file_name()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string()),
        ModuleType::Skill => {
            let best = crate::targets::best_root_for(roots, &output.target, &output.path)?;
            let rel = output.path.strip_prefix(&best.root).ok()?;
            let rel_str = rel.to_string_lossy().replace('\\', "/");
            let skill_name = module_name_from_id(module_id);
            let Some((first, rest)) = rel_str.split_once('/') else {
                return Some(rel_str);
            };
            if first == skill_name && !rest.is_empty() {
                Some(rest.to_string())
            } else {
                Some(rel_str)
            }
        }
    }
}

fn source_layer_for_module_file(
    engine: &Engine,
    module: &Module,
    module_rel_path: &str,
) -> anyhow::Result<String> {
    let rel = std::path::Path::new(module_rel_path);

    let global = overlay_dir_for_scope(engine, &module.id, OverlayScope::Global);
    let machine = overlay_dir_for_scope(engine, &module.id, OverlayScope::Machine);
    let project = overlay_dir_for_scope(engine, &module.id, OverlayScope::Project);

    if project.join(rel).exists() {
        return Ok("project".to_string());
    }
    if machine.join(rel).exists() {
        return Ok("machine".to_string());
    }
    if global.join(rel).exists() {
        return Ok("global".to_string());
    }

    let upstream = resolve_upstream_module_root(&engine.home, &engine.repo, module)?;
    if upstream.join(rel).exists() {
        return Ok("upstream".to_string());
    }

    Ok("missing".to_string())
}

fn explain_plan(cli: &Cli, engine: &Engine) -> anyhow::Result<()> {
    #[derive(serde::Serialize)]
    struct ExplainedModule {
        module_id: String,
        module_type: Option<String>,
        layer: Option<String>,
        module_path: Option<String>,
    }

    #[derive(serde::Serialize)]
    struct ExplainedChange {
        op: String,
        target: String,
        path: String,
        modules: Vec<ExplainedModule>,
    }

    let targets = selected_targets(&engine.manifest, &cli.target)?;
    let render = engine.desired_state(&cli.profile, &cli.target)?;
    let desired = render.desired;
    let mut warnings = render.warnings;
    let roots = render.roots;

    let manifest_index = load_manifest_module_ids(&roots)?;

    let managed_paths_from_manifest =
        crate::target_manifest::load_managed_paths_from_manifests(&roots)?;
    let managed_paths = if !managed_paths_from_manifest.is_empty() {
        Some(filter_managed(managed_paths_from_manifest, &cli.target))
    } else {
        latest_snapshot(&engine.home, &["deploy", "rollback"])?
            .as_ref()
            .map(load_managed_paths_from_snapshot)
            .transpose()?
            .map(|m| filter_managed(m, &cli.target))
    };
    let plan = compute_plan(&desired, managed_paths.as_ref())?;

    let mut explained = Vec::new();
    for c in &plan.changes {
        let tp = TargetPath {
            target: c.target.clone(),
            path: PathBuf::from(&c.path),
        };

        let module_ids = match c.op {
            crate::deploy::Op::Delete => manifest_index.get(&tp).cloned().unwrap_or_default(),
            crate::deploy::Op::Create | crate::deploy::Op::Update => desired
                .get(&tp)
                .map(|f| f.module_ids.clone())
                .unwrap_or_default(),
        };

        let mut modules = Vec::new();
        for module_id in module_ids {
            let module = engine.manifest.modules.iter().find(|m| m.id == module_id);
            let module_type = module.map(|m| format!("{:?}", m.module_type));
            let module_path =
                module.and_then(|m| module_rel_path_for_output(m, &module_id, &tp, &roots));
            let layer = match (module, module_path.as_deref()) {
                (Some(m), Some(rel)) => Some(source_layer_for_module_file(engine, m, rel)?),
                _ => None,
            };
            modules.push(ExplainedModule {
                module_id,
                module_type,
                layer,
                module_path,
            });
        }

        explained.push(ExplainedChange {
            op: format!("{:?}", c.op).to_lowercase(),
            target: c.target.clone(),
            path: c.path.clone(),
            modules,
        });
    }

    if cli.json {
        let mut envelope = JsonEnvelope::ok(
            "explain.plan",
            serde_json::json!({
                "profile": cli.profile,
                "targets": targets,
                "changes": explained,
            }),
        );
        envelope.warnings = warnings;
        print_json(&envelope)?;
    } else {
        for w in warnings.drain(..) {
            eprintln!("Warning: {w}");
        }
        println!("Explain plan (machine_id={}):", engine.machine_id);
        for c in explained {
            println!("- {} {} {}", c.op, c.target, c.path);
            for m in c.modules {
                println!(
                    "  - module={} type={} layer={} path={}",
                    m.module_id,
                    m.module_type.as_deref().unwrap_or("-"),
                    m.layer.as_deref().unwrap_or("-"),
                    m.module_path.as_deref().unwrap_or("-")
                );
            }
        }
    }

    Ok(())
}

fn explain_status(cli: &Cli, engine: &Engine) -> anyhow::Result<()> {
    #[derive(serde::Serialize)]
    struct ExplainedDrift {
        kind: String,
        target: String,
        path: String,
        expected: Option<String>,
        actual: Option<String>,
        modules: Vec<String>,
    }

    let targets = selected_targets(&engine.manifest, &cli.target)?;
    let render = engine.desired_state(&cli.profile, &cli.target)?;
    let desired = render.desired;
    let mut warnings = render.warnings;
    let roots = render.roots;

    let manifest_index = load_manifest_module_ids(&roots)?;

    let managed_paths_from_manifest =
        crate::target_manifest::load_managed_paths_from_manifests(&roots)?;
    let managed_paths_from_manifest = filter_managed(managed_paths_from_manifest, &cli.target);

    let mut drift = Vec::new();
    if managed_paths_from_manifest.is_empty() {
        warnings.push(
            "no target manifests found; drift may be inaccurate (run deploy --apply to write manifests)"
                .to_string(),
        );
        for (tp, desired_file) in &desired {
            let expected = format!("sha256:{}", sha256_hex(&desired_file.bytes));
            match std::fs::read(&tp.path) {
                Ok(actual_bytes) => {
                    let actual = format!("sha256:{}", sha256_hex(&actual_bytes));
                    if actual != expected {
                        drift.push(ExplainedDrift {
                            kind: "modified".to_string(),
                            target: tp.target.clone(),
                            path: tp.path.to_string_lossy().to_string(),
                            expected: Some(expected),
                            actual: Some(actual),
                            modules: desired_file.module_ids.clone(),
                        });
                    }
                }
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                    drift.push(ExplainedDrift {
                        kind: "missing".to_string(),
                        target: tp.target.clone(),
                        path: tp.path.to_string_lossy().to_string(),
                        expected: Some(expected),
                        actual: None,
                        modules: desired_file.module_ids.clone(),
                    })
                }
                Err(err) => return Err(err).context("read deployed file"),
            }
        }
    } else {
        for tp in &managed_paths_from_manifest {
            let expected = desired
                .get(tp)
                .map(|f| format!("sha256:{}", sha256_hex(&f.bytes)));
            match std::fs::read(&tp.path) {
                Ok(actual_bytes) => {
                    let actual = format!("sha256:{}", sha256_hex(&actual_bytes));
                    if let Some(exp) = &expected {
                        if &actual != exp {
                            drift.push(ExplainedDrift {
                                kind: "modified".to_string(),
                                target: tp.target.clone(),
                                path: tp.path.to_string_lossy().to_string(),
                                expected: Some(exp.clone()),
                                actual: Some(actual),
                                modules: manifest_index.get(tp).cloned().unwrap_or_default(),
                            });
                        }
                    } else {
                        drift.push(ExplainedDrift {
                            kind: "extra".to_string(),
                            target: tp.target.clone(),
                            path: tp.path.to_string_lossy().to_string(),
                            expected: None,
                            actual: Some(actual),
                            modules: manifest_index.get(tp).cloned().unwrap_or_default(),
                        });
                    }
                }
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                    if let Some(exp) = expected {
                        drift.push(ExplainedDrift {
                            kind: "missing".to_string(),
                            target: tp.target.clone(),
                            path: tp.path.to_string_lossy().to_string(),
                            expected: Some(exp),
                            actual: None,
                            modules: manifest_index.get(tp).cloned().unwrap_or_default(),
                        });
                    }
                }
                Err(err) => return Err(err).context("read deployed file"),
            }
        }
    }

    if cli.json {
        let mut envelope = JsonEnvelope::ok(
            "explain.status",
            serde_json::json!({
                "profile": cli.profile,
                "targets": targets,
                "drift": drift,
            }),
        );
        envelope.warnings = warnings;
        print_json(&envelope)?;
    } else {
        for w in warnings.drain(..) {
            eprintln!("Warning: {w}");
        }
        println!("Explain status (machine_id={}):", engine.machine_id);
        for d in drift {
            println!(
                "- {} {} {} modules={}",
                d.kind,
                d.target,
                d.path,
                if d.modules.is_empty() {
                    "-".to_string()
                } else {
                    d.modules.join(",")
                }
            );
        }
    }

    Ok(())
}

fn evolve_propose(
    cli: &Cli,
    engine: &Engine,
    module_filter: Option<&str>,
    scope: EvolveScope,
    branch_override: Option<&str>,
) -> anyhow::Result<()> {
    #[derive(serde::Serialize)]
    struct ProposalItem {
        module_id: String,
        target: String,
        path: String,
    }

    #[derive(serde::Serialize)]
    struct SkippedItem {
        target: String,
        path: String,
        reason: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        module_id: Option<String>,
        #[serde(skip_serializing_if = "Vec::is_empty", default)]
        module_ids: Vec<String>,
    }

    #[derive(Default, serde::Serialize)]
    struct ProposalSummary {
        drifted_proposeable: u64,
        drifted_skipped: u64,
        skipped_missing: u64,
        skipped_multi_module: u64,
        skipped_read_error: u64,
    }

    let render = engine.desired_state(&cli.profile, &cli.target)?;
    let desired = render.desired;
    let roots = render.roots;

    let mut summary = ProposalSummary::default();
    let mut candidates: Vec<(String, TargetPath, Vec<u8>)> = Vec::new();
    let mut skipped: Vec<SkippedItem> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    for (tp, desired_file) in &desired {
        if let Some(filter) = module_filter {
            if !desired_file.module_ids.iter().any(|id| id == filter) {
                continue;
            }
        }

        let actual = match std::fs::read(&tp.path) {
            Ok(bytes) => Some(bytes),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => None,
            Err(err) => {
                summary.skipped_read_error += 1;
                warnings.push(format!(
                    "evolve.propose: skipped {} {}: failed to read deployed file: {err}",
                    tp.target,
                    tp.path.display()
                ));
                continue;
            }
        };

        let is_drifted = match &actual {
            Some(bytes) => bytes != &desired_file.bytes,
            None => true,
        };
        if !is_drifted {
            continue;
        }

        if desired_file.module_ids.len() != 1 {
            summary.drifted_skipped += 1;
            summary.skipped_multi_module += 1;
            skipped.push(SkippedItem {
                target: tp.target.clone(),
                path: tp.path.to_string_lossy().to_string(),
                reason: "multi_module_output".to_string(),
                module_id: None,
                module_ids: desired_file.module_ids.clone(),
            });
            continue;
        }

        let module_id = desired_file.module_ids[0].clone();
        match actual {
            Some(actual) => {
                summary.drifted_proposeable += 1;
                candidates.push((module_id, tp.clone(), actual));
            }
            None => {
                summary.drifted_skipped += 1;
                summary.skipped_missing += 1;
                skipped.push(SkippedItem {
                    target: tp.target.clone(),
                    path: tp.path.to_string_lossy().to_string(),
                    reason: "missing".to_string(),
                    module_id: Some(module_id),
                    module_ids: Vec::new(),
                });
            }
        }
    }

    let mut items: Vec<ProposalItem> = candidates
        .iter()
        .map(|(module_id, tp, _)| ProposalItem {
            module_id: module_id.clone(),
            target: tp.target.clone(),
            path: tp.path.to_string_lossy().to_string(),
        })
        .collect();
    items.sort_by(|a, b| {
        (a.module_id.as_str(), a.path.as_str()).cmp(&(b.module_id.as_str(), b.path.as_str()))
    });

    skipped.sort_by(|a, b| {
        (a.reason.as_str(), a.target.as_str(), a.path.as_str()).cmp(&(
            b.reason.as_str(),
            b.target.as_str(),
            b.path.as_str(),
        ))
    });

    if items.is_empty() {
        let reason = if skipped.is_empty() {
            "no_drift"
        } else {
            "no_proposeable_drift"
        };

        if cli.json {
            let envelope = JsonEnvelope::ok(
                "evolve.propose",
                serde_json::json!({
                    "created": false,
                    "reason": reason,
                    "summary": summary,
                    "skipped": skipped,
                }),
            );
            let mut envelope = envelope;
            envelope.warnings = warnings;
            print_json(&envelope)?;
        } else {
            for w in warnings {
                eprintln!("Warning: {w}");
            }
            if reason == "no_drift" {
                println!("No drifted managed files to propose");
            } else {
                println!("No proposeable drifted files to propose");
                if !skipped.is_empty() {
                    println!("Skipped drift (not proposeable):");
                    for s in skipped {
                        let who = s
                            .module_id
                            .as_deref()
                            .map(|m| m.to_string())
                            .unwrap_or_else(|| {
                                if s.module_ids.is_empty() {
                                    "-".to_string()
                                } else {
                                    s.module_ids.join(",")
                                }
                            });
                        println!("- {} {} {} modules={who}", s.reason, s.target, s.path);
                    }
                }
            }
        }
        return Ok(());
    }

    if cli.dry_run {
        if cli.json {
            let envelope = JsonEnvelope::ok(
                "evolve.propose",
                serde_json::json!({
                    "created": false,
                    "reason": "dry_run",
                    "candidates": items,
                    "skipped": skipped,
                    "summary": summary,
                }),
            );
            let mut envelope = envelope;
            envelope.warnings = warnings;
            print_json(&envelope)?;
        } else {
            for w in warnings {
                eprintln!("Warning: {w}");
            }
            println!("Candidates (dry-run):");
            for i in items {
                println!("- {} {} {}", i.module_id, i.target, i.path);
            }
            if !skipped.is_empty() {
                println!("Skipped drift (not proposeable):");
                for s in skipped {
                    let who = s
                        .module_id
                        .as_deref()
                        .map(|m| m.to_string())
                        .unwrap_or_else(|| {
                            if s.module_ids.is_empty() {
                                "-".to_string()
                            } else {
                                s.module_ids.join(",")
                            }
                        });
                    println!("- {} {} {} modules={who}", s.reason, s.target, s.path);
                }
            }
        }
        return Ok(());
    }

    if cli.json && !cli.yes {
        return Err(UserError::confirm_required("evolve propose"));
    }
    if !cli.json && !cli.yes && !confirm("Create evolve proposal branch?")? {
        println!("Aborted");
        return Ok(());
    }

    let repo_dir = engine.repo.repo_dir.as_path();
    if !repo_dir.join(".git").exists() {
        anyhow::bail!(
            "config repo is not a git repository: {}",
            repo_dir.display()
        );
    }

    let status = crate::git::git_in(repo_dir, &["status", "--porcelain"])?;
    if !status.trim().is_empty() {
        anyhow::bail!("refusing to propose with a dirty working tree (commit or stash first)");
    }

    let original = crate::git::git_in(repo_dir, &["rev-parse", "--abbrev-ref", "HEAD"])?;
    let original = original.trim().to_string();

    let branch = branch_override.map(|s| s.to_string()).unwrap_or_else(|| {
        let nanos = time::OffsetDateTime::now_utc().unix_timestamp_nanos();
        format!("evolve/propose-{nanos}")
    });

    crate::git::git_in(repo_dir, &["checkout", "-b", branch.as_str()])?;

    let mut touched = Vec::new();
    for (module_id, output, actual) in &candidates {
        let Some(module) = engine.manifest.modules.iter().find(|m| m.id == *module_id) else {
            continue;
        };
        let Some(module_rel) = module_rel_path_for_output(module, module_id, output, &roots) else {
            continue;
        };

        let overlay_dir = match scope {
            EvolveScope::Global => overlay_dir_for_scope(engine, module_id, OverlayScope::Global),
            EvolveScope::Machine => overlay_dir_for_scope(engine, module_id, OverlayScope::Machine),
            EvolveScope::Project => overlay_dir_for_scope(engine, module_id, OverlayScope::Project),
        };

        let dst = overlay_dir.join(&module_rel);
        if let Some(parent) = dst.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("create {}", parent.display()))?;
        }
        write_atomic(&dst, actual).with_context(|| format!("write {}", dst.display()))?;
        touched.push(
            dst.strip_prefix(&engine.repo.repo_dir)
                .unwrap_or(&dst)
                .to_string_lossy()
                .to_string(),
        );
    }

    if touched.is_empty() {
        crate::git::git_in(repo_dir, &["checkout", original.as_str()]).ok();
        anyhow::bail!("no proposeable files (only multi-module outputs or unknown modules)");
    }

    crate::git::git_in(repo_dir, &["add", "-A"])?;

    let commit = std::process::Command::new("git")
        .current_dir(repo_dir)
        .args(["commit", "-m", "chore(evolve): propose overlay updates"])
        .output();

    let committed = match commit {
        Ok(out) if out.status.success() => true,
        Ok(out) => {
            eprintln!(
                "Warning: git commit failed: {}",
                String::from_utf8_lossy(&out.stderr)
            );
            false
        }
        Err(err) => {
            eprintln!("Warning: failed to run git commit: {err}");
            false
        }
    };

    if committed {
        crate::git::git_in(repo_dir, &["checkout", original.as_str()]).ok();
    }

    if cli.json {
        let envelope = JsonEnvelope::ok(
            "evolve.propose",
            serde_json::json!({
                "created": true,
                "branch": branch,
                "scope": scope,
                "files": touched,
                "committed": committed,
            }),
        );
        print_json(&envelope)?;
    } else {
        println!("Created proposal branch: {branch}");
        for f in &touched {
            println!("- {f}");
        }
        if !committed {
            println!("Note: commit failed; changes are left on the proposal branch.");
        }
    }

    Ok(())
}

fn selected_targets(manifest: &Manifest, target_filter: &str) -> anyhow::Result<Vec<String>> {
    let mut known: Vec<String> = manifest.targets.keys().cloned().collect();
    known.sort();

    match target_filter {
        "all" => Ok(known),
        "codex" | "claude_code" => {
            if !manifest.targets.contains_key(target_filter) {
                return Err(anyhow::Error::new(
                    UserError::new(
                        "E_CONFIG_INVALID",
                        format!("target not configured: {target_filter}"),
                    )
                    .with_details(serde_json::json!({
                        "target": target_filter,
                        "hint": "add the target under `targets:` in agentpack.yaml",
                    })),
                ));
            }
            Ok(vec![target_filter.to_string()])
        }
        other => Err(anyhow::Error::new(
            UserError::new(
                "E_TARGET_UNSUPPORTED",
                format!("unsupported --target: {other}"),
            )
            .with_details(serde_json::json!({
                "target": other,
                "allowed": ["all","codex","claude_code"],
            })),
        )),
    }
}

fn bootstrap_scope_flags(scope: BootstrapScope) -> (bool, bool) {
    match scope {
        BootstrapScope::User => (true, false),
        BootstrapScope::Project => (false, true),
        BootstrapScope::Both => (true, true),
    }
}

fn bootstrap_scope_str(scope: BootstrapScope) -> &'static str {
    match scope {
        BootstrapScope::User => "user",
        BootstrapScope::Project => "project",
        BootstrapScope::Both => "both",
    }
}

fn codex_home_for_manifest(manifest: &Manifest) -> anyhow::Result<PathBuf> {
    if let Some(cfg) = manifest.targets.get("codex") {
        if let Some(serde_yaml::Value::String(s)) = cfg.options.get("codex_home") {
            if !s.trim().is_empty() {
                return expand_tilde(s);
            }
        }
    }

    if let Ok(env) = std::env::var("CODEX_HOME") {
        if !env.trim().is_empty() {
            return expand_tilde(&env);
        }
    }

    expand_tilde("~/.codex")
}

fn expand_tilde(s: &str) -> anyhow::Result<PathBuf> {
    if let Some(rest) = s.strip_prefix("~/") {
        let home = dirs::home_dir().context("resolve home dir")?;
        return Ok(home.join(rest));
    }
    Ok(PathBuf::from(s))
}

fn overlay_dir_for_scope(engine: &Engine, module_id: &str, scope: OverlayScope) -> PathBuf {
    let fs_key = crate::ids::module_fs_key(module_id);
    let canonical = match scope {
        OverlayScope::Global => engine.repo.repo_dir.join("overlays").join(&fs_key),
        OverlayScope::Machine => engine
            .repo
            .repo_dir
            .join("overlays/machines")
            .join(&engine.machine_id)
            .join(&fs_key),
        OverlayScope::Project => engine
            .repo
            .repo_dir
            .join("projects")
            .join(&engine.project.project_id)
            .join("overlays")
            .join(&fs_key),
    };

    let legacy_fs_key = crate::ids::module_fs_key_unbounded(module_id);
    let legacy_fs_key = (legacy_fs_key != fs_key).then(|| match scope {
        OverlayScope::Global => engine.repo.repo_dir.join("overlays").join(&legacy_fs_key),
        OverlayScope::Machine => engine
            .repo
            .repo_dir
            .join("overlays/machines")
            .join(&engine.machine_id)
            .join(&legacy_fs_key),
        OverlayScope::Project => engine
            .repo
            .repo_dir
            .join("projects")
            .join(&engine.project.project_id)
            .join("overlays")
            .join(&legacy_fs_key),
    });

    let legacy = crate::ids::is_safe_legacy_path_component(module_id).then(|| match scope {
        OverlayScope::Global => engine.repo.repo_dir.join("overlays").join(module_id),
        OverlayScope::Machine => engine
            .repo
            .repo_dir
            .join("overlays/machines")
            .join(&engine.machine_id)
            .join(module_id),
        OverlayScope::Project => engine
            .repo
            .repo_dir
            .join("projects")
            .join(&engine.project.project_id)
            .join("overlays")
            .join(module_id),
    });

    if canonical.exists() {
        canonical
    } else if legacy_fs_key.as_ref().is_some_and(|p| p.exists()) {
        legacy_fs_key.expect("legacy fs_key exists")
    } else if legacy.as_ref().is_some_and(|p| p.exists()) {
        legacy.expect("legacy exists")
    } else {
        canonical
    }
}

fn target_scope_flags(scope: &TargetScope) -> (bool, bool) {
    match scope {
        TargetScope::User => (true, false),
        TargetScope::Project => (false, true),
        TargetScope::Both => (true, true),
    }
}

fn extract_agentpack_version(text: &str) -> Option<String> {
    for line in text.lines() {
        if let Some((_, rest)) = line.split_once("agentpack_version:") {
            let mut value = rest.trim();
            value = value.trim_end_matches("-->");
            value = value.trim();
            value = value.trim_matches(|c| c == '"' || c == '\'');
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }
    None
}

fn warn_operator_assets_if_outdated(
    engine: &Engine,
    targets: &[String],
    warnings: &mut Vec<String>,
) -> anyhow::Result<()> {
    let current = env!("CARGO_PKG_VERSION");

    for target in targets {
        match target.as_str() {
            "codex" => {
                let Some(cfg) = engine.manifest.targets.get("codex") else {
                    continue;
                };
                let (allow_user, allow_project) = target_scope_flags(&cfg.scope);
                let codex_home = codex_home_for_manifest(&engine.manifest)?;

                if allow_user {
                    let path = codex_home.join("skills/agentpack-operator/SKILL.md");
                    check_operator_file(
                        &path,
                        "codex/user",
                        current,
                        warnings,
                        "agentpack bootstrap --target codex --scope user",
                    )?;
                }
                if allow_project {
                    let path = engine
                        .project
                        .project_root
                        .join(".codex/skills/agentpack-operator/SKILL.md");
                    check_operator_file(
                        &path,
                        "codex/project",
                        current,
                        warnings,
                        "agentpack bootstrap --target codex --scope project",
                    )?;
                }
            }
            "claude_code" => {
                let Some(cfg) = engine.manifest.targets.get("claude_code") else {
                    continue;
                };
                let (allow_user, allow_project) = target_scope_flags(&cfg.scope);

                if allow_user {
                    let dir = expand_tilde("~/.claude/commands")?;
                    check_operator_command_dir(
                        &dir,
                        "claude_code/user",
                        current,
                        warnings,
                        "agentpack bootstrap --target claude_code --scope user",
                    )?;
                }
                if allow_project {
                    let dir = engine.project.project_root.join(".claude/commands");
                    check_operator_command_dir(
                        &dir,
                        "claude_code/project",
                        current,
                        warnings,
                        "agentpack bootstrap --target claude_code --scope project",
                    )?;
                }
            }
            _ => {}
        }
    }

    Ok(())
}

fn check_operator_file(
    path: &std::path::Path,
    location: &str,
    current: &str,
    warnings: &mut Vec<String>,
    suggested: &str,
) -> anyhow::Result<()> {
    if !path.exists() {
        warnings.push(format!(
            "operator assets missing ({location}): {}; run: {suggested}",
            path.display()
        ));
        return Ok(());
    }

    let text = std::fs::read_to_string(path)
        .with_context(|| format!("read operator asset {}", path.display()))?;
    let Some(have) = extract_agentpack_version(&text) else {
        warnings.push(format!(
            "operator assets missing agentpack_version ({location}): {}; run: {suggested}",
            path.display()
        ));
        return Ok(());
    };

    if have != current {
        warnings.push(format!(
            "operator assets outdated ({location}): {} has {}, want {}; run: {suggested}",
            path.display(),
            have,
            current
        ));
    }

    Ok(())
}

fn check_operator_command_dir(
    dir: &std::path::Path,
    location: &str,
    current: &str,
    warnings: &mut Vec<String>,
    suggested: &str,
) -> anyhow::Result<()> {
    const EXPECTED: &[&str] = &[
        "ap-doctor.md",
        "ap-update.md",
        "ap-preview.md",
        "ap-plan.md",
        "ap-diff.md",
        "ap-deploy.md",
        "ap-status.md",
        "ap-explain.md",
        "ap-evolve.md",
    ];

    let mut present = 0usize;
    let mut missing = Vec::new();
    let mut missing_version: Option<std::path::PathBuf> = None;
    let mut outdated: Option<(std::path::PathBuf, String)> = None;

    for name in EXPECTED {
        let path = dir.join(name);
        if !path.exists() {
            missing.push(*name);
            continue;
        }
        present += 1;

        let text = std::fs::read_to_string(&path)
            .with_context(|| format!("read operator asset {}", path.display()))?;
        let Some(have) = extract_agentpack_version(&text) else {
            missing_version.get_or_insert(path);
            continue;
        };
        if have != current {
            outdated.get_or_insert((path, have));
        }
    }

    if present == 0 {
        warnings.push(format!(
            "operator assets missing ({location}): {}; run: {suggested}",
            dir.display()
        ));
        return Ok(());
    }

    if let Some((path, have)) = outdated {
        warnings.push(format!(
            "operator assets outdated ({location}): {} has {}, want {}; run: {suggested}",
            path.display(),
            have,
            current
        ));
        return Ok(());
    }

    if let Some(path) = missing_version {
        warnings.push(format!(
            "operator assets missing agentpack_version ({location}): {}; run: {suggested}",
            path.display()
        ));
        return Ok(());
    }

    if !missing.is_empty() {
        warnings.push(format!(
            "operator assets incomplete ({location}): missing {}; run: {suggested}",
            missing.join(", "),
        ));
    }

    Ok(())
}

fn print_diff(
    plan: &crate::deploy::PlanResult,
    desired: &crate::deploy::DesiredState,
) -> anyhow::Result<()> {
    for c in &plan.changes {
        let path = std::path::PathBuf::from(&c.path);
        let desired_key = TargetPath {
            target: c.target.clone(),
            path: path.clone(),
        };

        let before_text = if matches!(c.op, crate::deploy::Op::Create) {
            Some(String::new())
        } else {
            crate::deploy::read_text(&path)?
        };
        let after_text = if matches!(c.op, crate::deploy::Op::Delete) {
            Some(String::new())
        } else {
            desired
                .get(&desired_key)
                .and_then(|f| String::from_utf8(f.bytes.clone()).ok())
        };

        println!("\n=== {} {} ===", c.target, c.path);
        match (before_text, after_text) {
            (Some(from), Some(to)) => {
                print!(
                    "{}",
                    unified_diff(
                        &from,
                        &to,
                        &format!("before: {}", c.path),
                        &format!("after: {}", c.path)
                    )
                );
            }
            _ => {
                println!("(binary or non-utf8 content; diff omitted)");
            }
        }
    }

    Ok(())
}

fn dir_is_writable(dir: &std::path::Path) -> bool {
    if !dir.is_dir() {
        return false;
    }

    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .map(|d| d.as_nanos())
        .unwrap_or(0);

    let test_path = dir.join(format!(".agentpack-write-test-{nanos}"));
    let created = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&test_path)
        .and_then(|mut f| std::io::Write::write_all(&mut f, b"ok\n"))
        .is_ok();

    if created {
        let _ = std::fs::remove_file(&test_path);
    }

    created
}

fn confirm(prompt: &str) -> anyhow::Result<bool> {
    use std::io::Write as _;

    print!("{prompt} [y/N] ");
    std::io::stdout().flush().ok();
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    let s = input.trim().to_lowercase();
    Ok(s == "y" || s == "yes")
}

impl Cli {
    fn command_name(&self) -> String {
        match &self.command {
            Commands::Init => "init",
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
            Commands::Status => "status",
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
