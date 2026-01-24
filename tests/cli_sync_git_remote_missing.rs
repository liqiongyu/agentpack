mod journeys;

use journeys::common::{TestEnv, git_ok, git_stdout, run_json_fail, run_ok};

#[test]
fn sync_returns_stable_error_code_when_remote_missing() {
    let env = TestEnv::new();

    run_ok(&env, &["--json", "--yes", "init", "--git"]);

    let repo_dir = env.repo_dir();
    git_ok(&repo_dir, &["config", "user.email", "test@example.com"]);
    git_ok(&repo_dir, &["config", "user.name", "Test User"]);

    git_ok(&repo_dir, &["add", "-A"]);
    git_ok(&repo_dir, &["commit", "-m", "chore(test): seed repo"]);

    let remotes = git_stdout(&repo_dir, &["remote"]);
    if remotes.lines().any(|line| line.trim() == "origin") {
        git_ok(&repo_dir, &["remote", "remove", "origin"]);
    }
    let remotes = git_stdout(&repo_dir, &["remote"]);
    assert!(
        !remotes.lines().any(|line| line.trim() == "origin"),
        "expected origin remote to be missing"
    );

    let status_clean = git_stdout(&repo_dir, &["status", "--porcelain"]);
    assert!(status_clean.trim().is_empty(), "expected clean repo");

    let v = run_json_fail(&env, &["sync", "--rebase", "--json", "--yes"]);
    assert_eq!(v["ok"], false);
    assert_eq!(v["command"], "sync");
    assert_eq!(v["errors"][0]["code"], "E_GIT_REMOTE_MISSING");
    assert_eq!(v["errors"][0]["details"]["command"], "sync");
    assert_eq!(v["errors"][0]["details"]["remote"], "origin");
    assert_eq!(
        v["errors"][0]["details"]["reason_code"].as_str(),
        Some("git_remote_missing")
    );
    assert_eq!(
        v["errors"][0]["details"]["next_actions"],
        serde_json::json!(["set_git_remote", "retry_command"])
    );
    assert!(v["errors"][0]["details"]["repo"].is_string());
    assert!(v["errors"][0]["details"]["repo_posix"].is_string());
    assert!(v["errors"][0]["details"]["hint"].is_string());
}
