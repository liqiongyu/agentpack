use std::path::{Path, PathBuf};
use std::process::Command;

fn git_init(dir: &Path) {
    assert!(
        Command::new("git")
            .current_dir(dir)
            .args(["init"])
            .output()
            .expect("git init")
            .status
            .success()
    );
}

fn example_repo_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/examples/minimal_repo")
}

#[test]
#[cfg(feature = "target-codex")]
fn minimal_repo_example_plan_succeeds_for_codex() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let home = tmp.path();

    let workspace = home.join("workspace");
    std::fs::create_dir_all(&workspace).expect("create workspace");
    git_init(&workspace);

    let repo = example_repo_dir();
    assert!(repo.join("agentpack.yaml").exists());

    let bin = env!("CARGO_BIN_EXE_agentpack");
    let out = Command::new(bin)
        .current_dir(&workspace)
        .args([
            "--repo",
            repo.to_string_lossy().as_ref(),
            "--target",
            "codex",
            "plan",
        ])
        .env("AGENTPACK_HOME", home)
        .output()
        .expect("run agentpack");

    assert!(
        out.status.success(),
        "agentpack plan failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}
