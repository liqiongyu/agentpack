use std::path::PathBuf;

use agentpack::config::GitSource;
use agentpack::hash::sha256_hex;
use agentpack::paths::AgentpackHome;
use agentpack::store::{Store, sanitize_module_id};

#[test]
fn store_checkout_dir_dedups_by_url_hash_and_migrates_legacy_if_present() -> anyhow::Result<()> {
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
    let module_id_a = "instructions:base";
    let module_id_b = "prompt:base";
    let commit = "deadbeef";
    let url = "https://example.invalid/repo.git";

    let canonical = store.git_checkout_dir(url, commit);
    let hash_dir_name = canonical
        .parent()
        .and_then(|p| p.file_name())
        .expect("dir name")
        .to_string_lossy()
        .to_string();
    assert_eq!(hash_dir_name, sha256_hex(url.as_bytes()));

    // Create a legacy checkout directory (module_id-based) and ensure it migrates to canonical.
    let legacy_dir: PathBuf = home
        .cache_dir
        .join("git")
        .join(sanitize_module_id(module_id_a))
        .join(commit);
    std::fs::create_dir_all(&legacy_dir)?;

    let src = GitSource {
        url: url.to_string(),
        ref_name: "main".to_string(),
        subdir: String::new(),
        shallow: true,
    };

    let used_a = store.ensure_git_checkout(module_id_a, &src, commit)?;
    assert_eq!(used_a, canonical);

    let used_b = store.ensure_git_checkout(module_id_b, &src, commit)?;
    assert_eq!(used_b, canonical);

    Ok(())
}
