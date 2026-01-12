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

fn write_codex_prompt_manifest(repo_dir: &Path, codex_home: &Path) {
    let module_dir = repo_dir.join("modules/prompt/one");
    std::fs::create_dir_all(&module_dir).expect("create prompt module dir");
    std::fs::write(module_dir.join("prompt.md"), "# prompt\n").expect("write prompt");

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
        path: "modules/prompt/one"
"#,
        codex_home.display()
    );
    std::fs::write(repo_dir.join("agentpack.yaml"), manifest).expect("write manifest");
}

#[test]
fn evolve_restore_json_requires_yes_when_it_would_write() {
    let tmp = tempfile::tempdir().expect("tempdir");
    assert!(agentpack_in(tmp.path(), &["init"]).status.success());

    let codex_home = tmp.path().join("codex_home");
    std::fs::create_dir_all(&codex_home).expect("create codex_home");

    let repo_dir = tmp.path().join("repo");
    write_codex_prompt_manifest(&repo_dir, &codex_home);

    let out = agentpack_in(
        tmp.path(),
        &["--target", "codex", "evolve", "restore", "--json"],
    );
    assert!(!out.status.success());
    let v = parse_stdout_json(&out);
    assert_eq!(v["ok"], false);
    assert_eq!(v["errors"][0]["code"], "E_CONFIRM_REQUIRED");
}

#[test]
fn evolve_restore_restores_missing_desired_outputs() {
    let tmp = tempfile::tempdir().expect("tempdir");
    assert!(agentpack_in(tmp.path(), &["init"]).status.success());

    let codex_home = tmp.path().join("codex_home");
    std::fs::create_dir_all(&codex_home).expect("create codex_home");

    let repo_dir = tmp.path().join("repo");
    write_codex_prompt_manifest(&repo_dir, &codex_home);

    let prompt_path = codex_home.join("prompts").join("prompt.md");
    assert!(!prompt_path.exists());

    let out = agentpack_in(
        tmp.path(),
        &["--target", "codex", "evolve", "restore", "--json", "--yes"],
    );
    assert!(out.status.success());
    let v = parse_stdout_json(&out);
    assert_eq!(v["ok"], true);
    assert_eq!(v["command"], "evolve.restore");

    assert!(prompt_path.is_file());
    assert_eq!(
        std::fs::read_to_string(&prompt_path).expect("read prompt"),
        "# prompt\n"
    );
}
