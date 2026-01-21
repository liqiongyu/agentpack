mod journeys;

use journeys::common::{
    TestEnv, git_clone_branch, git_ok, git_stdout, read_text_normalized, run_json_ok, write_file,
};

#[test]
fn journey_j6_multi_machine_sync_bare_remote_rebase() {
    let td = tempfile::tempdir().expect("tempdir");
    let remote_dir = td.path().join("remote.git");

    let remote_dir_str = remote_dir.to_string_lossy();
    git_ok(td.path(), &["init", "--bare", remote_dir_str.as_ref()]);

    // Machine A: init config repo, commit baseline, and push to a shared bare remote.
    let env_a = TestEnv::new();
    env_a.init_repo_with_base_modules();
    let repo_a = env_a.repo_dir();
    write_file(&repo_a.join("sync-a.txt"), "a0\n");
    write_file(&repo_a.join("sync-b.txt"), "b0\n");

    git_ok(&repo_a, &["init"]);
    git_ok(&repo_a, &["config", "user.email", "test@example.com"]);
    git_ok(&repo_a, &["config", "user.name", "Machine A"]);
    git_ok(&repo_a, &["add", "."]);
    git_ok(&repo_a, &["commit", "-m", "baseline"]);
    let branch = git_stdout(&repo_a, &["rev-parse", "--abbrev-ref", "HEAD"]);
    let branch = branch.trim();

    git_ok(
        &repo_a,
        &["remote", "add", "origin", remote_dir_str.as_ref()],
    );
    git_ok(&repo_a, &["push", "-u", "origin", branch]);

    // Machine B: clone the config repo from the shared bare remote.
    let env_b = TestEnv::new();
    let repo_b = env_b.repo_dir();
    git_clone_branch(&remote_dir, branch, &repo_b);
    git_ok(&repo_b, &["config", "user.email", "test@example.com"]);
    git_ok(&repo_b, &["config", "user.name", "Machine B"]);

    // Remote advances on machine A.
    write_file(&repo_a.join("sync-a.txt"), "a1\n");
    git_ok(&repo_a, &["add", "sync-a.txt"]);
    git_ok(&repo_a, &["commit", "-m", "machine-a change"]);
    git_ok(&repo_a, &["push", "origin", branch]);

    // Machine B creates a divergent commit, then syncs with rebase.
    write_file(&repo_b.join("sync-b.txt"), "b1\n");
    git_ok(&repo_b, &["add", "sync-b.txt"]);
    git_ok(&repo_b, &["commit", "-m", "machine-b change"]);

    let sync_b = run_json_ok(
        &env_b,
        &["--json", "--yes", "sync", "--rebase", "--remote", "origin"],
    );
    assert_eq!(sync_b["ok"].as_bool(), Some(true));
    assert_eq!(sync_b["data"]["remote"].as_str(), Some("origin"));
    assert_eq!(sync_b["data"]["rebase"].as_bool(), Some(true));
    assert_eq!(sync_b["data"]["branch"].as_str(), Some(branch));
    let commands = sync_b["data"]["commands"]
        .as_array()
        .expect("commands array");
    assert!(
        commands
            .iter()
            .any(|c| c.as_str() == Some(&format!("git pull --rebase origin {branch}"))),
        "expected commands to include pull --rebase; got {commands:?}"
    );
    assert!(
        commands
            .iter()
            .any(|c| c.as_str() == Some(&format!("git push origin {branch}"))),
        "expected commands to include push; got {commands:?}"
    );

    assert_eq!(read_text_normalized(&repo_b.join("sync-a.txt")), "a1\n");
    assert_eq!(read_text_normalized(&repo_b.join("sync-b.txt")), "b1\n");

    // Machine A syncs and converges to the same content.
    let sync_a = run_json_ok(
        &env_a,
        &["--json", "--yes", "sync", "--rebase", "--remote", "origin"],
    );
    assert_eq!(sync_a["ok"].as_bool(), Some(true));
    assert_eq!(sync_a["data"]["remote"].as_str(), Some("origin"));
    assert_eq!(sync_a["data"]["rebase"].as_bool(), Some(true));
    assert_eq!(sync_a["data"]["branch"].as_str(), Some(branch));

    assert_eq!(read_text_normalized(&repo_a.join("sync-a.txt")), "a1\n");
    assert_eq!(read_text_normalized(&repo_a.join("sync-b.txt")), "b1\n");
}
