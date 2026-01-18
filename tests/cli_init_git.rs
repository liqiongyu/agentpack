use std::process::Command;

#[test]
fn init_git_initializes_repo_and_writes_gitignore() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let bin = env!("CARGO_BIN_EXE_agentpack");

    let out = Command::new(bin)
        .args(["init", "--git"])
        .env("AGENTPACK_HOME", tmp.path())
        .output()
        .expect("run agentpack");
    assert!(out.status.success());

    let repo = tmp.path().join("repo");
    assert!(repo.join(".git").exists());

    let gitignore = std::fs::read_to_string(repo.join(".gitignore")).expect("read .gitignore");
    assert!(
        gitignore
            .lines()
            .any(|l| l.trim() == ".agentpack.manifest*.json")
    );
}
