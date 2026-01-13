use std::fs;
use std::path::Path;

use agentpack::config::ModuleType;
use agentpack::deploy;
use agentpack::deploy::DesiredFile;
use agentpack::deploy::TargetPath;
use agentpack::fs::copy_tree;
use agentpack::lockfile::hash_tree;
use agentpack::lockfile::{LockedModule, Lockfile, ResolvedGitSource, ResolvedSource};
use agentpack::overlay::compose_module_tree;
use agentpack::overlay::ensure_overlay_skeleton;
use agentpack::overlay::ensure_overlay_skeleton_sparse;
use agentpack::overlay::materialize_overlay_from_upstream;
use agentpack::overlay::resolve_upstream_module_root;
use agentpack::paths::AgentpackHome;
use agentpack::paths::RepoPaths;
use agentpack::source::parse_source_spec;
use agentpack::state::DeploymentSnapshot;
use agentpack::target_manifest::{ManagedManifestFile, TargetManifest, manifest_path};
use agentpack::targets::TargetRoot;
use agentpack::validate::validate_materialized_module;

#[test]
fn parse_source_spec_parses_local_and_git() -> anyhow::Result<()> {
    let s = parse_source_spec("local:modules/instructions/base")?;
    assert_eq!(
        s.local_path.as_ref().expect("local_path").path,
        "modules/instructions/base"
    );
    assert!(s.git.is_none());

    let s = parse_source_spec(
        "git:https://example.com/repo.git#ref=v1.2.3&subdir=skills/foo&shallow=false",
    )?;
    let git = s.git.as_ref().expect("git");
    assert_eq!(git.url, "https://example.com/repo.git");
    assert_eq!(git.ref_name, "v1.2.3");
    assert_eq!(git.subdir, "skills/foo");
    assert!(!git.shallow);
    assert!(s.local_path.is_none());

    Ok(())
}

#[test]
fn hash_tree_is_deterministic_and_ignores_dot_git() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    fs::write(dir.path().join("a.txt"), "a")?;
    fs::create_dir_all(dir.path().join("dir"))?;
    fs::write(dir.path().join("dir").join("b.txt"), "b")?;
    fs::create_dir_all(dir.path().join(".git"))?;
    fs::write(dir.path().join(".git").join("config"), "ignored")?;

    let (files1, hash1) = hash_tree(dir.path())?;
    let paths1: Vec<&str> = files1.iter().map(|f| f.path.as_str()).collect();
    assert_eq!(paths1, vec!["a.txt", "dir/b.txt"]);

    let (files2, hash2) = hash_tree(dir.path())?;
    let paths2: Vec<&str> = files2.iter().map(|f| f.path.as_str()).collect();
    assert_eq!(paths2, vec!["a.txt", "dir/b.txt"]);
    assert_eq!(hash1, hash2);

    Ok(())
}

#[test]
fn compose_module_tree_applies_overlays_in_order() -> anyhow::Result<()> {
    let tmp = tempfile::tempdir()?;

    let upstream = tmp.path().join("upstream");
    let global = tmp.path().join("global");
    let project = tmp.path().join("project");
    let out = tmp.path().join("out");

    fs::create_dir_all(&upstream)?;
    fs::create_dir_all(&global)?;
    fs::create_dir_all(&project)?;

    fs::write(upstream.join("hello.txt"), "upstream")?;
    fs::write(global.join("hello.txt"), "global")?;
    fs::write(global.join("only-global.txt"), "g")?;
    fs::write(project.join("hello.txt"), "project")?;

    compose_module_tree(&upstream, &[&global, &project], &out)?;
    assert_eq!(fs::read_to_string(out.join("hello.txt"))?, "project");
    assert_eq!(fs::read_to_string(out.join("only-global.txt"))?, "g");

    Ok(())
}

fn git(cwd: &std::path::Path, args: &[&str]) -> anyhow::Result<String> {
    let out = std::process::Command::new("git")
        .current_dir(cwd)
        .args(args)
        .output()?;
    if !out.status.success() {
        anyhow::bail!(
            "git {:?} failed: {}",
            args,
            String::from_utf8_lossy(&out.stderr)
        );
    }
    Ok(String::from_utf8(out.stdout)?)
}

#[test]
fn resolve_upstream_module_root_auto_fetches_missing_git_checkout() -> anyhow::Result<()> {
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
    fs::create_dir_all(&home.repo_dir)?;

    // Create a local git repo to act as a "git source".
    let upstream = root.join("upstream");
    fs::create_dir_all(&upstream)?;
    fs::write(upstream.join("SKILL.md"), "# test\n")?;
    let _ = git(&upstream, &["init"])?;
    let _ = git(&upstream, &["config", "user.email", "test@example.com"])?;
    let _ = git(&upstream, &["config", "user.name", "Test"])?;
    let _ = git(&upstream, &["add", "."])?;
    let _ = git(&upstream, &["commit", "-m", "init"])?;
    let commit = git(&upstream, &["rev-parse", "HEAD"])?.trim().to_string();

    // Write a lockfile that points to the exact commit, but do NOT populate the cache.
    let repo = RepoPaths::resolve(&home, None)?;
    let lock = Lockfile {
        version: 1,
        generated_at: "2026-01-11T00:00:00Z".to_string(),
        modules: vec![LockedModule {
            id: "skill:test".to_string(),
            module_type: ModuleType::Skill,
            resolved_source: ResolvedSource {
                local_path: None,
                git: Some(ResolvedGitSource {
                    url: upstream.to_string_lossy().to_string(),
                    commit: commit.clone(),
                    subdir: String::new(),
                }),
            },
            resolved_version: commit.clone(),
            sha256: "unused".to_string(),
            file_manifest: Vec::new(),
        }],
    };
    lock.save(&repo.lockfile_path)?;

    let module = agentpack::config::Module {
        id: "skill:test".to_string(),
        module_type: ModuleType::Skill,
        enabled: true,
        tags: Vec::new(),
        targets: Vec::new(),
        source: agentpack::config::Source {
            local_path: None,
            git: Some(agentpack::config::GitSource {
                url: upstream.to_string_lossy().to_string(),
                ref_name: commit.clone(),
                subdir: String::new(),
                shallow: false,
            }),
        },
        metadata: Default::default(),
    };

    let root = resolve_upstream_module_root(&home, &repo, &module)?;
    assert!(root.join("SKILL.md").is_file());

    // Ensure the checkout was created under cache.
    let store = agentpack::store::Store::new(&home);
    let url = upstream.to_string_lossy().to_string();
    let checkout_dir = store.git_checkout_dir(&url, &commit);
    assert!(
        checkout_dir.exists(),
        "expected checkout dir missing at {}",
        checkout_dir.display()
    );

    Ok(())
}

#[test]
fn ensure_overlay_skeleton_does_not_overwrite_existing_overlay() -> anyhow::Result<()> {
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
    fs::create_dir_all(&home.repo_dir)?;

    let repo = RepoPaths::resolve(&home, None)?;

    // Local upstream module
    let module_root = home.repo_dir.join("modules").join("skill_test");
    fs::create_dir_all(&module_root)?;
    fs::write(module_root.join("SKILL.md"), "upstream\n")?;

    let module = agentpack::config::Module {
        id: "skill_test".to_string(),
        module_type: ModuleType::Skill,
        enabled: true,
        tags: Vec::new(),
        targets: Vec::new(),
        source: agentpack::config::Source {
            local_path: Some(agentpack::config::LocalPathSource {
                path: "modules/skill_test".to_string(),
            }),
            git: None,
        },
        metadata: Default::default(),
    };

    let mut profiles = std::collections::BTreeMap::new();
    profiles.insert(
        "default".to_string(),
        agentpack::config::Profile {
            include_tags: Vec::new(),
            include_modules: Vec::new(),
            exclude_modules: Vec::new(),
        },
    );
    let manifest = agentpack::config::Manifest {
        version: 1,
        profiles,
        targets: Default::default(),
        modules: vec![module],
    };

    let overlay_dir = home
        .repo_dir
        .join("overlays")
        .join("skill_test")
        .to_path_buf();

    let s1 = ensure_overlay_skeleton(&home, &repo, &manifest, "skill_test", &overlay_dir)?;
    assert!(s1.created);
    assert!(overlay_dir.join("SKILL.md").is_file());
    assert!(
        overlay_dir
            .join(".agentpack")
            .join("baseline.json")
            .is_file()
    );

    // Simulate user edit.
    fs::write(overlay_dir.join("SKILL.md"), "overlay\n")?;

    let s2 = ensure_overlay_skeleton(&home, &repo, &manifest, "skill_test", &overlay_dir)?;
    assert!(!s2.created);
    assert_eq!(
        fs::read_to_string(overlay_dir.join("SKILL.md"))?,
        "overlay\n"
    );

    Ok(())
}

#[test]
fn ensure_overlay_skeleton_sparse_creates_metadata_without_copying() -> anyhow::Result<()> {
    let tmp = tempfile::tempdir()?;
    let root = tmp.path().to_path_buf();
    let home = AgentpackHome {
        repo_dir: root.join("repo"),
        state_dir: root.join("state"),
        cache_dir: root.join("cache"),
        snapshots_dir: root.join("state").join("snapshots"),
        logs_dir: root.join("state").join("logs"),
        root,
    };
    fs::create_dir_all(&home.repo_dir)?;

    let repo = RepoPaths::resolve(&home, None)?;

    let module_root = home.repo_dir.join("modules").join("skill_test");
    fs::create_dir_all(&module_root)?;
    fs::write(module_root.join("SKILL.md"), "upstream\n")?;

    let module = agentpack::config::Module {
        id: "skill_test".to_string(),
        module_type: ModuleType::Skill,
        enabled: true,
        tags: Vec::new(),
        targets: Vec::new(),
        source: agentpack::config::Source {
            local_path: Some(agentpack::config::LocalPathSource {
                path: "modules/skill_test".to_string(),
            }),
            git: None,
        },
        metadata: Default::default(),
    };

    let mut profiles = std::collections::BTreeMap::new();
    profiles.insert(
        "default".to_string(),
        agentpack::config::Profile {
            include_tags: Vec::new(),
            include_modules: Vec::new(),
            exclude_modules: Vec::new(),
        },
    );
    let manifest = agentpack::config::Manifest {
        version: 1,
        profiles,
        targets: Default::default(),
        modules: vec![module],
    };

    let overlay_dir = home.repo_dir.join("overlays").join("skill_test");

    let s1 = ensure_overlay_skeleton_sparse(&home, &repo, &manifest, "skill_test", &overlay_dir)?;
    assert!(s1.created);
    assert!(!overlay_dir.join("SKILL.md").exists());
    assert!(
        overlay_dir
            .join(".agentpack")
            .join("baseline.json")
            .is_file()
    );
    assert!(overlay_dir.join(".agentpack").join("module_id").is_file());

    let s2 = ensure_overlay_skeleton_sparse(&home, &repo, &manifest, "skill_test", &overlay_dir)?;
    assert!(!s2.created);
    assert!(!overlay_dir.join("SKILL.md").exists());

    Ok(())
}

#[test]
fn materialize_overlay_from_upstream_copies_missing_files_without_overwriting_edits()
-> anyhow::Result<()> {
    let tmp = tempfile::tempdir()?;
    let root = tmp.path().to_path_buf();
    let home = AgentpackHome {
        repo_dir: root.join("repo"),
        state_dir: root.join("state"),
        cache_dir: root.join("cache"),
        snapshots_dir: root.join("state").join("snapshots"),
        logs_dir: root.join("state").join("logs"),
        root,
    };
    fs::create_dir_all(&home.repo_dir)?;

    let repo = RepoPaths::resolve(&home, None)?;

    let module_root = home.repo_dir.join("modules").join("skill_test");
    fs::create_dir_all(&module_root)?;
    fs::write(module_root.join("SKILL.md"), "upstream\n")?;
    fs::write(module_root.join("extra.txt"), "extra\n")?;

    let module = agentpack::config::Module {
        id: "skill_test".to_string(),
        module_type: ModuleType::Skill,
        enabled: true,
        tags: Vec::new(),
        targets: Vec::new(),
        source: agentpack::config::Source {
            local_path: Some(agentpack::config::LocalPathSource {
                path: "modules/skill_test".to_string(),
            }),
            git: None,
        },
        metadata: Default::default(),
    };

    let mut profiles = std::collections::BTreeMap::new();
    profiles.insert(
        "default".to_string(),
        agentpack::config::Profile {
            include_tags: Vec::new(),
            include_modules: Vec::new(),
            exclude_modules: Vec::new(),
        },
    );
    let manifest = agentpack::config::Manifest {
        version: 1,
        profiles,
        targets: Default::default(),
        modules: vec![module],
    };

    let overlay_dir = home.repo_dir.join("overlays").join("skill_test");

    ensure_overlay_skeleton_sparse(&home, &repo, &manifest, "skill_test", &overlay_dir)?;

    // Simulate user edit.
    fs::write(overlay_dir.join("SKILL.md"), "overlay\n")?;

    materialize_overlay_from_upstream(&home, &repo, &manifest, "skill_test", &overlay_dir)?;

    assert_eq!(
        fs::read_to_string(overlay_dir.join("SKILL.md"))?,
        "overlay\n"
    );
    assert_eq!(
        fs::read_to_string(overlay_dir.join("extra.txt"))?,
        "extra\n"
    );

    Ok(())
}

#[test]
fn validate_command_frontmatter_requires_description_and_allowed_tools_for_bash()
-> anyhow::Result<()> {
    let tmp = tempfile::tempdir()?;

    fs::write(tmp.path().join("cmd.md"), "!bash\necho hi\n")?;
    let err = validate_materialized_module(&ModuleType::Command, "command:test", tmp.path())
        .unwrap_err()
        .to_string();
    assert!(err.contains("missing YAML frontmatter"));

    fs::write(
        tmp.path().join("cmd.md"),
        "---\ndescription: \"x\"\n---\n\n!bash\necho hi\n",
    )?;
    let err = validate_materialized_module(&ModuleType::Command, "command:test", tmp.path())
        .unwrap_err()
        .to_string();
    assert!(err.contains("missing allowed-tools"));

    fs::write(
        tmp.path().join("cmd.md"),
        "---\ndescription: \"x\"\nallowed-tools:\n  - Bash(\"echo hi\")\n---\n\n!bash\necho hi\n",
    )?;
    validate_materialized_module(&ModuleType::Command, "command:test", tmp.path())?;

    Ok(())
}

#[test]
fn copy_tree_ignores_agentpack_metadata_dir() -> anyhow::Result<()> {
    let tmp = tempfile::tempdir()?;
    let src = tmp.path().join("src");
    let dst = tmp.path().join("dst");
    fs::create_dir_all(src.join(".agentpack"))?;
    fs::write(src.join(".agentpack").join("baseline.json"), "x")?;
    fs::write(src.join("file.txt"), "y")?;

    copy_tree(&src, &dst)?;
    assert!(dst.join("file.txt").is_file());
    assert!(!dst.join(".agentpack").join("baseline.json").exists());

    Ok(())
}

#[test]
fn latest_snapshot_filters_kinds() -> anyhow::Result<()> {
    let tmp = tempfile::tempdir()?;
    let root = tmp.path().to_path_buf();
    let home = AgentpackHome {
        repo_dir: root.join("repo"),
        state_dir: root.join("state"),
        cache_dir: root.join("cache"),
        snapshots_dir: root.join("state").join("snapshots"),
        logs_dir: root.join("state").join("logs"),
        root,
    };

    fs::create_dir_all(&home.snapshots_dir)?;

    let deploy = DeploymentSnapshot {
        kind: "deploy".to_string(),
        id: "1".to_string(),
        created_at: "2026-01-01T00:00:00Z".to_string(),
        targets: vec!["codex".to_string()],
        managed_files: Vec::new(),
        changes: Vec::new(),
        rolled_back_to: None,
        lockfile_sha256: None,
        backup_root: String::new(),
    };
    deploy.save(&DeploymentSnapshot::path(&home, &deploy.id))?;

    let bootstrap = DeploymentSnapshot {
        kind: "bootstrap".to_string(),
        id: "2".to_string(),
        created_at: "2026-01-02T00:00:00Z".to_string(),
        targets: vec!["codex".to_string()],
        managed_files: Vec::new(),
        changes: Vec::new(),
        rolled_back_to: None,
        lockfile_sha256: None,
        backup_root: String::new(),
    };
    bootstrap.save(&DeploymentSnapshot::path(&home, &bootstrap.id))?;

    let picked = agentpack::state::latest_snapshot(&home, &["deploy"])?;
    assert_eq!(picked.expect("snapshot").kind, "deploy");

    Ok(())
}

#[test]
fn target_path_orders_stably() {
    let a = TargetPath {
        target: "codex".to_string(),
        path: Path::new("/tmp/a").to_path_buf(),
    };
    let b = TargetPath {
        target: "codex".to_string(),
        path: Path::new("/tmp/b").to_path_buf(),
    };
    assert!(a < b);
}

#[test]
fn plan_orders_changes_stably() -> anyhow::Result<()> {
    let tmp = tempfile::tempdir()?;
    let a = tmp.path().join("a.txt");
    let b = tmp.path().join("b.txt");

    fs::write(&b, "old")?;

    let mut desired = deploy::DesiredState::new();
    desired.insert(
        TargetPath {
            target: "codex".to_string(),
            path: a.clone(),
        },
        DesiredFile {
            bytes: b"new-a".to_vec(),
            module_ids: Vec::new(),
        },
    );
    desired.insert(
        TargetPath {
            target: "codex".to_string(),
            path: b.clone(),
        },
        DesiredFile {
            bytes: b"new-b".to_vec(),
            module_ids: Vec::new(),
        },
    );

    let plan = deploy::plan(&desired, None)?;
    let changed: Vec<String> = plan
        .changes
        .into_iter()
        .map(|c| {
            Path::new(&c.path)
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string()
        })
        .collect();
    assert_eq!(changed, vec!["a.txt", "b.txt"]);
    Ok(())
}

#[test]
fn plan_matches_golden_snapshot() -> anyhow::Result<()> {
    let tmp = tempfile::tempdir()?;
    let a = tmp.path().join("a.txt");
    let b = tmp.path().join("b.txt");

    fs::write(&b, "old")?;

    let mut desired = deploy::DesiredState::new();
    desired.insert(
        TargetPath {
            target: "codex".to_string(),
            path: a.clone(),
        },
        DesiredFile {
            bytes: b"new-a".to_vec(),
            module_ids: Vec::new(),
        },
    );
    desired.insert(
        TargetPath {
            target: "codex".to_string(),
            path: b.clone(),
        },
        DesiredFile {
            bytes: b"new-b".to_vec(),
            module_ids: Vec::new(),
        },
    );

    let plan = deploy::plan(&desired, None)?;
    let tmp_prefix = tmp.path().to_string_lossy().replace('\\', "/");
    let normalize = |p: &str| {
        let mut s = p.replace('\\', "/");
        if s.starts_with(&tmp_prefix) {
            s = s.replacen(&tmp_prefix, "<TMP>", 1);
        }
        s
    };

    let changes: Vec<serde_json::Value> = plan
        .changes
        .iter()
        .map(|c| {
            serde_json::json!({
                "target": &c.target,
                "op": &c.op,
                "path": normalize(&c.path),
                "before_sha256": c.before_sha256.as_deref(),
                "after_sha256": c.after_sha256.as_deref(),
                "update_kind": c.update_kind.as_ref(),
                "reason": &c.reason,
            })
        })
        .collect();
    let out = serde_json::json!({
        "changes": changes,
        "summary": &plan.summary,
    });

    let mut actual = serde_json::to_string_pretty(&out)?;
    if !actual.ends_with('\n') {
        actual.push('\n');
    }

    let expected = fs::read_to_string("tests/golden/plan_codex.json")?;
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn load_managed_paths_from_manifests_reads_rel_paths() -> anyhow::Result<()> {
    let tmp = tempfile::tempdir()?;
    let root = tmp.path().to_path_buf();

    let mut manifest = TargetManifest::new("codex".to_string(), "t".to_string(), None);
    manifest.managed_files.push(ManagedManifestFile {
        path: "a.txt".to_string(),
        sha256: "deadbeef".to_string(),
        module_ids: vec!["module:x".to_string()],
    });
    manifest.save(&manifest_path(&root))?;

    let roots = vec![TargetRoot {
        target: "codex".to_string(),
        root: root.clone(),
        scan_extras: true,
    }];
    let managed = agentpack::target_manifest::load_managed_paths_from_manifests(&roots)?;
    let managed = managed.managed_paths;
    assert!(managed.contains(&TargetPath {
        target: "codex".to_string(),
        path: root.join("a.txt"),
    }));
    Ok(())
}

#[test]
fn plan_deletes_only_manifest_managed_files() -> anyhow::Result<()> {
    let tmp = tempfile::tempdir()?;
    let root = tmp.path();

    fs::write(root.join("managed.txt"), "x")?;
    fs::write(root.join("unmanaged.txt"), "y")?;

    let mut manifest = TargetManifest::new("codex".to_string(), "t".to_string(), None);
    manifest.managed_files.push(ManagedManifestFile {
        path: "managed.txt".to_string(),
        sha256: agentpack::hash::sha256_hex(b"x"),
        module_ids: vec!["module:x".to_string()],
    });
    manifest.save(&manifest_path(root))?;

    let roots = vec![TargetRoot {
        target: "codex".to_string(),
        root: root.to_path_buf(),
        scan_extras: true,
    }];
    let managed = agentpack::target_manifest::load_managed_paths_from_manifests(&roots)?;
    let managed = managed.managed_paths;

    let desired = deploy::DesiredState::new();
    let plan = deploy::plan(&desired, Some(&managed))?;
    assert_eq!(plan.summary.delete, 1);
    assert!(
        plan.changes
            .iter()
            .any(|c| c.path.ends_with("managed.txt") && matches!(c.op, deploy::Op::Delete))
    );
    assert!(
        !plan
            .changes
            .iter()
            .any(|c| c.path.ends_with("unmanaged.txt"))
    );
    Ok(())
}

#[test]
fn apply_plan_writes_target_manifests() -> anyhow::Result<()> {
    let home_tmp = tempfile::tempdir()?;
    let home_root = home_tmp.path().to_path_buf();
    let home = AgentpackHome {
        repo_dir: home_root.join("repo"),
        state_dir: home_root.join("state"),
        cache_dir: home_root.join("cache"),
        snapshots_dir: home_root.join("state").join("snapshots"),
        logs_dir: home_root.join("state").join("logs"),
        root: home_root,
    };

    let target_tmp = tempfile::tempdir()?;
    let target_root = target_tmp.path().to_path_buf();

    let managed_path = target_root.join("managed.txt");
    let mut desired = deploy::DesiredState::new();
    desired.insert(
        TargetPath {
            target: "codex".to_string(),
            path: managed_path.clone(),
        },
        DesiredFile {
            bytes: b"hello\n".to_vec(),
            module_ids: vec!["module:test".to_string()],
        },
    );

    let plan = deploy::plan(&desired, None)?;
    let roots = vec![TargetRoot {
        target: "codex".to_string(),
        root: target_root.clone(),
        scan_extras: true,
    }];

    let snapshot = agentpack::apply::apply_plan(&home, "deploy", &plan, &desired, None, &roots)?;

    assert!(managed_path.is_file());
    let mf = manifest_path(&target_root);
    assert!(mf.is_file());

    let manifest = TargetManifest::load(&mf)?;
    assert_eq!(manifest.tool, "codex");
    assert_eq!(manifest.snapshot_id.as_deref(), Some(snapshot.id.as_str()));
    assert_eq!(manifest.managed_files.len(), 1);
    assert_eq!(manifest.managed_files[0].path, "managed.txt");
    assert_eq!(
        manifest.managed_files[0].module_ids,
        vec!["module:test".to_string()]
    );
    assert_eq!(
        manifest.managed_files[0].sha256,
        agentpack::hash::sha256_hex(b"hello\n")
    );

    Ok(())
}
