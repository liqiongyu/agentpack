use crate::deploy::TargetPath;
use crate::deploy::plan as compute_plan;
use crate::engine::Engine;
use crate::output::{JsonEnvelope, print_json};
use crate::user_error::UserError;

use super::super::args::BootstrapScope;
use super::Ctx;

const TEMPLATE_CODEX_OPERATOR_SKILL: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/templates/codex/skills/agentpack-operator/SKILL.md"
));
const TEMPLATE_CLAUDE_AP_DOCTOR: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/templates/claude/commands/ap-doctor.md"
));
const TEMPLATE_CLAUDE_AP_UPDATE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/templates/claude/commands/ap-update.md"
));
const TEMPLATE_CLAUDE_AP_PREVIEW: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/templates/claude/commands/ap-preview.md"
));
const TEMPLATE_CLAUDE_AP_PLAN: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/templates/claude/commands/ap-plan.md"
));
const TEMPLATE_CLAUDE_AP_DEPLOY: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/templates/claude/commands/ap-deploy.md"
));
const TEMPLATE_CLAUDE_AP_STATUS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/templates/claude/commands/ap-status.md"
));
const TEMPLATE_CLAUDE_AP_DIFF: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/templates/claude/commands/ap-diff.md"
));
const TEMPLATE_CLAUDE_AP_EXPLAIN: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/templates/claude/commands/ap-explain.md"
));
const TEMPLATE_CLAUDE_AP_EVOLVE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/templates/claude/commands/ap-evolve.md"
));
const TEMPLATE_CLAUDE_OPERATOR_SKILL: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/templates/claude/skills/agentpack-operator/SKILL.md"
));

pub(crate) fn build_desired_and_roots(
    engine: &Engine,
    targets: &[String],
    scope: BootstrapScope,
) -> anyhow::Result<(
    crate::deploy::DesiredState,
    Vec<crate::targets::TargetRoot>,
    &'static str,
)> {
    let (allow_user, allow_project) = bootstrap_scope_flags(scope);
    let scope_str = bootstrap_scope_str(scope);

    let mut desired = crate::deploy::DesiredState::new();
    let mut roots: Vec<crate::targets::TargetRoot> = Vec::new();

    if targets.iter().any(|t| t == "codex") {
        let codex_home = super::super::util::codex_home_for_manifest(&engine.manifest)?;
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
        let Some(cfg) = engine.manifest.targets.get("claude_code") else {
            return Ok((desired, roots, scope_str));
        };
        let write_repo_skills = allow_project && get_bool(&cfg.options, "write_repo_skills", false);
        let write_user_skills = allow_user && get_bool(&cfg.options, "write_user_skills", false);

        let bytes_skill = render_operator_template_bytes(TEMPLATE_CLAUDE_OPERATOR_SKILL);
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
            let user_dir = super::super::util::expand_tilde("~/.claude/commands")?;
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

        if allow_user && write_user_skills {
            let dir = super::super::util::expand_tilde("~/.claude/skills")?;
            roots.push(crate::targets::TargetRoot {
                target: "claude_code".to_string(),
                root: dir.clone(),
                scan_extras: true,
            });
            desired.insert(
                TargetPath {
                    target: "claude_code".to_string(),
                    path: dir.join("agentpack-operator/SKILL.md"),
                },
                crate::deploy::DesiredFile {
                    bytes: bytes_skill.clone(),
                    module_ids: vec!["skill:agentpack-operator".to_string()],
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

        if allow_project && write_repo_skills {
            let dir = engine.project.project_root.join(".claude/skills");
            roots.push(crate::targets::TargetRoot {
                target: "claude_code".to_string(),
                root: dir.clone(),
                scan_extras: true,
            });
            desired.insert(
                TargetPath {
                    target: "claude_code".to_string(),
                    path: dir.join("agentpack-operator/SKILL.md"),
                },
                crate::deploy::DesiredFile {
                    bytes: bytes_skill,
                    module_ids: vec!["skill:agentpack-operator".to_string()],
                },
            );
        }
    }

    Ok((desired, roots, scope_str))
}

pub(crate) fn run(ctx: &Ctx<'_>, scope: BootstrapScope) -> anyhow::Result<()> {
    let engine = Engine::load(ctx.cli.repo.as_deref(), ctx.cli.machine.as_deref())?;
    let targets = super::super::util::selected_targets(&engine.manifest, &ctx.cli.target)?;
    let (desired, roots, scope_str) = build_desired_and_roots(&engine, &targets, scope)?;

    let plan = compute_plan(&desired, None)?;

    if !ctx.cli.json {
        println!(
            "Plan: +{} ~{} -{}",
            plan.summary.create, plan.summary.update, plan.summary.delete
        );
        super::super::util::print_diff(&plan, &desired)?;
    }

    if ctx.cli.dry_run {
        if ctx.cli.json {
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
        if ctx.cli.json {
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

    if ctx.cli.json && !ctx.cli.yes {
        return Err(UserError::confirm_required("bootstrap"));
    }

    if !ctx.cli.yes && !ctx.cli.json && !super::super::util::confirm("Apply bootstrap changes?")? {
        println!("Aborted");
        return Ok(());
    }

    let snapshot =
        crate::apply::apply_plan(&engine.home, "bootstrap", &plan, &desired, None, &roots)?;
    if ctx.cli.json {
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

    Ok(())
}

fn render_operator_template_bytes(template: &str) -> Vec<u8> {
    template
        .replace("{{AGENTPACK_VERSION}}", env!("CARGO_PKG_VERSION"))
        .into_bytes()
}

fn get_bool(
    map: &std::collections::BTreeMap<String, serde_yaml::Value>,
    key: &str,
    default: bool,
) -> bool {
    match map.get(key) {
        Some(serde_yaml::Value::Bool(b)) => *b,
        Some(serde_yaml::Value::String(s)) => match s.trim().to_ascii_lowercase().as_str() {
            "true" | "yes" | "1" => true,
            "false" | "no" | "0" => false,
            _ => default,
        },
        _ => default,
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
