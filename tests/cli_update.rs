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
fn json_mode_requires_yes_for_update() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let output = agentpack_in(tmp.path(), &["update", "--json"]);
    assert!(!output.status.success());

    let v = parse_stdout_json(&output);
    assert_eq!(v["ok"], false);
    assert_eq!(v["command"], "update");
    assert_eq!(v["errors"][0]["code"], "E_CONFIRM_REQUIRED");
}

#[test]
fn update_runs_lock_then_fetch_when_lockfile_is_missing_and_fetch_only_when_present() {
    let tmp = tempfile::tempdir().expect("tempdir");

    let init = agentpack_in(tmp.path(), &["init"]);
    assert!(init.status.success());

    let first = agentpack_in(tmp.path(), &["update", "--json", "--yes"]);
    assert!(first.status.success());
    let first_json = parse_stdout_json(&first);
    let first_steps = first_json["data"]["steps"].as_array().expect("steps array");
    assert_eq!(first_steps.len(), 2);
    assert_eq!(first_steps[0]["name"], "lock");
    assert_eq!(first_steps[1]["name"], "fetch");

    let lockfile = tmp.path().join("repo").join("agentpack.lock.json");
    let first_lockfile = std::fs::read_to_string(&lockfile).expect("read lockfile");

    let second = agentpack_in(tmp.path(), &["update", "--json", "--yes"]);
    assert!(second.status.success());
    let second_json = parse_stdout_json(&second);
    let second_steps = second_json["data"]["steps"]
        .as_array()
        .expect("steps array");
    assert_eq!(second_steps.len(), 1);
    assert_eq!(second_steps[0]["name"], "fetch");

    let second_lockfile = std::fs::read_to_string(&lockfile).expect("read lockfile");
    assert_eq!(second_lockfile, first_lockfile);
}
