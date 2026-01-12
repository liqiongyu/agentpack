use std::collections::BTreeMap;

use agentpack::config::{LocalPathSource, Manifest, Module, ModuleType, Profile, Source};
use agentpack::lockfile::generate_lockfile;
use agentpack::paths::{AgentpackHome, RepoPaths};
use agentpack::store::Store;

#[test]
fn lockfile_stores_local_path_repo_relative() -> anyhow::Result<()> {
    let tmp = tempfile::tempdir().expect("tempdir");

    let repo_dir = tmp.path().join("repo");
    std::fs::create_dir_all(repo_dir.join("modules/prompts")).expect("create dirs");
    std::fs::write(repo_dir.join("modules/prompts/test.md"), "hello\n").expect("write file");

    let mut profiles = BTreeMap::new();
    profiles.insert(
        "default".to_string(),
        Profile {
            include_tags: Vec::new(),
            include_modules: Vec::new(),
            exclude_modules: Vec::new(),
        },
    );

    let manifest = Manifest {
        version: 1,
        profiles,
        targets: BTreeMap::new(),
        modules: vec![Module {
            id: "prompt:test".to_string(),
            module_type: ModuleType::Prompt,
            enabled: true,
            tags: Vec::new(),
            targets: Vec::new(),
            source: Source {
                local_path: Some(LocalPathSource {
                    path: "modules/prompts/test.md".to_string(),
                }),
                git: None,
            },
            metadata: BTreeMap::new(),
        }],
    };

    let repo = RepoPaths {
        repo_dir: repo_dir.clone(),
        manifest_path: repo_dir.join("agentpack.yaml"),
        lockfile_path: repo_dir.join("agentpack.lock.json"),
    };

    let home_root = tmp.path().join("home");
    let state_dir = home_root.join("state");
    let home = AgentpackHome {
        root: home_root.clone(),
        repo_dir: home_root.join("repo"),
        state_dir: state_dir.clone(),
        cache_dir: home_root.join("cache"),
        snapshots_dir: state_dir.join("snapshots"),
        logs_dir: state_dir.join("logs"),
    };
    let store = Store::new(&home);

    let lock = generate_lockfile(&repo, &manifest, &store)?;
    assert_eq!(lock.modules.len(), 1);

    let stored = lock.modules[0]
        .resolved_source
        .local_path
        .as_ref()
        .expect("local_path populated")
        .path
        .as_str();
    assert_eq!(stored, "modules/prompts/test.md");
    assert!(!stored.contains('\\'), "stored path should be POSIX style");

    let tmp_str = tmp.path().to_string_lossy();
    assert!(
        !stored.contains(tmp_str.as_ref()),
        "stored path should not embed the machine absolute path"
    );

    Ok(())
}
