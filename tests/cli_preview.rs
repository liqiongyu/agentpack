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
fn preview_json_contains_plan_and_optional_diff() {
    let tmp = tempfile::tempdir().expect("tempdir");

    let init = agentpack_in(tmp.path(), &["init"]);
    assert!(init.status.success());

    let codex_home = tmp.path().join("codex_home");
    std::fs::create_dir_all(&codex_home).expect("create codex_home");

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
      codex_home: '{}'
      write_agents_global: true
      write_user_prompts: false
      write_user_skills: false

modules: []
"#,
        codex_home.display()
    );
    let manifest_path = tmp.path().join("repo").join("agentpack.yaml");
    std::fs::write(&manifest_path, manifest).expect("write manifest");

    let preview = agentpack_in(tmp.path(), &["--target", "codex", "preview", "--json"]);
    assert!(preview.status.success());
    let v = parse_stdout_json(&preview);
    assert_eq!(v["ok"], true);
    assert_eq!(v["command"], "preview");
    assert!(v["data"]["plan"]["summary"].is_object());

    let preview_diff = agentpack_in(
        tmp.path(),
        &["--target", "codex", "preview", "--diff", "--json"],
    );
    assert!(preview_diff.status.success());
    let v = parse_stdout_json(&preview_diff);
    assert_eq!(v["ok"], true);
    assert_eq!(v["command"], "preview");
    assert!(v["data"]["plan"]["summary"].is_object());
    assert!(v["data"]["diff"]["summary"].is_object());
}
