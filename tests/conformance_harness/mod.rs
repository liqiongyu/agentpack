use std::path::{Path, PathBuf};
use std::process::{Command, Output};

pub(crate) struct ConformanceHarness {
    _tmp: tempfile::TempDir,
    home: PathBuf,
    workspace: PathBuf,
}

impl ConformanceHarness {
    pub(crate) fn new() -> Self {
        let tmp = tempfile::tempdir().expect("tempdir");

        let home = tmp.path().join("home");
        std::fs::create_dir_all(&home).expect("create home");

        let workspace = tmp.path().join("workspace");
        std::fs::create_dir_all(&workspace).expect("create workspace");

        assert!(git_in(&workspace, &["init"]).status.success());
        // Provide a stable origin for deterministic project_id derivation.
        let _ = git_in(
            &workspace,
            &[
                "remote",
                "add",
                "origin",
                "https://github.com/example/example.git",
            ],
        );

        Self {
            _tmp: tmp,
            home,
            workspace,
        }
    }

    pub(crate) fn home(&self) -> &Path {
        &self.home
    }

    pub(crate) fn workspace(&self) -> &Path {
        &self.workspace
    }

    pub(crate) fn agentpack(&self, args: &[&str]) -> Output {
        agentpack_in(&self.home, &self.workspace, args)
    }
}

fn agentpack_in(home: &Path, cwd: &Path, args: &[&str]) -> Output {
    let bin = env!("CARGO_BIN_EXE_agentpack");
    Command::new(bin)
        .current_dir(cwd)
        .args(args)
        .env("AGENTPACK_HOME", home)
        .env("AGENTPACK_MACHINE_ID", "test-machine")
        .env("HOME", home)
        .env("USERPROFILE", home)
        .env("XDG_CONFIG_HOME", home)
        .env("XDG_CACHE_HOME", home)
        .env("XDG_DATA_HOME", home)
        .output()
        .expect("run agentpack")
}

fn git_in(dir: &Path, args: &[&str]) -> Output {
    Command::new("git")
        .current_dir(dir)
        .args(args)
        .output()
        .expect("run git")
}
