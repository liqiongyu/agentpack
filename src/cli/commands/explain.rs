use std::path::PathBuf;

use anyhow::Context as _;

use crate::deploy::TargetPath;
use crate::deploy::load_managed_paths_from_snapshot;
use crate::deploy::plan as compute_plan;
use crate::engine::Engine;
use crate::hash::sha256_hex;
use crate::output::{JsonEnvelope, print_json};
use crate::state::latest_snapshot;

use super::super::args::ExplainCommands;
use super::Ctx;

pub(crate) fn run(ctx: &Ctx<'_>, command: &ExplainCommands) -> anyhow::Result<()> {
    let engine = Engine::load(ctx.cli.repo.as_deref(), ctx.cli.machine.as_deref())?;
    match command {
        ExplainCommands::Plan => explain_plan(ctx.cli, &engine),
        ExplainCommands::Diff => explain_plan(ctx.cli, &engine),
        ExplainCommands::Status => explain_status(ctx.cli, &engine),
    }
}

fn explain_plan(cli: &super::super::args::Cli, engine: &Engine) -> anyhow::Result<()> {
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

    let targets = super::super::util::selected_targets(&engine.manifest, &cli.target)?;
    let render = engine.desired_state(&cli.profile, &cli.target)?;
    let desired = render.desired;
    let mut warnings = render.warnings;
    let roots = render.roots;

    let manifest_index = super::super::util::load_manifest_module_ids(&roots)?;

    let managed_paths_from_manifest =
        crate::target_manifest::load_managed_paths_from_manifests(&roots)?;
    let managed_paths = if !managed_paths_from_manifest.is_empty() {
        Some(super::super::util::filter_managed(
            managed_paths_from_manifest,
            &cli.target,
        ))
    } else {
        latest_snapshot(&engine.home, &["deploy", "rollback"])?
            .as_ref()
            .map(load_managed_paths_from_snapshot)
            .transpose()?
            .map(|m| super::super::util::filter_managed(m, &cli.target))
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
            let module_path = module.and_then(|m| {
                super::super::util::module_rel_path_for_output(m, &module_id, &tp, &roots)
            });
            let layer = match (module, module_path.as_deref()) {
                (Some(m), Some(rel)) => Some(super::super::util::source_layer_for_module_file(
                    engine, m, rel,
                )?),
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

fn explain_status(cli: &super::super::args::Cli, engine: &Engine) -> anyhow::Result<()> {
    #[derive(serde::Serialize)]
    struct ExplainedDrift {
        kind: String,
        target: String,
        path: String,
        expected: Option<String>,
        actual: Option<String>,
        modules: Vec<String>,
    }

    let targets = super::super::util::selected_targets(&engine.manifest, &cli.target)?;
    let render = engine.desired_state(&cli.profile, &cli.target)?;
    let desired = render.desired;
    let mut warnings = render.warnings;
    let roots = render.roots;

    let manifest_index = super::super::util::load_manifest_module_ids(&roots)?;

    let managed_paths_from_manifest =
        crate::target_manifest::load_managed_paths_from_manifests(&roots)?;
    let managed_paths_from_manifest =
        super::super::util::filter_managed(managed_paths_from_manifest, &cli.target);

    let mut drift = Vec::new();
    if managed_paths_from_manifest.is_empty() {
        warnings.push(
            "no target manifests found; drift may be inaccurate (run deploy --apply to write manifests)".to_string(),
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
