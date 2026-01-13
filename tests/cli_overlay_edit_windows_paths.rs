use std::path::{Path, PathBuf};
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
fn overlay_edit_uses_filesystem_safe_paths_for_long_module_ids() {
    let tmp = tempfile::tempdir().expect("tempdir");
    assert!(agentpack_in(tmp.path(), &["init"]).status.success());

    let repo_dir = tmp.path().join("repo");
    let long_suffix = "a".repeat(200);
    let module_id = format!("instructions:{long_suffix}");

    let manifest = format!(
        r#"version: 1

profiles:
  default:
    include_tags: ["base"]

targets:
  codex:
    mode: files
    scope: both
    options:
      codex_home: "~/.codex"
      write_repo_skills: false
      write_user_skills: false
      write_user_prompts: false
      write_agents_global: false
      write_agents_repo_root: false

modules:
  - id: "{module_id}"
    type: instructions
    tags: ["base"]
    source:
      local_path:
        path: "modules/instructions/base"
"#,
    );
    std::fs::write(repo_dir.join("agentpack.yaml"), manifest).expect("write manifest");

    let out = agentpack_in(
        tmp.path(),
        &[
            "overlay", "edit", &module_id, "--scope", "global", "--json", "--yes",
        ],
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let v = parse_stdout_json(&out);
    assert_eq!(v["ok"], true);
    assert_eq!(v["command"], "overlay.edit");

    let overlay_dir = v["data"]["overlay_dir"]
        .as_str()
        .expect("overlay_dir string");
    let overlay_path = PathBuf::from(overlay_dir);
    assert!(overlay_path.is_dir(), "overlay_dir exists");

    let leaf = overlay_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or_default()
        .to_string();
    assert!(
        !leaf.contains(':'),
        "overlay dir uses filesystem-safe module_fs_key"
    );
}
