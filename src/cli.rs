use std::path::PathBuf;

use anyhow::Context as _;
use clap::{CommandFactory, Parser, Subcommand, ValueEnum};

use crate::config::{Manifest, Module, ModuleType};
use crate::deploy::{TargetPath, load_managed_paths_from_snapshot, plan as compute_plan};
use crate::diff::unified_diff;
use crate::engine::Engine;
use crate::hash::sha256_hex;
use crate::lockfile::{Lockfile, generate_lockfile, hash_tree};
use crate::output::{JsonEnvelope, JsonError, print_json};
use crate::overlay::ensure_overlay_skeleton;
use crate::paths::{AgentpackHome, RepoPaths};
use crate::project::ProjectContext;
use crate::source::parse_source_spec;
use crate::state::latest_snapshot;
use crate::store::Store;

const TEMPLATE_CODEX_OPERATOR_SKILL: &str =
    include_str!("../templates/codex/skills/agentpack-operator/SKILL.md");
const TEMPLATE_CLAUDE_AP_PLAN: &str = include_str!("../templates/claude/commands/ap-plan.md");
const TEMPLATE_CLAUDE_AP_DEPLOY: &str = include_str!("../templates/claude/commands/ap-deploy.md");
const TEMPLATE_CLAUDE_AP_STATUS: &str = include_str!("../templates/claude/commands/ap-status.md");
const TEMPLATE_CLAUDE_AP_DIFF: &str = include_str!("../templates/claude/commands/ap-diff.md");

#[derive(Parser, Debug)]
#[command(name = "agentpack")]
#[command(about = "AI-first local asset control plane", long_about = None)]
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

    /// Check local environment and target paths
    Doctor,

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

            if cli.json && !cli.yes {
                anyhow::bail!("refusing to --apply in --json mode without --yes");
            }

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
        Commands::Doctor => {
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

            let engine = Engine::load(cli.repo.as_deref(), cli.machine.as_deref())?;
            let render = engine.desired_state(&cli.profile, &cli.target)?;
            let mut warnings = render.warnings;

            let mut checks = Vec::new();
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

            if cli.json {
                let mut envelope = JsonEnvelope::ok(
                    "doctor",
                    serde_json::json!({
                        "machine_id": engine.machine_id,
                        "roots": checks,
                    }),
                );
                envelope.warnings = warnings;
                print_json(&envelope)?;
            } else {
                for w in warnings.drain(..) {
                    eprintln!("Warning: {w}");
                }
                println!("Machine ID: {}", engine.machine_id);
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

            if targets.iter().any(|t| t == "codex") {
                let codex_home = codex_home_for_manifest(&engine.manifest)?;
                let bytes = TEMPLATE_CODEX_OPERATOR_SKILL.as_bytes().to_vec();

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
                }
            }

            if targets.iter().any(|t| t == "claude_code") {
                let bytes_plan = TEMPLATE_CLAUDE_AP_PLAN.as_bytes().to_vec();
                let bytes_deploy = TEMPLATE_CLAUDE_AP_DEPLOY.as_bytes().to_vec();
                let bytes_status = TEMPLATE_CLAUDE_AP_STATUS.as_bytes().to_vec();
                let bytes_diff = TEMPLATE_CLAUDE_AP_DIFF.as_bytes().to_vec();

                if allow_user {
                    let user_dir = expand_tilde("~/.claude/commands")?;
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
                }

                if allow_project {
                    let repo_dir = engine.project.project_root.join(".claude/commands");
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

            if cli.json && !cli.yes {
                anyhow::bail!("refusing to bootstrap in --json mode without --yes");
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

            if !cli.yes && !cli.json && !confirm("Apply bootstrap changes?")? {
                println!("Aborted");
                return Ok(());
            }

            let snapshot =
                crate::apply::apply_plan(&engine.home, "bootstrap", &plan, &desired, None, &[])?;
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

fn selected_targets(manifest: &Manifest, target_filter: &str) -> anyhow::Result<Vec<String>> {
    let mut known: Vec<String> = manifest.targets.keys().cloned().collect();
    known.sort();

    match target_filter {
        "all" => Ok(known),
        "codex" | "claude_code" => {
            if !manifest.targets.contains_key(target_filter) {
                anyhow::bail!("target not configured: {target_filter}");
            }
            Ok(vec![target_filter.to_string()])
        }
        other => anyhow::bail!("unsupported --target: {other}"),
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
            Commands::Add { .. } => "add",
            Commands::Remove { .. } => "remove",
            Commands::Lock => "lock",
            Commands::Fetch => "fetch",
            Commands::Plan => "plan",
            Commands::Diff => "diff",
            Commands::Deploy { .. } => "deploy",
            Commands::Status => "status",
            Commands::Doctor => "doctor",
            Commands::Remote { .. } => "remote",
            Commands::Sync { .. } => "sync",
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
