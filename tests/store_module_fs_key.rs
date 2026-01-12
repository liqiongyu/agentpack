use std::path::PathBuf;

use agentpack::config::GitSource;
use agentpack::paths::AgentpackHome;
use agentpack::store::{Store, sanitize_module_id};

#[test]
fn store_checkout_dir_uses_fs_key_and_falls_back_to_legacy_if_present() -> anyhow::Result<()> {
    let tmp = tempfile::tempdir()?;
    let root = tmp.path().to_path_buf();

    let home = AgentpackHome {
        repo_dir: root.join("repo"),
        state_dir: root.join("state"),
        cache_dir: root.join("cache"),
        snapshots_dir: root.join("state").join("snapshots"),
        logs_dir: root.join("state").join("logs"),
        root: root.clone(),
    };

    let store = Store::new(&home);
    let module_id = "instructions:base";
    let commit = "deadbeef";

    let canonical = store.git_checkout_dir(module_id, commit);
    let dir_name = canonical
        .parent()
        .and_then(|p| p.file_name())
        .expect("dir name")
        .to_string_lossy()
        .to_string();
    assert!(dir_name.starts_with("instructions_base--"));

    // Create legacy checkout directory and ensure ensure_git_checkout returns it (without cloning).
    let legacy_dir: PathBuf = home
        .cache_dir
        .join("git")
        .join(sanitize_module_id(module_id))
        .join(commit);
    std::fs::create_dir_all(&legacy_dir)?;

    let src = GitSource {
        url: "https://example.invalid/repo.git".to_string(),
        ref_name: "main".to_string(),
        subdir: String::new(),
        shallow: true,
    };

    let used = store.ensure_git_checkout(module_id, &src, commit)?;
    assert_eq!(used, legacy_dir);

    Ok(())
}
