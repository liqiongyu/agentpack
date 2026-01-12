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
fn doctor_warns_when_legacy_and_canonical_overlay_dirs_coexist() {
    let tmp = tempfile::tempdir().expect("tempdir");

    let init = agentpack_in(tmp.path(), &["init"]);
    assert!(init.status.success());

    let codex_home = tmp.path().join("codex_home");
    std::fs::create_dir_all(&codex_home).expect("create codex_home");

    let module_id = "instructions-base";
    let manifest = format!(
        r#"version: 1

profiles:
  default:
    include_tags: []
    include_modules: []
    exclude_modules: []

targets:
  codex:
    mode: files
    scope: user
    options:
      codex_home: '{}'
      write_agents_global: false
      write_user_prompts: false
      write_user_skills: false
      write_repo_skills: false
      write_agents_repo_root: false

modules:
  - id: "{module_id}"
    type: instructions
    enabled: true
    tags: []
    targets: ["codex"]
    source:
      local_path:
        path: "modules/instructions/base"
"#,
        codex_home.display()
    );
    let manifest_path = tmp.path().join("repo").join("agentpack.yaml");
    std::fs::write(&manifest_path, manifest).expect("write manifest");

    let repo_dir = tmp.path().join("repo");
    let canonical = repo_dir
        .join("overlays")
        .join(agentpack::ids::module_fs_key(module_id));
    let legacy = repo_dir.join("overlays").join(module_id);
    std::fs::create_dir_all(&canonical).expect("create canonical overlay dir");
    std::fs::create_dir_all(&legacy).expect("create legacy overlay dir");

    let out = agentpack_in(tmp.path(), &["--target", "codex", "doctor", "--json"]);
    assert!(out.status.success());
    let v = parse_stdout_json(&out);
    let warnings = v["warnings"].as_array().expect("warnings array");
    assert!(
        warnings.iter().any(|w| w
            .as_str()
            .unwrap_or_default()
            .contains(&format!("overlay layout (global) module {module_id}"))),
        "expected overlay layout warning"
    );
}
