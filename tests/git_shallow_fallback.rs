use std::path::Path;
use std::process::Command;

fn git_in(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .current_dir(dir)
        .args(args)
        .output()
        .expect("run git");
    assert!(
        out.status.success(),
        "git {:?} failed: {}",
        args,
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

#[test]
fn clone_checkout_git_retries_non_shallow_when_checkout_fails() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let work = tmp.path().join("work");
    let remote = tmp.path().join("remote.git");
    std::fs::create_dir_all(&work).expect("create work dir");

    git_in(&work, &["init", "-b", "main"]);
    git_in(&work, &["config", "user.email", "test@example.com"]);
    git_in(&work, &["config", "user.name", "Test"]);

    std::fs::write(work.join("file.txt"), "one\n").expect("write file");
    git_in(&work, &["add", "."]);
    git_in(&work, &["commit", "-m", "c1"]);
    let first_commit = git_in(&work, &["rev-parse", "HEAD"]);

    std::fs::write(work.join("file.txt"), "two\n").expect("write file");
    git_in(&work, &["add", "."]);
    git_in(&work, &["commit", "-m", "c2"]);

    git_in(
        tmp.path(),
        &["init", "--bare", remote.to_string_lossy().as_ref()],
    );
    git_in(
        &work,
        &["remote", "add", "origin", remote.to_string_lossy().as_ref()],
    );
    git_in(&work, &["push", "-u", "origin", "main"]);

    let url = format!("file://{}", remote.to_string_lossy());
    let dest = tmp.path().join("checkout");
    agentpack::git::clone_checkout_git(&url, "main", &first_commit, &dest, true)
        .expect("clone_checkout_git");

    let head = git_in(&dest, &["rev-parse", "HEAD"]);
    assert_eq!(head, first_commit);
}
