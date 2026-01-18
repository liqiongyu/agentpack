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

#[test]
fn core_commands_do_not_read_or_parse_org_config() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let repo_dir = tmp.path().join("repo");
    std::fs::create_dir_all(&repo_dir).expect("create repo dir");

    let codex_home = tmp.path().join("codex");
    std::fs::create_dir_all(&codex_home).expect("create codex_home");

    std::fs::write(
        repo_dir.join("agentpack.yaml"),
        format!(
            r#"version: 1

profiles:
  default: {{}}

targets:
  codex:
    mode: files
    scope: user
    options:
      codex_home: '{}'
      write_repo_skills: false
      write_user_skills: false
      write_user_prompts: false
      write_agents_global: false
      write_agents_repo_root: false

modules: []
"#,
            codex_home.display()
        ),
    )
    .expect("write agentpack.yaml");

    // If a core command ever tries to read governance config by default, this should cause
    // a deterministic parse failure and break the test.
    std::fs::write(repo_dir.join("agentpack.org.yaml"), "version: [\n")
        .expect("write invalid agentpack.org.yaml");

    for args in [["plan", "--json"], ["status", "--json"]] {
        let out = agentpack_in(tmp.path(), &args);
        assert!(out.status.success(), "agentpack {args:?} should succeed");
        let v = parse_stdout_json(&out);
        assert_eq!(v["ok"], true, "agentpack {args:?} ok=true");
        assert_eq!(v["errors"].as_array().unwrap().len(), 0);
    }
}
