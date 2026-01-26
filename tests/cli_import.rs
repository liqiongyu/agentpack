use std::path::{Path, PathBuf};
use std::process::Command;

use agentpack::config::Manifest;

fn agentpack_in(home: &Path, cwd: &Path, args: &[&str]) -> std::process::Output {
    let bin = env!("CARGO_BIN_EXE_agentpack");
    Command::new(bin)
        .current_dir(cwd)
        .args(args)
        .env("AGENTPACK_HOME", home)
        .env("AGENTPACK_MACHINE_ID", "test-machine")
        .env("HOME", home)
        .output()
        .expect("run agentpack")
}

fn git_in(dir: &Path, args: &[&str]) -> std::process::Output {
    Command::new("git")
        .current_dir(dir)
        .args(args)
        .output()
        .expect("run git")
}

fn parse_stdout_json(output: &std::process::Output) -> serde_json::Value {
    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(&stdout).expect("stdout is valid json")
}

fn tags_include(v: &serde_json::Value, expected: &[&str]) -> bool {
    let Some(tags) = v["tags"].as_array() else {
        return false;
    };
    expected
        .iter()
        .all(|t| tags.iter().any(|v| v.as_str() == Some(*t)))
}

fn init_workspace(tmp: &tempfile::TempDir) -> anyhow::Result<PathBuf> {
    let workspace = tmp.path().join("workspace");
    std::fs::create_dir_all(&workspace)?;

    assert!(git_in(&workspace, &["init"]).status.success());
    // Provide a stable origin for deterministic project_id derivation.
    let _ = git_in(
        &workspace,
        &[
            "remote",
            "add",
            "origin",
            "https://github.com/example/example.git",
        ],
    );

    Ok(workspace)
}

fn write_skill(dir: &Path, name: &str, description: &str) -> anyhow::Result<()> {
    std::fs::create_dir_all(dir)?;
    std::fs::write(
        dir.join("SKILL.md"),
        format!(
            r#"---
name: {name}
description: {description}
---

# {name}
"#
        ),
    )?;
    Ok(())
}

fn write_claude_command(path: &Path, description: &str) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(
        path,
        format!(
            r#"---
description: "{description}"
---

# /{}
"#,
            path.file_stem().and_then(|s| s.to_str()).unwrap_or("cmd")
        ),
    )?;
    Ok(())
}

#[test]
fn import_dry_run_and_apply_work_and_update_manifest() -> anyhow::Result<()> {
    let tmp = tempfile::tempdir().expect("tempdir");
    let home = tmp.path();
    let workspace = init_workspace(&tmp)?;

    // Project assets.
    std::fs::write(workspace.join("AGENTS.md"), "# Project AGENTS\n")?;
    write_claude_command(
        &workspace.join(".claude/commands/ap-test.md"),
        "test command",
    )?;
    write_skill(
        &workspace.join(".codex/skills/my-skill"),
        "my-skill",
        "project skill",
    )?;

    // User assets (under a temp home-root).
    let home_root = tmp.path().join("home_root");
    std::fs::create_dir_all(home_root.join(".codex/prompts"))?;
    std::fs::write(home_root.join(".codex/prompts/prompt1.md"), "Prompt 1\n")?;
    write_claude_command(
        &home_root.join(".claude/commands/ap-user.md"),
        "user command",
    )?;
    write_skill(
        &home_root.join(".codex/skills/user-skill"),
        "user-skill",
        "user skill",
    )?;

    // Init config repo.
    let init = agentpack_in(home, &workspace, &["init"]);
    assert!(init.status.success());

    // Dry-run.
    let out = agentpack_in(
        home,
        &workspace,
        &[
            "import",
            "--home-root",
            home_root.to_str().unwrap(),
            "--json",
        ],
    );
    assert!(out.status.success());
    let v = parse_stdout_json(&out);
    assert_eq!(v["ok"], true);
    assert_eq!(v["command"], "import");
    assert_eq!(v["data"]["applied"], false);
    assert_eq!(v["data"]["reason"], "dry_run");

    let project_id = v["data"]["project"]["project_id"]
        .as_str()
        .expect("project_id");

    let plan = v["data"]["plan"].as_array().expect("plan array");
    let expect_ids = vec![
        format!("instructions:project-{project_id}"),
        format!("command:project-{project_id}-ap-test"),
        "prompt:prompt1".to_string(),
        "command:ap-user".to_string(),
        format!("skill:project-{project_id}-my-skill"),
        "skill:user-skill".to_string(),
    ];
    for id in expect_ids {
        assert!(
            plan.iter()
                .any(|p| p["module_id"] == id && p["op"] == "create"),
            "plan contains create for {id}"
        );
    }

    let prompt = plan
        .iter()
        .find(|p| p["module_id"] == "prompt:prompt1")
        .expect("prompt plan item");
    assert!(
        tags_include(prompt, &["imported", "user", "codex"]),
        "prompt tags include imported/user/codex"
    );

    let user_cmd = plan
        .iter()
        .find(|p| p["module_id"] == "command:ap-user")
        .expect("user command plan item");
    assert!(
        tags_include(user_cmd, &["imported", "user", "claude_code"]),
        "user command tags include imported/user/claude_code"
    );

    // Apply.
    let out = agentpack_in(
        home,
        &workspace,
        &[
            "import",
            "--home-root",
            home_root.to_str().unwrap(),
            "--apply",
            "--yes",
            "--json",
        ],
    );
    assert!(out.status.success());
    let v = parse_stdout_json(&out);
    assert_eq!(v["ok"], true);
    assert_eq!(v["command"], "import");
    assert_eq!(v["data"]["applied"], true);

    let repo_dir = home.join("repo");
    assert!(
        repo_dir
            .join("modules/prompts/imported/prompt1.md")
            .is_file()
    );
    assert!(
        repo_dir
            .join("modules/claude-commands/imported/user/ap-user.md")
            .is_file()
    );
    assert!(
        repo_dir
            .join(format!(
                "modules/claude-commands/imported/project-{project_id}/ap-test.md"
            ))
            .is_file()
    );
    assert!(
        repo_dir
            .join(format!(
                "modules/instructions/imported/{project_id}/AGENTS.md"
            ))
            .is_file()
    );
    assert!(
        repo_dir
            .join(format!(
                "modules/skills/imported/project-{project_id}-my-skill/SKILL.md"
            ))
            .is_file()
    );
    assert!(
        repo_dir
            .join("modules/skills/imported/user-skill/SKILL.md")
            .is_file()
    );

    let manifest = Manifest::load(&repo_dir.join("agentpack.yaml"))?;
    assert!(
        manifest
            .modules
            .iter()
            .any(|m| m.id == format!("instructions:project-{project_id}")),
        "manifest contains project instructions module"
    );
    assert!(
        manifest
            .profiles
            .contains_key(&format!("project-{project_id}")),
        "manifest contains project profile"
    );

    Ok(())
}

#[cfg(unix)]
#[test]
fn import_warns_and_succeeds_when_skill_dir_is_symlink_to_dir() -> anyhow::Result<()> {
    use std::os::unix::fs::symlink;

    let tmp = tempfile::tempdir().expect("tempdir");
    let home = tmp.path();
    let workspace = init_workspace(&tmp)?;

    // User skill with a symlinked dir entry under ~/.codex/skills.
    let home_root = tmp.path().join("home_root");
    let real_skill = home_root.join(".agents/skills/copywriting");
    write_skill(&real_skill, "copywriting", "user skill")?;

    let codex_skills = home_root.join(".codex/skills");
    std::fs::create_dir_all(&codex_skills)?;
    symlink(&real_skill, codex_skills.join("copywriting"))?;

    assert!(agentpack_in(home, &workspace, &["init"]).status.success());

    let out = agentpack_in(
        home,
        &workspace,
        &[
            "import",
            "--home-root",
            home_root.to_str().unwrap(),
            "--json",
        ],
    );
    assert!(out.status.success());
    let v = parse_stdout_json(&out);
    assert_eq!(v["ok"], true);
    assert_eq!(v["command"], "import");

    let warnings = v["warnings"].as_array().expect("warnings array");
    assert!(
        warnings.iter().any(|w| {
            w.as_str()
                .unwrap_or("")
                .contains("dereferenced symlinked directory source")
                && w.as_str().unwrap_or("").contains("skill:copywriting")
        }),
        "expected symlink dereference warning for skill:copywriting"
    );

    let plan = v["data"]["plan"].as_array().expect("plan array");
    assert!(
        plan.iter()
            .any(|p| p["module_id"] == "skill:copywriting" && p["op"] == "create"),
        "plan contains create for skill:copywriting"
    );

    Ok(())
}

#[test]
fn import_plan_avoids_duplicate_skill_module_ids_across_scopes() -> anyhow::Result<()> {
    let tmp = tempfile::tempdir().expect("tempdir");
    let home = tmp.path();
    let workspace = init_workspace(&tmp)?;

    // Project skill.
    write_skill(
        &workspace.join(".codex/skills/dup-skill"),
        "dup-skill",
        "project skill",
    )?;

    // User skill (same name).
    let home_root = tmp.path().join("home_root");
    write_skill(
        &home_root.join(".codex/skills/dup-skill"),
        "dup-skill",
        "user skill",
    )?;

    assert!(agentpack_in(home, &workspace, &["init"]).status.success());

    let out = agentpack_in(
        home,
        &workspace,
        &[
            "import",
            "--home-root",
            home_root.to_str().unwrap(),
            "--json",
        ],
    );
    assert!(out.status.success());
    let v = parse_stdout_json(&out);

    let project_id = v["data"]["project"]["project_id"]
        .as_str()
        .expect("project_id");
    let plan = v["data"]["plan"].as_array().expect("plan array");

    let ids: Vec<String> = plan
        .iter()
        .filter_map(|p| p["module_id"].as_str().map(|s| s.to_string()))
        .collect();
    let unique: std::collections::BTreeSet<_> = ids.iter().collect();
    assert_eq!(ids.len(), unique.len(), "module_id values are unique");

    let user_id = "skill:dup-skill".to_string();
    let project_skill_id = format!("skill:project-{project_id}-dup-skill");
    for id in [user_id, project_skill_id] {
        assert!(
            plan.iter()
                .any(|p| p["module_id"] == id && p["op"] == "create"),
            "plan contains create for {id}"
        );
    }

    Ok(())
}

#[test]
fn import_dry_run_reports_destination_conflicts() -> anyhow::Result<()> {
    let tmp = tempfile::tempdir().expect("tempdir");
    let home = tmp.path();
    let workspace = init_workspace(&tmp)?;

    let home_root = tmp.path().join("home_root");
    std::fs::create_dir_all(home_root.join(".codex/prompts"))?;
    std::fs::write(home_root.join(".codex/prompts/prompt1.md"), "Prompt 1\n")?;

    assert!(agentpack_in(home, &workspace, &["init"]).status.success());

    // Pre-create the destination file without adding a module to the manifest.
    let repo_dir = home.join("repo");
    let conflict_path = repo_dir.join("modules/prompts/imported/prompt1.md");
    if let Some(parent) = conflict_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&conflict_path, "existing\n")?;

    let out = agentpack_in(
        home,
        &workspace,
        &[
            "import",
            "--home-root",
            home_root.to_str().unwrap(),
            "--json",
        ],
    );
    assert!(out.status.success());
    let v = parse_stdout_json(&out);
    assert_eq!(v["ok"], true);
    assert_eq!(v["command"], "import");
    assert_eq!(v["data"]["reason"], "dry_run");

    let conflicts = v["data"]["conflicts"].as_array().expect("conflicts array");
    let dest = conflicts
        .iter()
        .find(|c| c["kind"] == "dest_path_exists")
        .expect("dest_path_exists conflict");
    assert!(
        dest["sample_paths_posix"]
            .as_array()
            .expect("sample_paths_posix array")
            .iter()
            .any(|p| {
                p.as_str()
                    .unwrap_or("")
                    .ends_with("repo/modules/prompts/imported/prompt1.md")
            }),
        "conflict includes prompt1 destination"
    );

    let plan = v["data"]["plan"].as_array().expect("plan array");
    let prompt = plan
        .iter()
        .find(|p| p["module_id"] == "prompt:prompt1")
        .expect("prompt plan item");
    assert_eq!(prompt["dest_exists"], true);

    Ok(())
}

#[test]
fn import_dry_run_reports_module_id_collisions() -> anyhow::Result<()> {
    let tmp = tempfile::tempdir().expect("tempdir");
    let home = tmp.path();
    let workspace = init_workspace(&tmp)?;

    // Two different dirs that sanitize to the same module id (skill:dup).
    let home_root = tmp.path().join("home_root");
    write_skill(&home_root.join(".codex/skills/dup"), "dup", "skill one")?;
    write_skill(&home_root.join(".codex/skills/dup!"), "dup", "skill two")?;

    assert!(agentpack_in(home, &workspace, &["init"]).status.success());

    let out = agentpack_in(
        home,
        &workspace,
        &[
            "import",
            "--home-root",
            home_root.to_str().unwrap(),
            "--json",
        ],
    );
    assert!(out.status.success());
    let v = parse_stdout_json(&out);

    let conflicts = v["data"]["conflicts"].as_array().expect("conflicts array");
    let dup = conflicts
        .iter()
        .find(|c| c["kind"] == "duplicate_module_id_in_scan" && c["module_id"] == "skill:dup")
        .expect("duplicate_module_id_in_scan conflict for skill:dup");
    assert_eq!(dup["count"], 2);

    let plan = v["data"]["plan"].as_array().expect("plan array");
    assert!(
        plan.iter()
            .any(|p| p["module_id"] == "skill:dup" && p["op"] == "skip_invalid"),
        "one duplicate is skipped"
    );

    Ok(())
}

#[test]
fn import_apply_refuses_to_overwrite_existing_dest_path() -> anyhow::Result<()> {
    let tmp = tempfile::tempdir().expect("tempdir");
    let home = tmp.path();
    let workspace = init_workspace(&tmp)?;

    let home_root = tmp.path().join("home_root");
    std::fs::create_dir_all(home_root.join(".codex/prompts"))?;
    std::fs::write(home_root.join(".codex/prompts/prompt1.md"), "Prompt 1\n")?;

    assert!(agentpack_in(home, &workspace, &["init"]).status.success());

    // Pre-create the destination file without adding a module to the manifest.
    let repo_dir = home.join("repo");
    let conflict_path = repo_dir.join("modules/prompts/imported/prompt1.md");
    if let Some(parent) = conflict_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&conflict_path, "existing\n")?;

    let out = agentpack_in(
        home,
        &workspace,
        &[
            "import",
            "--home-root",
            home_root.to_str().unwrap(),
            "--apply",
            "--yes",
            "--json",
        ],
    );
    assert!(!out.status.success());
    let v = parse_stdout_json(&out);
    assert_eq!(v["ok"], false);
    assert_eq!(v["command"], "import");
    assert_eq!(v["errors"][0]["code"], "E_IMPORT_CONFLICT");
    assert_eq!(
        v["errors"][0]["details"]["reason_code"].as_str(),
        Some("import_conflict")
    );
    assert_eq!(
        v["errors"][0]["details"]["next_actions"],
        serde_json::json!(["resolve_import_conflict", "retry_import_apply"])
    );
    assert_eq!(v["errors"][0]["details"]["count"], 1);
    assert!(
        v["errors"][0]["details"]["sample_paths_posix"]
            .as_array()
            .expect("sample_paths_posix array")
            .iter()
            .any(|p| {
                p.as_str()
                    .unwrap_or("")
                    .ends_with("repo/modules/prompts/imported/prompt1.md")
            }),
        "details include prompt1 destination"
    );
    Ok(())
}
