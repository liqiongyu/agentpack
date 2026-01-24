mod journeys;

use journeys::common::{TestEnv, git_ok, git_stdout, run_json_fail, run_ok, write_file};

#[test]
fn sync_returns_stable_error_code_when_on_detached_head() {
    let env = TestEnv::new();

    run_ok(&env, &["--json", "--yes", "init", "--git"]);

    let repo_dir = env.repo_dir();
    git_ok(&repo_dir, &["config", "user.email", "test@example.com"]);
    git_ok(&repo_dir, &["config", "user.name", "Test User"]);

    write_file(&repo_dir.join("seed.txt"), "seed\n");
    git_ok(&repo_dir, &["add", "-A"]);
    git_ok(&repo_dir, &["commit", "-m", "chore(test): seed repo"]);

    let commit = git_stdout(&repo_dir, &["rev-parse", "HEAD"])
        .trim()
        .to_string();
    assert!(!commit.is_empty());

    git_ok(&repo_dir, &["checkout", commit.as_str()]);
    let head = git_stdout(&repo_dir, &["rev-parse", "--abbrev-ref", "HEAD"]);
    assert_eq!(head.trim(), "HEAD", "expected detached HEAD");

    let status_clean = git_stdout(&repo_dir, &["status", "--porcelain"]);
    assert!(status_clean.trim().is_empty(), "expected clean repo");

    let v = run_json_fail(&env, &["sync", "--rebase", "--json", "--yes"]);
    assert_eq!(v["ok"], false);
    assert_eq!(v["command"], "sync");
    assert_eq!(v["errors"][0]["code"], "E_GIT_DETACHED_HEAD");
    assert_eq!(v["errors"][0]["details"]["command"], "sync");
    assert_eq!(
        v["errors"][0]["details"]["reason_code"].as_str(),
        Some("git_detached_head")
    );
    assert_eq!(
        v["errors"][0]["details"]["next_actions"],
        serde_json::json!(["checkout_branch", "retry_command"])
    );
    assert!(v["errors"][0]["details"]["repo"].is_string());
    assert!(v["errors"][0]["details"]["repo_posix"].is_string());
    assert!(v["errors"][0]["details"]["hint"].is_string());
}
