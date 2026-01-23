use anyhow::Context as _;

pub(super) async fn call_explain_in_process(
    args: super::ExplainArgs,
) -> anyhow::Result<(String, serde_json::Value)> {
    tokio::task::spawn_blocking(move || {
        use crate::app::explain_json::{
            ExplainedChange, ExplainedDrift, ExplainedModule, explain_plan_json_data,
            explain_status_json_data,
        };

        let repo_override = args.common.repo.as_ref().map(std::path::PathBuf::from);
        let profile = args.common.profile.as_deref().unwrap_or("default");
        let target = args.common.target.as_deref().unwrap_or("all");
        let machine_override = args.common.machine.as_deref();

        let (command, command_id, command_path) = match args.kind {
            super::ExplainKindArg::Plan => ("explain.plan", "explain plan", ["explain", "plan"]),
            super::ExplainKindArg::Diff => ("explain.plan", "explain diff", ["explain", "diff"]),
            super::ExplainKindArg::Status => {
                ("explain.status", "explain status", ["explain", "status"])
            }
        };
        let meta = super::CommandMeta {
            command,
            command_id,
            command_path: &command_path,
        };

        let result = (|| -> anyhow::Result<(String, serde_json::Value)> {
            let engine = crate::engine::Engine::load(repo_override.as_deref(), machine_override)?;

            match args.kind {
                super::ExplainKindArg::Plan | super::ExplainKindArg::Diff => {
                    let targets = crate::cli::util::selected_targets(&engine.manifest, target)?;
                    let render = engine.desired_state(profile, target)?;
                    let desired = render.desired;
                    let mut warnings = render.warnings;
                    let roots = render.roots;

                    let manifest_index = crate::cli::util::load_manifest_module_ids(&roots)?;
                    warnings.extend(manifest_index.warnings);
                    let manifest_index = manifest_index.index;

                    let managed_paths_from_manifest =
                        crate::target_manifest::load_managed_paths_from_manifests(&roots)?;
                    warnings.extend(managed_paths_from_manifest.warnings);
                    let managed_paths_from_manifest = managed_paths_from_manifest.managed_paths;
                    let managed_paths = if !managed_paths_from_manifest.is_empty() {
                        Some(crate::cli::util::filter_managed(
                            managed_paths_from_manifest,
                            target,
                        ))
                    } else {
                        crate::state::latest_snapshot(&engine.home, &["deploy", "rollback"])?
                            .as_ref()
                            .map(crate::deploy::load_managed_paths_from_snapshot)
                            .transpose()?
                            .map(|m| crate::cli::util::filter_managed(m, target))
                    };
                    let plan = crate::deploy::plan(&desired, managed_paths.as_ref())?;

                    let mut explained = Vec::new();
                    for c in &plan.changes {
                        let tp = crate::deploy::TargetPath {
                            target: c.target.clone(),
                            path: std::path::PathBuf::from(&c.path),
                        };

                        let module_ids = match c.op {
                            crate::deploy::Op::Delete => {
                                manifest_index.get(&tp).cloned().unwrap_or_default()
                            }
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
                                crate::cli::util::module_rel_path_for_output(
                                    m, &module_id, &tp, &roots,
                                )
                            });
                            let layer = match (module, module_path.as_deref()) {
                                (Some(m), Some(rel)) => Some(
                                    crate::cli::util::source_layer_for_module_file(
                                        &engine, m, rel,
                                    )?,
                                ),
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
                            path_posix: c.path_posix.clone(),
                            modules,
                        });
                    }

                    let data = explain_plan_json_data(profile, targets, explained);
                    let mut envelope = crate::output::JsonEnvelope::ok(meta.command, data)
                        .with_command_meta(meta.command_id_string(), meta.command_path_vec());
                    envelope.warnings = warnings;
                    let text = serde_json::to_string_pretty(&envelope)?;
                    let envelope = serde_json::to_value(&envelope)?;
                    Ok((text, envelope))
                }
                super::ExplainKindArg::Status => {
                    let targets = crate::cli::util::selected_targets(&engine.manifest, target)?;
                    let render = engine.desired_state(profile, target)?;
                    let desired = render.desired;
                    let mut warnings = render.warnings;
                    let roots = render.roots;

                    let manifest_index = crate::cli::util::load_manifest_module_ids(&roots)?;
                    warnings.extend(manifest_index.warnings);
                    let manifest_index = manifest_index.index;

                    let managed_paths_from_manifest =
                        crate::target_manifest::load_managed_paths_from_manifests(&roots)?;
                    warnings.extend(managed_paths_from_manifest.warnings);
                    let managed_paths_from_manifest = crate::cli::util::filter_managed(
                        managed_paths_from_manifest.managed_paths,
                        target,
                    );

                    let mut drift = Vec::new();
                    if managed_paths_from_manifest.is_empty() {
                        warnings.push(
                            "no target manifests found; drift may be inaccurate (run deploy --apply to write manifests)".to_string(),
                        );
                        for (tp, desired_file) in &desired {
                            let expected = format!(
                                "sha256:{}",
                                crate::hash::sha256_hex(&desired_file.bytes)
                            );
                            match std::fs::read(&tp.path) {
                                Ok(actual_bytes) => {
                                    let actual = format!(
                                        "sha256:{}",
                                        crate::hash::sha256_hex(&actual_bytes)
                                    );
                                    if actual != expected {
                                        drift.push(ExplainedDrift {
                                            kind: "modified".to_string(),
                                            target: tp.target.clone(),
                                            path: tp.path.to_string_lossy().to_string(),
                                            path_posix: crate::paths::path_to_posix_string(&tp.path),
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
                                        path_posix: crate::paths::path_to_posix_string(&tp.path),
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
                                .map(|f| format!("sha256:{}", crate::hash::sha256_hex(&f.bytes)));
                            match std::fs::read(&tp.path) {
                                Ok(actual_bytes) => {
                                    let actual = format!(
                                        "sha256:{}",
                                        crate::hash::sha256_hex(&actual_bytes)
                                    );
                                    if let Some(exp) = &expected {
                                        if &actual != exp {
                                            drift.push(ExplainedDrift {
                                                kind: "modified".to_string(),
                                                target: tp.target.clone(),
                                                path: tp.path.to_string_lossy().to_string(),
                                                path_posix: crate::paths::path_to_posix_string(
                                                    &tp.path,
                                                ),
                                                expected: Some(exp.clone()),
                                                actual: Some(actual),
                                                modules: manifest_index
                                                    .get(tp)
                                                    .cloned()
                                                    .unwrap_or_default(),
                                            });
                                        }
                                    } else {
                                        drift.push(ExplainedDrift {
                                            kind: "extra".to_string(),
                                            target: tp.target.clone(),
                                            path: tp.path.to_string_lossy().to_string(),
                                            path_posix: crate::paths::path_to_posix_string(&tp.path),
                                            expected: None,
                                            actual: Some(actual),
                                            modules: manifest_index
                                                .get(tp)
                                                .cloned()
                                                .unwrap_or_default(),
                                        });
                                    }
                                }
                                Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                                    if let Some(exp) = expected {
                                        drift.push(ExplainedDrift {
                                            kind: "missing".to_string(),
                                            target: tp.target.clone(),
                                            path: tp.path.to_string_lossy().to_string(),
                                            path_posix: crate::paths::path_to_posix_string(
                                                &tp.path,
                                            ),
                                            expected: Some(exp),
                                            actual: None,
                                            modules: manifest_index
                                                .get(tp)
                                                .cloned()
                                                .unwrap_or_default(),
                                        });
                                    }
                                }
                                Err(err) => return Err(err).context("read deployed file"),
                            }
                        }
                    }

                    let data = explain_status_json_data(profile, targets, drift);
                    let mut envelope = crate::output::JsonEnvelope::ok(meta.command, data)
                        .with_command_meta(meta.command_id_string(), meta.command_path_vec());
                    envelope.warnings = warnings;
                    let text = serde_json::to_string_pretty(&envelope)?;
                    let envelope = serde_json::to_value(&envelope)?;
                    Ok((text, envelope))
                }
            }
        })();

        match result {
            Ok(v) => Ok(v),
            Err(err) => {
                let envelope = super::envelope_from_anyhow_error(meta, &err);
                let text = serde_json::to_string_pretty(&envelope)?;
                Ok((text, envelope))
            }
        }
    })
    .await
    .context("mcp explain handler task join")?
}
