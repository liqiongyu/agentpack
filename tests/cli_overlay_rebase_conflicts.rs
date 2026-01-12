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

fn git(cwd: &Path, args: &[&str]) -> anyhow::Result<String> {
    let out = Command::new("git").current_dir(cwd).args(args).output()?;
    if !out.status.success() {
        anyhow::bail!(
            "git {:?} failed: {}",
            args,
            String::from_utf8_lossy(&out.stderr)
        );
    }
    Ok(String::from_utf8(out.stdout)?)
}

#[test]
fn overlay_rebase_conflicts_return_stable_json_error_code() -> anyhow::Result<()> {
    let tmp = tempfile::tempdir()?;
    let home = tmp.path();

    let project_dir = home.join("project");
    std::fs::create_dir_all(&project_dir)?;

    let init = agentpack_in(home, &project_dir, &["init"]);
    assert!(init.status.success());

    let repo_dir = home.join("repo");

    // Put the config repo under git so local-path baselines can record a stable merge base.
    let _ = git(&repo_dir, &["init"])?;
    let _ = git(&repo_dir, &["config", "user.email", "test@example.com"])?;
    let _ = git(&repo_dir, &["config", "user.name", "Test"])?;
    let _ = git(&repo_dir, &["add", "."])?;
    let _ = git(&repo_dir, &["commit", "-m", "init"])?;

    // Add a local-path module.
    let module_root = repo_dir.join("modules").join("skill_test");
    std::fs::create_dir_all(&module_root)?;
    std::fs::write(module_root.join("SKILL.md"), "line1\nline2\nline3\n")?;

    let manifest_path = repo_dir.join("agentpack.yaml");
    let mut manifest = Manifest::load(&manifest_path)?;
    manifest.modules.push(Module {
        id: "skill_test".to_string(),
        module_type: ModuleType::Skill,
        enabled: true,
        tags: Vec::new(),
        targets: Vec::new(),
        source: Source {
            local_path: Some(LocalPathSource {
                path: "modules/skill_test".to_string(),
            }),
            git: None,
        },
        metadata: Default::default(),
    });
    manifest.save(&manifest_path)?;

    let _ = git(&repo_dir, &["add", "."])?;
    let _ = git(&repo_dir, &["commit", "-m", "add skill_test module"])?;

    // Create overlay baseline.
    let edit = agentpack_in(home, &project_dir, &["overlay", "edit", "skill_test"]);
    assert!(edit.status.success());

    let overlay_dir = repo_dir.join("overlays").join(module_fs_key("skill_test"));

    // Simulate overlay edit and upstream edit on the same line to force conflict.
    std::fs::write(overlay_dir.join("SKILL.md"), "line1\nline2-ours\nline3\n")?;
    std::fs::write(module_root.join("SKILL.md"), "line1\nline2-theirs\nline3\n")?;

    let _ = git(&repo_dir, &["add", "."])?;
    let _ = git(&repo_dir, &["commit", "-m", "upstream change"])?;

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
    assert!(!rebase.status.success(), "rebase should fail on conflict");

    let v = parse_stdout_json(&rebase);
    assert_eq!(v["ok"], false);
    assert_eq!(v["errors"][0]["code"], "E_OVERLAY_REBASE_CONFLICT");

    Ok(())
}
