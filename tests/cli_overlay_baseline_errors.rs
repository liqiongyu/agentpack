use std::path::Path;
use std::process::Command;

use agentpack::config::{LocalPathSource, Manifest, Module, ModuleType, Source};
use agentpack::ids::module_fs_key;

fn agentpack_in(home: &Path, cwd: &Path, args: &[&str]) -> std::process::Output {
    let bin = env!("CARGO_BIN_EXE_agentpack");
    Command::new(bin)
        .current_dir(cwd)
        .args(args)
        .env("AGENTPACK_HOME", home)
        .env("EDITOR", "")
        .output()
        .expect("run agentpack")
}

fn parse_stdout_json(output: &std::process::Output) -> serde_json::Value {
    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(&stdout).expect("stdout is valid json")
}

fn git_ok(cwd: &Path, args: &[&str]) -> anyhow::Result<()> {
    let out = Command::new("git").current_dir(cwd).args(args).output()?;
    if !out.status.success() {
        anyhow::bail!(
            "git {:?} failed: {}",
            args,
            String::from_utf8_lossy(&out.stderr)
        );
    }
    Ok(())
}

fn add_local_path_skill_module(repo_dir: &Path, module_id: &str) -> anyhow::Result<()> {
    let module_root = repo_dir.join("modules").join(module_id);
    std::fs::create_dir_all(&module_root)?;
    std::fs::write(module_root.join("SKILL.md"), "line1\nline2\nline3\n")?;

    let manifest_path = repo_dir.join("agentpack.yaml");
    let mut manifest = Manifest::load(&manifest_path)?;
    manifest.modules.push(Module {
        id: module_id.to_string(),
        module_type: ModuleType::Skill,
        enabled: true,
        tags: Vec::new(),
        targets: Vec::new(),
        source: Source {
            local_path: Some(LocalPathSource {
                path: format!("modules/{module_id}"),
            }),
            git: None,
        },
        metadata: Default::default(),
    });
    manifest.save(&manifest_path)?;
    Ok(())
}

#[test]
fn overlay_rebase_json_includes_guidance_when_overlay_is_missing() -> anyhow::Result<()> {
    let tmp = tempfile::tempdir()?;
    let home = tmp.path();

    let project_dir = home.join("project");
    std::fs::create_dir_all(&project_dir)?;

    let init = agentpack_in(home, &project_dir, &["init"]);
    assert!(init.status.success());

    let repo_dir = home.join("repo");
    add_local_path_skill_module(&repo_dir, "skill_test")?;

    let rebase = agentpack_in(
        home,
        &project_dir,
        &[
            "overlay",
            "rebase",
            "skill_test",
            "--scope",
            "global",
            "--json",
            "--yes",
        ],
    );
    assert!(
        !rebase.status.success(),
        "rebase should fail when overlay is missing"
    );

    let v = parse_stdout_json(&rebase);
    assert_eq!(v["ok"], false);
    assert_eq!(v["errors"][0]["code"], "E_OVERLAY_NOT_FOUND");
    assert_eq!(
        v["errors"][0]["details"]["reason_code"].as_str(),
        Some("overlay_not_found")
    );
    assert_eq!(
        v["errors"][0]["details"]["next_actions"],
        serde_json::json!(["run_overlay_edit", "retry_command"])
    );

    Ok(())
}

#[test]
fn overlay_rebase_json_includes_guidance_when_baseline_is_missing() -> anyhow::Result<()> {
    let tmp = tempfile::tempdir()?;
    let home = tmp.path();

    let project_dir = home.join("project");
    std::fs::create_dir_all(&project_dir)?;

    let init = agentpack_in(home, &project_dir, &["init"]);
    assert!(init.status.success());

    let repo_dir = home.join("repo");
    add_local_path_skill_module(&repo_dir, "skill_test")?;

    // Create overlay dir without baseline metadata.
    let overlay_dir = repo_dir.join("overlays").join(module_fs_key("skill_test"));
    std::fs::create_dir_all(&overlay_dir)?;

    let rebase = agentpack_in(
        home,
        &project_dir,
        &[
            "overlay",
            "rebase",
            "skill_test",
            "--scope",
            "global",
            "--json",
            "--yes",
        ],
    );
    assert!(
        !rebase.status.success(),
        "rebase should fail when baseline metadata is missing"
    );

    let v = parse_stdout_json(&rebase);
    assert_eq!(v["ok"], false);
    assert_eq!(v["errors"][0]["code"], "E_OVERLAY_BASELINE_MISSING");
    assert_eq!(
        v["errors"][0]["details"]["reason_code"].as_str(),
        Some("overlay_baseline_missing")
    );
    assert_eq!(
        v["errors"][0]["details"]["next_actions"],
        serde_json::json!(["run_overlay_edit", "retry_command"])
    );

    Ok(())
}

#[test]
fn overlay_rebase_json_includes_guidance_when_baseline_is_unsupported() -> anyhow::Result<()> {
    let tmp = tempfile::tempdir()?;
    let home = tmp.path();

    let project_dir = home.join("project");
    std::fs::create_dir_all(&project_dir)?;

    let init = agentpack_in(home, &project_dir, &["init"]);
    assert!(init.status.success());

    let repo_dir = home.join("repo");

    // Put the config repo under git so local-path baselines can locate merge bases.
    git_ok(&repo_dir, &["init"])?;
    git_ok(&repo_dir, &["config", "user.email", "test@example.com"])?;
    git_ok(&repo_dir, &["config", "user.name", "Test"])?;
    git_ok(&repo_dir, &["add", "."])?;
    git_ok(&repo_dir, &["commit", "-m", "init"])?;

    add_local_path_skill_module(&repo_dir, "skill_test")?;
    git_ok(&repo_dir, &["add", "."])?;
    git_ok(&repo_dir, &["commit", "-m", "add skill_test module"])?;

    // Create overlay baseline file without upstream identity.
    let overlay_meta_dir = repo_dir
        .join("overlays")
        .join(module_fs_key("skill_test"))
        .join(".agentpack");
    std::fs::create_dir_all(&overlay_meta_dir)?;
    let baseline_path = overlay_meta_dir.join("baseline.json");
    std::fs::write(
        baseline_path,
        r#"{
  "version": 1,
  "created_at": "t",
  "upstream_sha256": "deadbeef",
  "file_manifest": []
}
"#,
    )?;

    let rebase = agentpack_in(
        home,
        &project_dir,
        &[
            "overlay",
            "rebase",
            "skill_test",
            "--scope",
            "global",
            "--json",
            "--yes",
        ],
    );
    assert!(
        !rebase.status.success(),
        "rebase should fail when baseline cannot locate merge base"
    );

    let v = parse_stdout_json(&rebase);
    assert_eq!(v["ok"], false);
    assert_eq!(v["errors"][0]["code"], "E_OVERLAY_BASELINE_UNSUPPORTED");
    assert_eq!(
        v["errors"][0]["details"]["reason_code"].as_str(),
        Some("overlay_baseline_unsupported")
    );
    assert_eq!(
        v["errors"][0]["details"]["next_actions"],
        serde_json::json!([
            "init_git_repo",
            "commit_or_stash",
            "run_overlay_edit",
            "retry_command"
        ])
    );

    Ok(())
}
