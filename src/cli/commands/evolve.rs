use anyhow::Context as _;

use crate::deploy::TargetPath;
use crate::engine::Engine;
use crate::fs::write_atomic;
use crate::output::{JsonEnvelope, print_json};
use crate::user_error::UserError;

use super::super::args::{EvolveCommands, EvolveScope, OverlayScope};
use super::Ctx;

pub(crate) fn run(ctx: &Ctx<'_>, command: &EvolveCommands) -> anyhow::Result<()> {
    let engine = Engine::load(ctx.cli.repo.as_deref(), ctx.cli.machine.as_deref())?;
    match command {
        EvolveCommands::Propose {
            module_id,
            scope,
            branch,
        } => evolve_propose(
            ctx.cli,
            &engine,
            module_id.as_deref(),
            *scope,
            branch.as_deref(),
        ),
        EvolveCommands::Restore { module_id } => {
            evolve_restore(ctx.cli, &engine, module_id.as_deref())
        }
    }
}

fn evolve_propose(
    cli: &super::super::args::Cli,
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
        path_posix: String,
    }

    #[derive(serde::Serialize)]
    struct Suggestion {
        action: String,
        reason: String,
    }

    #[derive(serde::Serialize)]
    struct SkippedItem {
        target: String,
        path: String,
        path_posix: String,
        reason: String,
        reason_code: String,
        reason_message: String,
        next_actions: Vec<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        module_id: Option<String>,
        #[serde(skip_serializing_if = "Vec::is_empty", default)]
        module_ids: Vec<String>,
        #[serde(skip_serializing_if = "Vec::is_empty", default)]
        suggestions: Vec<Suggestion>,
    }

    #[derive(Default, serde::Serialize)]
    struct ProposalSummary {
        drifted_proposeable: u64,
        drifted_skipped: u64,
        skipped_missing: u64,
        skipped_multi_module: u64,
        skipped_read_error: u64,
    }

    type MarkedSectionCandidates = Vec<(String, Vec<u8>)>;

    fn try_propose_marked_instructions_sections(
        desired_bytes: &[u8],
        actual_bytes: &[u8],
        module_ids: &[String],
    ) -> anyhow::Result<Option<MarkedSectionCandidates>> {
        if !desired_bytes
            .windows(crate::markers::MODULE_SECTION_START_PREFIX.len())
            .any(|w| w == crate::markers::MODULE_SECTION_START_PREFIX.as_bytes())
        {
            return Ok(None);
        }
        if !actual_bytes
            .windows(crate::markers::MODULE_SECTION_START_PREFIX.len())
            .any(|w| w == crate::markers::MODULE_SECTION_START_PREFIX.as_bytes())
        {
            return Ok(None);
        }

        let desired = match crate::markers::parse_module_sections_from_bytes(desired_bytes) {
            Ok(v) => v,
            Err(_) => return Ok(None),
        };
        let actual = match crate::markers::parse_module_sections_from_bytes(actual_bytes) {
            Ok(v) => v,
            Err(_) => return Ok(None),
        };

        if module_ids
            .iter()
            .any(|id| !desired.contains_key(id) || !actual.contains_key(id))
        {
            return Ok(None);
        }

        let mut out = Vec::new();
        for module_id in module_ids {
            let Some(desired_text) = desired.get(module_id) else {
                continue;
            };
            let Some(actual_text) = actual.get(module_id) else {
                continue;
            };
            if desired_text != actual_text {
                out.push((module_id.clone(), actual_text.as_bytes().to_vec()));
            }
        }

        if out.is_empty() {
            Ok(None)
        } else {
            Ok(Some(out))
        }
    }

    let render = engine.desired_state(&cli.profile, &cli.target)?;
    let desired = render.desired;
    let roots = render.roots;

    let mut summary = ProposalSummary::default();
    let mut candidates: Vec<(String, TargetPath, Vec<u8>)> = Vec::new();
    let mut instructions_sections: std::collections::BTreeMap<String, Vec<u8>> =
        std::collections::BTreeMap::new();
    let mut skipped: Vec<SkippedItem> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    let prefix = action_prefix(cli);
    let suggestions_for_skipped_reason = |reason: &str| -> Vec<Suggestion> {
        match reason {
            "missing" => vec![
                Suggestion {
                    action: "agentpack evolve restore".to_string(),
                    reason: "restore missing desired outputs (create-only)".to_string(),
                },
                Suggestion {
                    action: "agentpack deploy --apply".to_string(),
                    reason: "re-deploy desired state and write/update manifests".to_string(),
                },
            ],
            "multi_module_output" => vec![
                Suggestion {
                    action: "Add per-module markers to aggregated instructions outputs".to_string(),
                    reason: "allow evolve.propose to map drift back to a single module".to_string(),
                },
                Suggestion {
                    action: "Split aggregated outputs so each file maps to one module".to_string(),
                    reason: "avoid multi-module outputs that cannot be proposed safely".to_string(),
                },
            ],
            _ => Vec::new(),
        }
    };
    let reason_message_for_skipped_reason = |reason: &str| -> String {
        match reason {
            "missing" => "expected managed output is missing on disk (use evolve.restore or deploy to recreate)".to_string(),
            "multi_module_output" => "output is produced by multiple modules and cannot be proposed safely (add markers or split outputs)".to_string(),
            _ => reason.to_string(),
        }
    };
    let next_actions_for_missing = |module_id: Option<&str>| -> Vec<String> {
        let mut actions = Vec::new();
        let mut restore = format!("{prefix} evolve restore");
        if let Some(module_id) = module_id {
            restore.push_str(&format!(" --module-id {module_id}"));
        }
        restore.push_str(" --yes --json");
        actions.push(restore);
        actions.push(format!("{prefix} deploy --apply --yes --json"));
        actions
    };

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
            match &actual {
                None => {
                    summary.drifted_skipped += 1;
                    summary.skipped_missing += 1;
                    let reason = "missing".to_string();
                    skipped.push(SkippedItem {
                        target: tp.target.clone(),
                        path: tp.path.to_string_lossy().to_string(),
                        path_posix: crate::paths::path_to_posix_string(&tp.path),
                        reason_code: reason.clone(),
                        reason_message: reason_message_for_skipped_reason(&reason),
                        next_actions: next_actions_for_missing(None),
                        reason,
                        module_id: None,
                        module_ids: desired_file.module_ids.clone(),
                        suggestions: suggestions_for_skipped_reason("missing"),
                    });
                }
                Some(actual) => {
                    if let Some(section_candidates) = try_propose_marked_instructions_sections(
                        &desired_file.bytes,
                        actual,
                        &desired_file.module_ids,
                    )? {
                        for (module_id, bytes) in section_candidates {
                            if let Some(prev) = instructions_sections.get(&module_id) {
                                if prev != &bytes {
                                    warnings.push(format!(
                                        "evolve.propose: skipped {} {} section for {}: conflicting edits across aggregated outputs",
                                        tp.target,
                                        tp.path.display(),
                                        module_id
                                    ));
                                    continue;
                                }
                                continue;
                            }

                            instructions_sections.insert(module_id.clone(), bytes.clone());
                            summary.drifted_proposeable += 1;
                            candidates.push((module_id, tp.clone(), bytes));
                        }
                        continue;
                    }

                    summary.drifted_skipped += 1;
                    summary.skipped_multi_module += 1;
                    let reason = "multi_module_output".to_string();
                    skipped.push(SkippedItem {
                        target: tp.target.clone(),
                        path: tp.path.to_string_lossy().to_string(),
                        path_posix: crate::paths::path_to_posix_string(&tp.path),
                        reason_code: reason.clone(),
                        reason_message: reason_message_for_skipped_reason(&reason),
                        next_actions: Vec::new(),
                        reason,
                        module_id: None,
                        module_ids: desired_file.module_ids.clone(),
                        suggestions: suggestions_for_skipped_reason("multi_module_output"),
                    });
                }
            }
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
                let reason = "missing".to_string();
                skipped.push(SkippedItem {
                    target: tp.target.clone(),
                    path: tp.path.to_string_lossy().to_string(),
                    path_posix: crate::paths::path_to_posix_string(&tp.path),
                    reason_code: reason.clone(),
                    reason_message: reason_message_for_skipped_reason(&reason),
                    next_actions: next_actions_for_missing(Some(module_id.as_str())),
                    reason,
                    module_id: Some(module_id),
                    module_ids: Vec::new(),
                    suggestions: suggestions_for_skipped_reason("missing"),
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
            path_posix: crate::paths::path_to_posix_string(&tp.path),
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
                        match s.reason.as_str() {
                            "missing" => {
                                println!(
                                    "  hint: run agentpack evolve restore (create-only) or agentpack deploy --apply"
                                );
                            }
                            "multi_module_output" => {
                                println!(
                                    "  hint: add per-module markers to aggregated outputs or split outputs so each file maps to one module"
                                );
                            }
                            _ => {}
                        }
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
                    match s.reason.as_str() {
                        "missing" => {
                            println!(
                                "  hint: run agentpack evolve restore (create-only) or agentpack deploy --apply"
                            );
                        }
                        "multi_module_output" => {
                            println!(
                                "  hint: add per-module markers to aggregated outputs or split outputs so each file maps to one module"
                            );
                        }
                        _ => {}
                    }
                }
            }
        }
        return Ok(());
    }

    if cli.json && !cli.yes {
        return Err(UserError::confirm_required("evolve propose"));
    }
    if !cli.json && !cli.yes && !super::super::util::confirm("Create evolve proposal branch?")? {
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
        let Some(module_rel) =
            super::super::util::module_rel_path_for_output(module, module_id, output, &roots)
        else {
            continue;
        };

        let overlay_dir = match scope {
            EvolveScope::Global => {
                super::super::util::overlay_dir_for_scope(engine, module_id, OverlayScope::Global)
            }
            EvolveScope::Machine => {
                super::super::util::overlay_dir_for_scope(engine, module_id, OverlayScope::Machine)
            }
            EvolveScope::Project => {
                super::super::util::overlay_dir_for_scope(engine, module_id, OverlayScope::Project)
            }
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
        let files_posix: Vec<String> = touched.iter().map(|p| p.replace('\\', "/")).collect();
        let envelope = JsonEnvelope::ok(
            "evolve.propose",
            serde_json::json!({
                "created": true,
                "branch": branch,
                "scope": scope,
                "files": touched,
                "files_posix": files_posix,
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

fn evolve_restore(
    cli: &super::super::args::Cli,
    engine: &Engine,
    module_filter: Option<&str>,
) -> anyhow::Result<()> {
    #[derive(Debug, Clone, serde::Serialize)]
    struct RestoreItem {
        target: String,
        path: String,
        path_posix: String,
        #[serde(skip_serializing_if = "Vec::is_empty", default)]
        module_ids: Vec<String>,
    }

    #[derive(Default, serde::Serialize)]
    struct RestoreSummary {
        missing: u64,
        restored: u64,
        skipped_existing: u64,
        skipped_read_error: u64,
    }

    let render = engine.desired_state(&cli.profile, &cli.target)?;
    let desired = render.desired;

    let mut warnings = render.warnings;
    let mut summary = RestoreSummary::default();
    let mut missing: Vec<(TargetPath, Vec<u8>, Vec<String>)> = Vec::new();

    for (tp, desired_file) in &desired {
        if let Some(filter) = module_filter {
            if !desired_file.module_ids.iter().any(|id| id == filter) {
                continue;
            }
        }

        match std::fs::metadata(&tp.path) {
            Ok(_) => {
                summary.skipped_existing += 1;
            }
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                summary.missing += 1;
                missing.push((
                    tp.clone(),
                    desired_file.bytes.clone(),
                    desired_file.module_ids.clone(),
                ));
            }
            Err(err) => {
                summary.skipped_read_error += 1;
                warnings.push(format!(
                    "evolve.restore: skipped {} {}: failed to stat path: {err}",
                    tp.target,
                    tp.path.display()
                ));
            }
        }
    }

    if missing.is_empty() {
        if cli.json {
            let mut envelope = JsonEnvelope::ok(
                "evolve.restore",
                serde_json::json!({
                    "restored": [],
                    "summary": summary,
                    "reason": "no_missing",
                }),
            );
            envelope.warnings = warnings;
            print_json(&envelope)?;
        } else {
            for w in warnings {
                eprintln!("Warning: {w}");
            }
            println!("No missing desired outputs to restore");
        }
        return Ok(());
    }

    if cli.json && !cli.yes && !cli.dry_run {
        return Err(UserError::confirm_required("evolve restore"));
    }
    if !cli.json
        && !cli.yes
        && !cli.dry_run
        && !super::super::util::confirm("Restore missing desired outputs?")?
    {
        println!("Aborted");
        return Ok(());
    }

    let mut restored: Vec<RestoreItem> = Vec::new();
    for (tp, bytes, module_ids) in missing {
        if !cli.dry_run {
            crate::fs::write_atomic(&tp.path, &bytes)
                .with_context(|| format!("write {}", tp.path.display()))?;
            summary.restored += 1;
        }

        restored.push(RestoreItem {
            target: tp.target.clone(),
            path: tp.path.to_string_lossy().to_string(),
            path_posix: crate::paths::path_to_posix_string(&tp.path),
            module_ids,
        });
    }

    restored.sort_by(|a, b| {
        (a.target.as_str(), a.path.as_str()).cmp(&(b.target.as_str(), b.path.as_str()))
    });

    if cli.json {
        let reason = if cli.dry_run { "dry_run" } else { "restored" };

        let mut envelope = JsonEnvelope::ok(
            "evolve.restore",
            serde_json::json!({
                "restored": restored,
                "summary": summary,
                "reason": reason,
            }),
        );
        envelope.warnings = warnings;
        print_json(&envelope)?;
    } else {
        for w in warnings {
            eprintln!("Warning: {w}");
        }
        if cli.dry_run {
            println!("Would restore missing desired outputs (dry-run):");
        } else {
            println!("Restored missing desired outputs:");
        }
        for item in restored {
            println!("- {} {}", item.target, item.path);
        }
    }

    Ok(())
}

fn action_prefix(cli: &crate::cli::args::Cli) -> String {
    let mut out = String::from("agentpack");
    if let Some(repo) = &cli.repo {
        out.push_str(&format!(" --repo {}", repo.display()));
    }
    if cli.profile != "default" {
        out.push_str(&format!(" --profile {}", cli.profile));
    }
    if cli.target != "all" {
        out.push_str(&format!(" --target {}", cli.target));
    }
    if let Some(machine) = &cli.machine {
        out.push_str(&format!(" --machine {machine}"));
    }
    out
}
