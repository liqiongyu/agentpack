mod journeys;

use journeys::common::{TestEnv, run_json_fail, run_ok};

#[test]
fn sync_returns_stable_error_code_when_config_repo_is_not_git() {
    let env = TestEnv::new();
    run_ok(&env, &["--json", "--yes", "init"]);

    let repo_dir = env.repo_dir();
    assert!(
        !repo_dir.join(".git").exists(),
        "expected config repo to not be a git repository"
    );

    let v = run_json_fail(&env, &["sync", "--rebase", "--json", "--yes"]);
    assert_eq!(v["ok"], false);
    assert_eq!(v["command"], "sync");
    assert_eq!(v["errors"][0]["code"], "E_GIT_REPO_REQUIRED");
    assert_eq!(v["errors"][0]["details"]["command"], "sync");
    assert!(v["errors"][0]["details"]["repo"].is_string());
    assert!(v["errors"][0]["details"]["repo_posix"].is_string());
    assert!(v["errors"][0]["details"]["hint"].is_string());
}
