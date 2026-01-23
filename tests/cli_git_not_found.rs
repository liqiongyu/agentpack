mod journeys;

use journeys::common::{TestEnv, parse_stdout_json, run_ok};

#[test]
fn stable_error_code_when_git_executable_missing() {
    let env = TestEnv::new();

    run_ok(&env, &["--json", "--yes", "init", "--git"]);

    let mut cmd = env.agentpack();
    cmd.env("PATH", "");
    cmd.args(["sync", "--rebase", "--json", "--yes"]);

    let out = cmd.output().expect("run agentpack");
    assert!(!out.status.success(), "expected command to fail");

    let v = parse_stdout_json(&out);
    assert_eq!(v["ok"], false);
    assert_eq!(v["command"], "sync");
    assert_eq!(v["errors"][0]["code"], "E_GIT_NOT_FOUND");
    assert!(v["errors"][0]["details"]["cwd"].is_string());
    assert!(v["errors"][0]["details"]["cwd_posix"].is_string());
    assert!(v["errors"][0]["details"]["args"].is_array());
    assert!(v["errors"][0]["details"]["hint"].is_string());
}
