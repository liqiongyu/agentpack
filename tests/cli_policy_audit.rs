use std::path::Path;
use std::process::Command;

fn agentpack_in(home: &Path, args: &[&str]) -> std::process::Output {
    let bin = env!("CARGO_BIN_EXE_agentpack");
    Command::new(bin)
        .args(args)
        .env("AGENTPACK_HOME", home)
        .output()
        .expect("run agentpack")
}

fn parse_stdout_json(output: &std::process::Output) -> serde_json::Value {
    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(&stdout).expect("stdout is valid json")
}

fn git_in(cwd: &Path, args: &[&str]) -> anyhow::Result<String> {
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
fn policy_audit_json_errors_when_lockfile_missing() {
    let tmp = tempfile::tempdir().expect("tempdir");
    assert!(agentpack_in(tmp.path(), &["init"]).status.success());

    let out = agentpack_in(tmp.path(), &["policy", "audit", "--json"]);
    assert!(!out.status.success());
    let v = parse_stdout_json(&out);
    assert_eq!(v["errors"][0]["code"], "E_LOCKFILE_MISSING");
}

#[test]
fn policy_audit_emits_report_and_includes_optional_policy_pack_lock() {
    let tmp = tempfile::tempdir().expect("tempdir");
    assert!(agentpack_in(tmp.path(), &["init"]).status.success());

    // Add at least one module so the lockfile includes modules[].
    assert!(
        agentpack_in(
            tmp.path(),
            &[
                "add",
                "instructions",
                "local:modules/instructions/base",
                "--id",
                "instructions:base",
                "--tags",
                "base",
            ],
        )
        .status
        .success()
    );
    assert!(agentpack_in(tmp.path(), &["lock"]).status.success());

    // Create a local policy pack and lock it so audit can include org_policy_pack.
    let repo = tmp.path().join("repo");
    std::fs::create_dir_all(repo.join("policies/my-pack")).expect("mkdir pack");
    std::fs::write(repo.join("policies/my-pack/rule.txt"), "hello").expect("write pack file");
    std::fs::write(
        repo.join("agentpack.org.yaml"),
        r#"version: 1

policy_pack:
  source: "local:policies/my-pack"
"#,
    )
    .expect("write org config");
    assert!(
        agentpack_in(tmp.path(), &["policy", "lock"])
            .status
            .success()
    );

    let out = agentpack_in(tmp.path(), &["policy", "audit", "--json"]);
    assert!(out.status.success());
    let v = parse_stdout_json(&out);
    assert_eq!(v["ok"], true);
    assert_eq!(v["command"], "policy.audit");
    assert!(v["data"]["modules"].as_array().is_some());
    assert!(!v["data"]["modules"].as_array().unwrap().is_empty());
    assert!(v["data"]["org_policy_pack"].is_object());
    assert_eq!(v["data"]["org_policy_pack"]["resolved_version"], "local");
}

#[test]
fn policy_audit_includes_change_summary_when_git_history_available() -> anyhow::Result<()> {
    let tmp = tempfile::tempdir().expect("tempdir");
    assert!(agentpack_in(tmp.path(), &["init"]).status.success());

    assert!(
        agentpack_in(
            tmp.path(),
            &[
                "add",
                "instructions",
                "local:modules/instructions/base",
                "--id",
                "instructions:base",
                "--tags",
                "base",
            ],
        )
        .status
        .success()
    );
    assert!(agentpack_in(tmp.path(), &["lock"]).status.success());

    let repo = tmp.path().join("repo");
    git_in(&repo, &["init"])?;
    git_in(&repo, &["config", "user.email", "test@example.com"])?;
    git_in(&repo, &["config", "user.name", "Test"])?;
    git_in(&repo, &["add", "."])?;
    git_in(&repo, &["commit", "-m", "init"])?;

    // Modify module contents and regenerate the lockfile so audit can diff lockfiles.
    let agents = repo.join("modules/instructions/base/AGENTS.md");
    std::fs::write(
        &agents,
        format!("{}\nupdated\n", std::fs::read_to_string(&agents)?),
    )?;
    assert!(agentpack_in(tmp.path(), &["lock"]).status.success());
    git_in(
        &repo,
        &[
            "add",
            "agentpack.lock.json",
            "modules/instructions/base/AGENTS.md",
        ],
    )?;
    git_in(&repo, &["commit", "-m", "update lockfile"])?;

    let out = agentpack_in(tmp.path(), &["policy", "audit", "--json"]);
    assert!(out.status.success());
    let v = parse_stdout_json(&out);
    assert!(v["data"]["change_summary"].is_object());
    assert_eq!(v["data"]["change_summary"]["base_ref"], "HEAD^");
    let changed = v["data"]["change_summary"]["modules_changed"]
        .as_array()
        .expect("modules_changed array");
    assert!(changed.iter().any(|c| c["id"] == "instructions:base"));

    Ok(())
}
