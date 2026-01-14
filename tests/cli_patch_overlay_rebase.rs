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
        .env("HOME", home)
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
fn overlay_rebase_updates_patch_files_against_upstream() -> anyhow::Result<()> {
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
    std::fs::write(
        module_root.join("SKILL.md"),
        "---\nname: my-skill\ndescription: base\n---\nline2\nline3\n",
    )?;

    let manifest_path = repo_dir.join("agentpack.yaml");
    let mut manifest = Manifest::load(&manifest_path)?;
    manifest.modules.push(Module {
        id: "skill_test".to_string(),
        module_type: ModuleType::Skill,
        enabled: true,
        tags: vec!["base".to_string()],
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

    // Create sparse overlay metadata.
    let edit = agentpack_in(
        home,
        &project_dir,
        &[
            "overlay",
            "edit",
            "skill_test",
            "--scope",
            "global",
            "--sparse",
        ],
    );
    assert!(edit.status.success());

    let overlay_dir = repo_dir.join("overlays").join(module_fs_key("skill_test"));
    std::fs::create_dir_all(overlay_dir.join(".agentpack/patches"))?;
    std::fs::write(
        overlay_dir.join(".agentpack/overlay.json"),
        "{\n  \"overlay_kind\": \"patch\"\n}\n",
    )?;

    // Patch changes the last line; we'll insert an upstream line to force the patch to be rewritten.
    let patch_path = overlay_dir.join(".agentpack/patches/SKILL.md.patch");
    std::fs::write(
        &patch_path,
        "--- a/SKILL.md\n+++ b/SKILL.md\n@@ -1,6 +1,6 @@\n ---\n name: my-skill\n description: base\n ---\n line2\n-line3\n+line3-patched\n",
    )?;

    let before = std::fs::read_to_string(&patch_path)?;

    // Upstream changes an unrelated line (frontmatter), requiring the patch to be rebased.
    std::fs::write(
        module_root.join("SKILL.md"),
        "---\nname: my-skill\ndescription: base-upstream\n---\nline2\nline3\n",
    )?;
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
    if !rebase.status.success() {
        let v = parse_stdout_json(&rebase);
        panic!("rebase should succeed, got: {v}");
    }

    let after = std::fs::read_to_string(&patch_path)?;
    assert_ne!(before, after, "expected patch file to be rewritten");

    // Verify the rebased patch produces merged content: upstream line1 + patched line3.
    let plan = agentpack_in(home, &project_dir, &["plan"]);
    assert!(plan.status.success(), "plan should succeed after rebase");

    Ok(())
}

#[test]
fn overlay_rebase_patch_conflict_writes_conflict_artifact_and_returns_error_code()
-> anyhow::Result<()> {
    let tmp = tempfile::tempdir()?;
    let home = tmp.path();

    let project_dir = home.join("project");
    std::fs::create_dir_all(&project_dir)?;

    let init = agentpack_in(home, &project_dir, &["init"]);
    assert!(init.status.success());

    let repo_dir = home.join("repo");

    let _ = git(&repo_dir, &["init"])?;
    let _ = git(&repo_dir, &["config", "user.email", "test@example.com"])?;
    let _ = git(&repo_dir, &["config", "user.name", "Test"])?;
    let _ = git(&repo_dir, &["add", "."])?;
    let _ = git(&repo_dir, &["commit", "-m", "init"])?;

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

    let edit = agentpack_in(
        home,
        &project_dir,
        &[
            "overlay",
            "edit",
            "skill_test",
            "--scope",
            "global",
            "--sparse",
        ],
    );
    assert!(edit.status.success());

    let overlay_dir = repo_dir.join("overlays").join(module_fs_key("skill_test"));
    std::fs::create_dir_all(overlay_dir.join(".agentpack/patches"))?;
    std::fs::write(
        overlay_dir.join(".agentpack/overlay.json"),
        "{\n  \"overlay_kind\": \"patch\"\n}\n",
    )?;

    // Patch edits the same line that upstream will change, forcing a conflict.
    std::fs::write(
        overlay_dir.join(".agentpack/patches/SKILL.md.patch"),
        "--- a/SKILL.md\n+++ b/SKILL.md\n@@ -1,3 +1,3 @@\n line1\n-line2\n+line2-ours\n line3\n",
    )?;

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

    let conflict_path = overlay_dir.join(".agentpack/conflicts/SKILL.md");
    assert!(
        conflict_path.exists(),
        "expected conflict artifact at {}",
        conflict_path.display()
    );

    Ok(())
}
