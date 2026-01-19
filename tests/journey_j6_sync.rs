mod journeys;

use std::path::Path;
use std::process::Output;

use journeys::common::TestEnv;

fn write_file(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("create parent dirs");
    }
    std::fs::write(path, contents).expect("write file");
}

fn run_out(env: &TestEnv, args: &[&str]) -> Output {
    env.agentpack().args(args).output().expect("run agentpack")
}

fn run_ok(env: &TestEnv, args: &[&str]) -> Output {
    let out = run_out(env, args);
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

fn git_out(dir: &Path, args: &[&str]) -> Output {
    std::process::Command::new("git")
        .current_dir(dir)
        .args(args)
        .output()
        .expect("run git")
}

fn git_ok(dir: &Path, args: &[&str]) {
    let out = git_out(dir, args);
    assert!(
        out.status.success(),
        "git command failed: git {}\nstdout:\n{}\nstderr:\n{}",
        args.join(" "),
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );
}

fn git_stdout(dir: &Path, args: &[&str]) -> String {
    let out = git_out(dir, args);
    assert!(
        out.status.success(),
        "git command failed: git {}\nstdout:\n{}\nstderr:\n{}",
        args.join(" "),
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );
    String::from_utf8(out.stdout).expect("git stdout utf8")
}

fn git_clone_branch(remote: &Path, branch: &str, dst: &Path) {
    let remote = remote.to_string_lossy().to_string();
    let dst = dst.to_string_lossy().to_string();
    let out = std::process::Command::new("git")
        .args(["clone", "--branch", branch, &remote, &dst])
        .output()
        .expect("git clone");
    assert!(
        out.status.success(),
        "git clone failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );
}

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

    let sync_b = parse_json(&run_ok(
        &env_b,
        &["--json", "--yes", "sync", "--rebase", "--remote", "origin"],
    ));
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

    assert_eq!(
        std::fs::read_to_string(repo_b.join("sync-a.txt")).expect("read sync-a.txt"),
        "a1\n"
    );
    assert_eq!(
        std::fs::read_to_string(repo_b.join("sync-b.txt")).expect("read sync-b.txt"),
        "b1\n"
    );

    // Machine A syncs and converges to the same content.
    let sync_a = parse_json(&run_ok(
        &env_a,
        &["--json", "--yes", "sync", "--rebase", "--remote", "origin"],
    ));
    assert_eq!(sync_a["ok"].as_bool(), Some(true));
    assert_eq!(sync_a["data"]["remote"].as_str(), Some("origin"));
    assert_eq!(sync_a["data"]["rebase"].as_bool(), Some(true));
    assert_eq!(sync_a["data"]["branch"].as_str(), Some(branch));

    assert_eq!(
        std::fs::read_to_string(repo_a.join("sync-a.txt")).expect("read sync-a.txt"),
        "a1\n"
    );
    assert_eq!(
        std::fs::read_to_string(repo_a.join("sync-b.txt")).expect("read sync-b.txt"),
        "b1\n"
    );
}
