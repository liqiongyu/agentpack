use std::path::{Path, PathBuf};
use std::process::Command;

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

fn write_manifest(repo_dir: &Path, codex_home: &Path) -> anyhow::Result<()> {
    let manifest = format!(
        r#"version: 1

profiles:
  default:
    include_tags: ["base"]

targets:
  codex:
    mode: files
    scope: user
    options:
      codex_home: '{codex_home}'
      write_agents_global: true
      write_agents_repo_root: false
      write_user_prompts: true
      write_user_skills: false
      write_repo_skills: false

modules:
  - id: instructions:base
    type: instructions
    enabled: true
    tags: ["base"]
    targets: ["codex"]
    source:
      local_path:
        path: modules/instructions/base

  - id: prompt:hello
    type: prompt
    enabled: true
    tags: ["base"]
    targets: ["codex"]
    source:
      local_path:
        path: modules/prompts/hello
"#,
        codex_home = codex_home.display()
    );
    std::fs::write(repo_dir.join("agentpack.yaml"), manifest)
        .map_err(|e| anyhow::anyhow!("write manifest: {e}"))?;
    Ok(())
}

fn write_module(
    repo_dir: &Path,
    rel_dir: &str,
    filename: &str,
    content: &str,
) -> anyhow::Result<()> {
    let dir = repo_dir.join(rel_dir);
    std::fs::create_dir_all(&dir).map_err(|e| anyhow::anyhow!("create module dir: {e}"))?;
    std::fs::write(dir.join(filename), content)
        .map_err(|e| anyhow::anyhow!("write module: {e}"))?;
    Ok(())
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

#[test]
fn status_only_filters_drift_and_includes_summary_total() -> anyhow::Result<()> {
    let tmp = tempfile::tempdir().expect("tempdir");
    let home = tmp.path();
    let workspace = init_workspace(&tmp)?;

    let init = agentpack_in(home, &workspace, &["init"]);
    assert!(init.status.success());

    let repo_dir = home.join("repo");
    let codex_home = home.join("codex_home");
    std::fs::create_dir_all(&codex_home)?;

    write_module(
        &repo_dir,
        "modules/instructions/base",
        "AGENTS.md",
        "# Base instructions\n",
    )?;
    write_module(&repo_dir, "modules/prompts/hello", "hello.md", "Hello v1\n")?;
    write_manifest(&repo_dir, &codex_home)?;

    let deploy = agentpack_in(
        home,
        &workspace,
        &["--target", "codex", "deploy", "--apply", "--yes", "--json"],
    );
    assert!(deploy.status.success());

    // Create missing + modified + extra drift.
    let _ = std::fs::remove_file(codex_home.join("AGENTS.md"));
    std::fs::write(codex_home.join("prompts").join("hello.md"), "local drift\n")?;
    std::fs::write(codex_home.join("prompts").join("extra.txt"), "extra\n")?;

    let status = agentpack_in(
        home,
        &workspace,
        &["--target", "codex", "status", "--only", "missing", "--json"],
    );
    assert!(status.status.success());

    let v = parse_stdout_json(&status);
    assert_eq!(v["command"], "status");
    assert!(v["ok"].as_bool().unwrap_or(false));

    let summary = &v["data"]["summary"];
    assert_eq!(summary["modified"].as_u64().unwrap_or_default(), 0);
    assert_eq!(summary["missing"].as_u64().unwrap_or_default(), 1);
    assert_eq!(summary["extra"].as_u64().unwrap_or_default(), 0);

    let total = &v["data"]["summary_total"];
    assert_eq!(total["modified"].as_u64().unwrap_or_default(), 1);
    assert_eq!(total["missing"].as_u64().unwrap_or_default(), 1);
    assert_eq!(total["extra"].as_u64().unwrap_or_default(), 1);

    let drift = v["data"]["drift"].as_array().expect("drift is array");
    assert!(!drift.is_empty());
    for item in drift {
        assert_eq!(item["kind"], "missing");
    }

    Ok(())
}
