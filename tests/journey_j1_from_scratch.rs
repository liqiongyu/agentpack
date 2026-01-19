mod journeys;

use std::process::Output;

use journeys::common::TestEnv;

fn run_ok(env: &TestEnv, args: &[&str]) -> Output {
    let out = env.agentpack().args(args).output().expect("run agentpack");
    assert!(
        out.status.success(),
        "command failed: agentpack {}\nstdout:\n{}\nstderr:\n{}",
        args.join(" "),
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );
    out
}

fn parse_json(out: &Output) -> serde_json::Value {
    serde_json::from_slice(&out.stdout).expect("parse json stdout")
}

#[test]
fn journey_j1_from_scratch_deploy_flow() {
    let env = TestEnv::new();

    // init (and seed minimal modules)
    env.init_repo_with_base_modules();

    // update
    run_ok(&env, &["--json", "--yes", "update"]);

    // preview --diff
    let preview = parse_json(&run_ok(&env, &["--json", "preview", "--diff"]));
    assert!(preview["ok"].as_bool().unwrap_or(false));
    assert!(preview["data"]["plan"]["summary"].is_object());
    assert!(preview["data"]["diff"]["summary"].is_object());

    // deploy --apply
    let deploy = parse_json(&run_ok(&env, &["--json", "--yes", "deploy", "--apply"]));
    assert!(deploy["ok"].as_bool().unwrap_or(false));
    assert!(deploy["data"]["applied"].as_bool().unwrap_or(false));
    let snapshot_id = deploy["data"]["snapshot_id"]
        .as_str()
        .expect("deploy snapshot_id")
        .to_string();
    assert!(!snapshot_id.trim().is_empty());

    // status
    let status = parse_json(&run_ok(&env, &["--json", "status"]));
    assert!(status["ok"].as_bool().unwrap_or(false));
    assert_eq!(status["data"]["summary"]["modified"].as_u64(), Some(0));
    assert_eq!(status["data"]["summary"]["missing"].as_u64(), Some(0));
    assert_eq!(status["data"]["summary"]["extra"].as_u64(), Some(0));

    // rollback
    let rollback = parse_json(&run_ok(
        &env,
        &["--json", "--yes", "rollback", "--to", &snapshot_id],
    ));
    assert!(rollback["ok"].as_bool().unwrap_or(false));
    assert_eq!(
        rollback["data"]["rolled_back_to"].as_str(),
        Some(snapshot_id.as_str())
    );
}
