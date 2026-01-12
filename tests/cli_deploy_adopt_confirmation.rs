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
fn deploy_json_refuses_adopt_updates_without_explicit_flag() {
    let tmp = tempfile::tempdir().expect("tempdir");

    let init = agentpack_in(tmp.path(), &["init"]);
    assert!(init.status.success());

    let codex_home = tmp.path().join("codex_home");
    std::fs::create_dir_all(codex_home.join("prompts")).expect("create prompts dir");

    let repo_dir = tmp.path().join("repo");
    let module_dir = repo_dir.join("modules/prompt_one");
    std::fs::create_dir_all(&module_dir).expect("create module dir");
    std::fs::write(module_dir.join("prompt.md"), "# new\n").expect("write prompt");

    let existing_path = codex_home.join("prompts").join("prompt.md");
    std::fs::write(&existing_path, "# old\n").expect("write existing prompt");

    let manifest = format!(
        r#"version: 1

profiles:
  default:
    include_tags: []
    include_modules: ["prompt:one"]
    exclude_modules: []

targets:
  codex:
    mode: files
    scope: user
    options:
      codex_home: '{}'
      write_agents_global: false
      write_agents_repo_root: false
      write_user_prompts: true
      write_user_skills: false
      write_repo_skills: false

modules:
  - id: "prompt:one"
    type: prompt
    enabled: true
    tags: []
    targets: ["codex"]
    source:
      local_path:
        path: "modules/prompt_one"
"#,
        codex_home.display()
    );
    let manifest_path = repo_dir.join("agentpack.yaml");
    std::fs::write(&manifest_path, manifest).expect("write manifest");

    let refused = agentpack_in(
        tmp.path(),
        &["--target", "codex", "deploy", "--apply", "--json", "--yes"],
    );
    assert!(!refused.status.success());
    let v = parse_stdout_json(&refused);
    assert_eq!(v["errors"][0]["code"], "E_ADOPT_CONFIRM_REQUIRED");
    assert_eq!(
        std::fs::read_to_string(&existing_path).expect("read existing prompt"),
        "# old\n"
    );

    let allowed = agentpack_in(
        tmp.path(),
        &[
            "--target", "codex", "deploy", "--apply", "--adopt", "--json", "--yes",
        ],
    );
    assert!(allowed.status.success());
    assert_eq!(
        std::fs::read_to_string(&existing_path).expect("read adopted prompt"),
        "# new\n"
    );
}
