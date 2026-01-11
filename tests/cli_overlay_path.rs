use std::path::{Path, PathBuf};
use std::process::Command;

use sha2::Digest as _;

fn agentpack_in(home: &Path, cwd: &Path, args: &[&str]) -> std::process::Output {
    let bin = env!("CARGO_BIN_EXE_agentpack");
    Command::new(bin)
        .current_dir(cwd)
        .args(args)
        .env("AGENTPACK_HOME", home)
        .output()
        .expect("run agentpack")
}

fn parse_stdout_json(output: &std::process::Output) -> serde_json::Value {
    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(&stdout).expect("stdout is valid json")
}

fn compute_project_id(project_root: &Path) -> String {
    let basis = project_root
        .canonicalize()
        .unwrap_or_else(|_| project_root.to_path_buf())
        .to_string_lossy()
        .to_string();
    let mut hasher = sha2::Sha256::new();
    hasher.update(basis.as_bytes());
    let hex = hex::encode(hasher.finalize());
    hex.chars().take(16).collect()
}

#[test]
fn overlay_path_resolves_global_machine_and_project_scopes() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let home = tmp.path();

    let project_dir = home.join("project");
    std::fs::create_dir_all(&project_dir).expect("create project dir");

    let init = agentpack_in(home, &project_dir, &["init"]);
    assert!(init.status.success());

    let doctor = agentpack_in(
        home,
        &project_dir,
        &["--target", "codex", "doctor", "--json"],
    );
    assert!(doctor.status.success());
    let doctor_json = parse_stdout_json(&doctor);
    let machine_id = doctor_json["data"]["machine_id"]
        .as_str()
        .expect("machine_id")
        .to_string();

    let module_id = "skill_test";
    let repo_dir = home.join("repo");

    let global = agentpack_in(
        home,
        &project_dir,
        &["overlay", "path", module_id, "--scope", "global", "--json"],
    );
    assert!(global.status.success());
    let global_json = parse_stdout_json(&global);
    let global_dir = global_json["data"]["overlay_dir"]
        .as_str()
        .expect("overlay_dir");
    assert_eq!(
        PathBuf::from(global_dir),
        repo_dir.join("overlays").join(module_id)
    );

    let machine = agentpack_in(
        home,
        &project_dir,
        &["overlay", "path", module_id, "--scope", "machine", "--json"],
    );
    assert!(machine.status.success());
    let machine_json = parse_stdout_json(&machine);
    let machine_dir = machine_json["data"]["overlay_dir"]
        .as_str()
        .expect("overlay_dir");
    assert_eq!(
        PathBuf::from(machine_dir),
        repo_dir
            .join("overlays/machines")
            .join(&machine_id)
            .join(module_id)
    );

    let project = agentpack_in(
        home,
        &project_dir,
        &["overlay", "path", module_id, "--scope", "project", "--json"],
    );
    assert!(project.status.success());
    let project_json = parse_stdout_json(&project);
    let project_dir_out = project_json["data"]["overlay_dir"]
        .as_str()
        .expect("overlay_dir");
    let project_id = compute_project_id(&project_dir);
    assert_eq!(
        PathBuf::from(project_dir_out),
        repo_dir
            .join("projects")
            .join(project_id)
            .join("overlays")
            .join(module_id)
    );
}
