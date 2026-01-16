use std::path::{Path, PathBuf};

use anyhow::Context as _;

use crate::config::{LocalPathSource, Manifest, Module, ModuleType, Source};
use crate::fs::{copy_tree, write_atomic};
use crate::output::{JsonEnvelope, print_json};
use crate::project::ProjectContext;
use crate::user_error::UserError;
use crate::validate::validate_materialized_module;

use super::Ctx;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Scope {
    User,
    Project,
}

impl Scope {
    fn as_str(&self) -> &'static str {
        match self {
            Scope::User => "user",
            Scope::Project => "project",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PlanOp {
    Create,
    SkipExistingModule,
    SkipInvalid,
}

impl PlanOp {
    fn as_str(&self) -> &'static str {
        match self {
            PlanOp::Create => "create",
            PlanOp::SkipExistingModule => "skip_existing_module",
            PlanOp::SkipInvalid => "skip_invalid",
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
struct ImportPlanItem {
    op: String,
    module_id: String,
    module_type: String,
    scope: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    targets: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tags: Vec<String>,
    source_path: String,
    source_path_posix: String,
    dest_path: String,
    dest_path_posix: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    skip_reason: Option<String>,
}

#[derive(Default, Debug, Clone, serde::Serialize)]
struct ImportSummary {
    candidates: u64,
    create: u64,
    skipped_existing_module: u64,
    skipped_invalid: u64,
}

#[derive(Debug, Clone)]
struct PlannedImport {
    op: PlanOp,
    module: Option<Module>,
    module_type: ModuleType,
    module_id: String,
    scope: Scope,
    src: PathBuf,
    dst: PathBuf,
    targets: Vec<String>,
    tags: Vec<String>,
    skip_reason: Option<String>,
}

#[derive(Debug, Clone)]
struct PlanInput {
    module_type: ModuleType,
    module_id: String,
    scope: Scope,
    src: PathBuf,
    dst: PathBuf,
    dst_rel_posix: String,
    targets: Vec<String>,
    tags: Vec<String>,
}

pub(crate) fn run(ctx: &Ctx<'_>, apply: bool, home_root: Option<&PathBuf>) -> anyhow::Result<()> {
    let want_apply = apply && !ctx.cli.dry_run;
    if want_apply {
        super::super::util::require_yes_for_json_mutation(ctx.cli, "import --apply")?;
    }

    let mut manifest = Manifest::load(&ctx.repo.manifest_path).context("load manifest")?;

    let cwd = std::env::current_dir().context("current dir")?;
    let project = ProjectContext::detect(&cwd).context("detect project")?;

    let home_root = match home_root.cloned() {
        Some(p) => p,
        None => dirs::home_dir().context("resolve home dir")?,
    };

    let project_tag = format!("project-{}", project.project_id);
    let project_profile = format!("project-{}", project.project_id);

    let mut warnings = Vec::new();
    let plan = build_plan(
        ctx,
        &manifest,
        &project,
        &home_root,
        &project_tag,
        &mut warnings,
    )?;

    let (plan_items, summary) = plan_to_output(&plan);
    let has_project_items = plan.iter().any(|p| p.scope == Scope::Project);
    let has_creates = plan.iter().any(|p| p.op == PlanOp::Create);
    let applied = want_apply && has_creates;
    let next_actions = build_next_actions(ctx, &project_profile, has_project_items);

    if ctx.cli.json {
        if applied {
            apply_imports(ctx, &mut manifest, &project_profile, &project_tag, &plan)?;
        }

        let mut data = serde_json::json!({
            "applied": applied,
            "repo": ctx.repo.repo_dir,
            "repo_posix": crate::paths::path_to_posix_string(&ctx.repo.repo_dir),
            "home_root": home_root,
            "home_root_posix": crate::paths::path_to_posix_string(&home_root),
            "project": {
                "project_id": project.project_id,
                "project_root": project.project_root,
                "project_root_posix": crate::paths::path_to_posix_string(&project.project_root),
                "origin_url": project.origin_url,
            },
            "plan": plan_items,
            "summary": summary,
            "next_actions": next_actions,
        });

        if !want_apply {
            data.as_object_mut()
                .context("import json data must be an object")?
                .insert(
                    "reason".to_string(),
                    serde_json::Value::String("dry_run".to_string()),
                );
        } else if !applied {
            data.as_object_mut()
                .context("import json data must be an object")?
                .insert(
                    "reason".to_string(),
                    serde_json::Value::String("no_changes".to_string()),
                );
        }

        let mut envelope = JsonEnvelope::ok("import", data);
        envelope.warnings = warnings;
        print_json(&envelope)?;
        return Ok(());
    }

    print_human(
        &plan,
        &summary,
        &next_actions,
        &project_profile,
        want_apply,
        applied,
    )?;

    // Apply phase (human mode): confirm, then write + update manifest.
    if !want_apply {
        for w in &warnings {
            eprintln!("Warning: {w}");
        }
        return Ok(());
    }

    if !applied {
        for w in &warnings {
            eprintln!("Warning: {w}");
        }
        println!("No changes");
        return Ok(());
    }

    if !ctx.cli.yes && !super::super::util::confirm("Apply import?")? {
        println!("Aborted");
        return Ok(());
    }

    for w in &warnings {
        eprintln!("Warning: {w}");
    }

    apply_imports(ctx, &mut manifest, &project_profile, &project_tag, &plan)?;

    println!("Import applied");
    Ok(())
}

fn build_plan(
    ctx: &Ctx<'_>,
    manifest: &Manifest,
    project: &ProjectContext,
    home_root: &Path,
    project_tag: &str,
    warnings: &mut Vec<String>,
) -> anyhow::Result<Vec<PlannedImport>> {
    let mut out = Vec::new();

    // User-scope assets.
    let user_prompts_dir = home_root.join(".codex/prompts");
    out.extend(planned_prompts_dir(
        ctx,
        manifest,
        &user_prompts_dir,
        warnings,
    )?);

    let user_skills_dir = home_root.join(".codex/skills");
    out.extend(planned_skill_dirs(
        ctx,
        manifest,
        &user_skills_dir,
        Scope::User,
        "base",
        warnings,
    )?);

    let user_commands_dir = home_root.join(".claude/commands");
    out.extend(planned_commands_dir(
        ctx,
        manifest,
        &user_commands_dir,
        Scope::User,
        "",
        "base",
        warnings,
    )?);

    // Project-scope assets.
    let agents = project.project_root.join("AGENTS.md");
    if agents.is_file() {
        out.push(planned_instructions_project(
            ctx,
            manifest,
            &agents,
            project_tag,
            &project.project_id,
            warnings,
        )?);
    }

    let repo_commands_dir = project.project_root.join(".claude/commands");
    out.extend(planned_commands_dir(
        ctx,
        manifest,
        &repo_commands_dir,
        Scope::Project,
        &format!("project-{}", project.project_id),
        project_tag,
        warnings,
    )?);

    let repo_skills_dir = project.project_root.join(".codex/skills");
    out.extend(planned_skill_dirs(
        ctx,
        manifest,
        &repo_skills_dir,
        Scope::Project,
        project_tag,
        warnings,
    )?);

    // Deterministic ordering.
    out.sort_by(|a, b| {
        (a.module_id.as_str(), a.src.as_os_str()).cmp(&(b.module_id.as_str(), b.src.as_os_str()))
    });

    // Deduplicate by module_id deterministically (keep first).
    let mut seen = std::collections::BTreeSet::new();
    for p in &mut out {
        if !seen.insert(p.module_id.clone()) {
            p.op = PlanOp::SkipInvalid;
            p.module = None;
            p.skip_reason = Some("duplicate_module_id_in_scan".to_string());
        }
    }
    Ok(out)
}

fn planned_instructions_project(
    ctx: &Ctx<'_>,
    manifest: &Manifest,
    src_agents: &Path,
    project_tag: &str,
    project_id: &str,
    warnings: &mut Vec<String>,
) -> anyhow::Result<PlannedImport> {
    let module_id = format!("instructions:project-{project_id}");
    let module_type = ModuleType::Instructions;
    let targets = vec![
        "codex".to_string(),
        "cursor".to_string(),
        "vscode".to_string(),
    ];
    let tags = vec![
        "imported".to_string(),
        "project".to_string(),
        project_tag.to_string(),
    ];

    let rel_dir = PathBuf::from("modules/instructions/imported").join(project_id);
    let dst = ctx.repo.repo_dir.join(&rel_dir);
    let dst_rel = rel_path_posix(&ctx.repo.repo_dir, &dst)?;

    plan_from_source(
        manifest,
        PlanInput {
            module_type,
            module_id,
            scope: Scope::Project,
            src: src_agents.to_path_buf(),
            dst,
            dst_rel_posix: dst_rel,
            targets,
            tags,
        },
        warnings,
    )
}

fn planned_commands_dir(
    ctx: &Ctx<'_>,
    manifest: &Manifest,
    dir: &Path,
    scope: Scope,
    module_id_prefix: &str,
    tag: &str,
    warnings: &mut Vec<String>,
) -> anyhow::Result<Vec<PlannedImport>> {
    let mut out = Vec::new();
    let files = list_markdown_files(dir)?;
    for f in files {
        let stem = file_stem_string(&f);
        let component = sanitize_id_component(&stem);
        let module_id = if module_id_prefix.is_empty() {
            format!("command:{component}")
        } else {
            format!("command:{module_id_prefix}-{component}")
        };

        let targets = vec!["claude_code".to_string()];
        let tags = if tag == "base" {
            vec![
                "base".to_string(),
                "imported".to_string(),
                "user".to_string(),
            ]
        } else {
            vec![
                "imported".to_string(),
                "project".to_string(),
                tag.to_string(),
            ]
        };

        let rel_dir = if scope == Scope::User {
            PathBuf::from("modules/claude-commands/imported/user")
        } else {
            PathBuf::from("modules/claude-commands/imported").join(tag)
        };
        let dst = ctx.repo.repo_dir.join(&rel_dir).join(
            f.file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("command.md"),
        );
        let dst_rel = rel_path_posix(&ctx.repo.repo_dir, &dst)?;

        out.push(plan_from_source(
            manifest,
            PlanInput {
                module_type: ModuleType::Command,
                module_id,
                scope,
                src: f,
                dst,
                dst_rel_posix: dst_rel,
                targets,
                tags,
            },
            warnings,
        )?);
    }
    Ok(out)
}

fn planned_prompts_dir(
    ctx: &Ctx<'_>,
    manifest: &Manifest,
    dir: &Path,
    warnings: &mut Vec<String>,
) -> anyhow::Result<Vec<PlannedImport>> {
    let mut out = Vec::new();
    let files = list_markdown_files(dir)?;
    for f in files {
        let stem = file_stem_string(&f);
        let component = sanitize_id_component(&stem);
        let module_id = format!("prompt:{component}");

        let targets = vec!["codex".to_string()];
        let tags = vec![
            "base".to_string(),
            "imported".to_string(),
            "user".to_string(),
        ];

        let rel_dir = PathBuf::from("modules/prompts/imported");
        let dst = ctx.repo.repo_dir.join(&rel_dir).join(
            f.file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("prompt.md"),
        );
        let dst_rel = rel_path_posix(&ctx.repo.repo_dir, &dst)?;

        out.push(plan_from_source(
            manifest,
            PlanInput {
                module_type: ModuleType::Prompt,
                module_id,
                scope: Scope::User,
                src: f,
                dst,
                dst_rel_posix: dst_rel,
                targets,
                tags,
            },
            warnings,
        )?);
    }
    Ok(out)
}

fn planned_skill_dirs(
    ctx: &Ctx<'_>,
    manifest: &Manifest,
    dir: &Path,
    scope: Scope,
    tag: &str,
    warnings: &mut Vec<String>,
) -> anyhow::Result<Vec<PlannedImport>> {
    let mut out = Vec::new();
    let skill_dirs = list_immediate_dirs(dir)?;
    for d in skill_dirs {
        let name = d
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("skill")
            .to_string();
        let component = sanitize_id_component(&name);
        let module_id = format!("skill:{component}");

        let targets = vec!["codex".to_string()];
        let tags = if tag == "base" {
            vec![
                "base".to_string(),
                "imported".to_string(),
                "user".to_string(),
            ]
        } else {
            vec![
                "imported".to_string(),
                "project".to_string(),
                tag.to_string(),
            ]
        };

        // Keep skills rooted by module_id to preserve output dir naming.
        let rel_dir = PathBuf::from("modules/skills/imported").join(&component);
        let dst = ctx.repo.repo_dir.join(&rel_dir);
        let dst_rel = rel_path_posix(&ctx.repo.repo_dir, &dst)?;

        out.push(plan_from_source(
            manifest,
            PlanInput {
                module_type: ModuleType::Skill,
                module_id,
                scope,
                src: d,
                dst,
                dst_rel_posix: dst_rel,
                targets,
                tags,
            },
            warnings,
        )?);
    }
    Ok(out)
}

fn plan_from_source(
    manifest: &Manifest,
    input: PlanInput,
    warnings: &mut Vec<String>,
) -> anyhow::Result<PlannedImport> {
    if manifest.modules.iter().any(|m| m.id == input.module_id) {
        return Ok(PlannedImport {
            op: PlanOp::SkipExistingModule,
            module: None,
            module_type: input.module_type,
            module_id: input.module_id,
            scope: input.scope,
            src: input.src,
            dst: input.dst,
            targets: input.targets,
            tags: input.tags,
            skip_reason: Some("module_id_already_exists".to_string()),
        });
    }

    let skip_reason = validate_source(&input.module_type, &input.module_id, &input.src, warnings)?;
    if let Some(reason) = skip_reason {
        return Ok(PlannedImport {
            op: PlanOp::SkipInvalid,
            module: None,
            module_type: input.module_type,
            module_id: input.module_id,
            scope: input.scope,
            src: input.src,
            dst: input.dst,
            targets: input.targets,
            tags: input.tags,
            skip_reason: Some(reason),
        });
    }

    let local_path = LocalPathSource {
        path: input.dst_rel_posix,
    };
    let module = Module {
        id: input.module_id.clone(),
        module_type: input.module_type.clone(),
        enabled: true,
        tags: input.tags.clone(),
        targets: input.targets.clone(),
        source: Source {
            local_path: Some(local_path),
            git: None,
        },
        metadata: Default::default(),
    };

    Ok(PlannedImport {
        op: PlanOp::Create,
        module: Some(module),
        module_type: input.module_type,
        module_id: input.module_id,
        scope: input.scope,
        src: input.src,
        dst: input.dst,
        targets: input.targets,
        tags: input.tags,
        skip_reason: None,
    })
}

fn validate_source(
    module_type: &ModuleType,
    module_id: &str,
    src: &Path,
    warnings: &mut Vec<String>,
) -> anyhow::Result<Option<String>> {
    if !src.exists() {
        return Ok(Some("source_missing".to_string()));
    }

    let tmp = tempfile::tempdir().context("create tempdir")?;
    let dst = tmp.path().join("materialized");
    std::fs::create_dir_all(&dst).context("create materialized dir")?;
    copy_tree(src, &dst).with_context(|| format!("copy source {}", src.display()))?;

    match validate_materialized_module(module_type, module_id, &dst) {
        Ok(()) => Ok(None),
        Err(err) => {
            warnings.push(format!(
                "skipping invalid module source for {module_id}: {err}"
            ));
            Ok(Some("invalid_module_source".to_string()))
        }
    }
}

fn plan_to_output(plan: &[PlannedImport]) -> (Vec<ImportPlanItem>, ImportSummary) {
    let mut summary = ImportSummary {
        candidates: plan.len() as u64,
        ..Default::default()
    };

    let mut items = Vec::new();
    for p in plan {
        match p.op {
            PlanOp::Create => summary.create += 1,
            PlanOp::SkipExistingModule => summary.skipped_existing_module += 1,
            PlanOp::SkipInvalid => summary.skipped_invalid += 1,
        }

        items.push(ImportPlanItem {
            op: p.op.as_str().to_string(),
            module_id: p.module_id.clone(),
            module_type: module_type_str(&p.module_type).to_string(),
            scope: p.scope.as_str().to_string(),
            targets: p.targets.clone(),
            tags: p.tags.clone(),
            source_path: p.src.to_string_lossy().to_string(),
            source_path_posix: crate::paths::path_to_posix_string(&p.src),
            dest_path: p.dst.to_string_lossy().to_string(),
            dest_path_posix: crate::paths::path_to_posix_string(&p.dst),
            skip_reason: p.skip_reason.clone(),
        });
    }

    (items, summary)
}

fn module_type_str(t: &ModuleType) -> &'static str {
    match t {
        ModuleType::Instructions => "instructions",
        ModuleType::Skill => "skill",
        ModuleType::Prompt => "prompt",
        ModuleType::Command => "command",
    }
}

fn build_next_actions(
    ctx: &Ctx<'_>,
    project_profile: &str,
    has_project_items: bool,
) -> Vec<String> {
    let prefix = action_prefix(ctx.cli);
    let prefix_project = format!(
        "{} --profile {}",
        action_prefix_without_profile(ctx.cli),
        project_profile
    );

    let mut out = Vec::new();
    if has_project_items {
        out.push(format!("{prefix_project} preview --diff"));
        out.push(format!("{prefix_project} deploy --apply"));
        if ctx.cli.json {
            out.push(format!("{prefix_project} preview --diff --json"));
            out.push(format!("{prefix_project} deploy --apply --yes --json"));
        }
    } else {
        out.push(format!("{prefix} preview --diff"));
        out.push(format!("{prefix} deploy --apply"));
        if ctx.cli.json {
            out.push(format!("{prefix} preview --diff --json"));
            out.push(format!("{prefix} deploy --apply --yes --json"));
        }
    }

    out
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

fn action_prefix_without_profile(cli: &crate::cli::args::Cli) -> String {
    let mut out = String::from("agentpack");
    if let Some(repo) = &cli.repo {
        out.push_str(&format!(" --repo {}", repo.display()));
    }
    if cli.target != "all" {
        out.push_str(&format!(" --target {}", cli.target));
    }
    if let Some(machine) = &cli.machine {
        out.push_str(&format!(" --machine {machine}"));
    }
    out
}

fn print_human(
    plan: &[PlannedImport],
    summary: &ImportSummary,
    next_actions: &[String],
    project_profile: &str,
    want_apply: bool,
    applied: bool,
) -> anyhow::Result<()> {
    println!(
        "Import plan: candidates={} create={} skipped_existing_module={} skipped_invalid={}",
        summary.candidates,
        summary.create,
        summary.skipped_existing_module,
        summary.skipped_invalid
    );

    if !plan.is_empty() {
        println!();
        for p in plan {
            let scope = p.scope.as_str();
            println!(
                "- [{}] {} {} ({})",
                p.op.as_str(),
                module_type_str(&p.module_type),
                p.module_id,
                scope
            );
        }
    }

    println!();
    if want_apply {
        if applied {
            println!("Will apply import (use --dry-run to force no writes).");
        } else {
            println!("Apply requested, but no changes detected.");
        }
    } else {
        println!("Dry-run (use --apply to write modules + update manifest).");
    }

    if plan.iter().any(|p| p.scope == Scope::Project) {
        println!();
        println!("Project profile: {project_profile}");
    }

    if !next_actions.is_empty() {
        println!();
        println!("Next actions:");
        for a in next_actions {
            println!("- {a}");
        }
    }

    Ok(())
}

fn apply_imports(
    ctx: &Ctx<'_>,
    manifest: &mut Manifest,
    project_profile: &str,
    project_tag: &str,
    plan: &[PlannedImport],
) -> anyhow::Result<()> {
    let mut conflicts = Vec::new();
    for p in plan.iter().filter(|p| p.op == PlanOp::Create) {
        if p.dst.exists() {
            conflicts.push(p.dst.to_string_lossy().to_string());
        }
    }
    if !conflicts.is_empty() {
        conflicts.sort();
        conflicts.truncate(20);
        return Err(anyhow::Error::new(
            UserError::new(
                "E_IMPORT_CONFLICT",
                "import destination already exists; refusing to overwrite",
            )
            .with_details(serde_json::json!({
                "sample_paths": conflicts,
                "hint": "delete or move the conflicting paths, then re-run import",
            })),
        ));
    }

    // Write module files first (so manifest never points at missing sources).
    for p in plan.iter().filter(|p| p.op == PlanOp::Create) {
        if p.module_type == ModuleType::Skill {
            copy_tree(&p.src, &p.dst).with_context(|| {
                format!("copy skill {} -> {}", p.src.display(), p.dst.display())
            })?;
        } else if p.module_type == ModuleType::Instructions {
            let bytes =
                std::fs::read(&p.src).with_context(|| format!("read {}", p.src.display()))?;
            let dst_file = p.dst.join("AGENTS.md");
            write_atomic(&dst_file, &bytes)
                .with_context(|| format!("write {}", dst_file.display()))?;
        } else {
            let bytes =
                std::fs::read(&p.src).with_context(|| format!("read {}", p.src.display()))?;
            write_atomic(&p.dst, &bytes).with_context(|| format!("write {}", p.dst.display()))?;
        }
    }

    let mut added_any = false;
    for p in plan.iter().filter(|p| p.op == PlanOp::Create) {
        let Some(module) = &p.module else {
            continue;
        };
        manifest.modules.push(module.clone());
        added_any = true;
    }

    if plan
        .iter()
        .any(|p| p.scope == Scope::Project && p.op == PlanOp::Create)
    {
        let profile = manifest
            .profiles
            .entry(project_profile.to_string())
            .or_insert(crate::config::Profile {
                include_tags: vec!["base".to_string(), project_tag.to_string()],
                include_modules: Vec::new(),
                exclude_modules: Vec::new(),
            });
        if !profile.include_tags.iter().any(|t| t == "base") {
            profile.include_tags.push("base".to_string());
        }
        if !profile.include_tags.iter().any(|t| t == project_tag) {
            profile.include_tags.push(project_tag.to_string());
        }
    }

    if !added_any {
        return Ok(());
    }

    manifest.modules.sort_by(|a, b| a.id.cmp(&b.id));
    manifest
        .save(&ctx.repo.manifest_path)
        .context("save manifest")?;

    Ok(())
}

fn list_markdown_files(dir: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    if !dir.is_dir() {
        return Ok(out);
    }

    for entry in std::fs::read_dir(dir).with_context(|| format!("read {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }
        out.push(path);
    }
    out.sort();
    Ok(out)
}

fn list_immediate_dirs(dir: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    if !dir.is_dir() {
        return Ok(out);
    }

    for entry in std::fs::read_dir(dir).with_context(|| format!("read {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            out.push(path);
        }
    }
    out.sort();
    Ok(out)
}

fn file_stem_string(path: &Path) -> String {
    path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("item")
        .to_string()
}

fn sanitize_id_component(raw: &str) -> String {
    let mut out = String::new();
    for c in raw.trim().chars() {
        let c = c.to_ascii_lowercase();
        if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
            out.push(c);
        } else {
            out.push('-');
        }
    }

    let out = out.trim_matches('-').to_string();
    if out.is_empty() {
        "item".to_string()
    } else {
        out
    }
}

fn rel_path_posix(repo_root: &Path, path: &Path) -> anyhow::Result<String> {
    let rel = path.strip_prefix(repo_root).unwrap_or(path);
    Ok(rel.to_string_lossy().replace('\\', "/"))
}
