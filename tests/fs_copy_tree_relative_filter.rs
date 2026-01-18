use std::fs;

use agentpack::fs::{copy_tree, copy_tree_missing_only};

#[test]
fn copy_tree_does_not_skip_when_src_path_contains_dot_agentpack() -> anyhow::Result<()> {
    let tmp = tempfile::tempdir()?;

    let src = tmp
        .path()
        .join(".agentpack")
        .join("repo")
        .join("modules")
        .join("skills")
        .join("imported")
        .join("foo");
    fs::create_dir_all(&src)?;
    fs::write(src.join("SKILL.md"), "# test skill\n")?;
    fs::create_dir_all(src.join(".agentpack"))?;
    fs::write(src.join(".agentpack").join("ignored.txt"), "ignored")?;
    fs::create_dir_all(src.join(".git"))?;
    fs::write(src.join(".git").join("config"), "ignored")?;

    let dst = tmp.path().join("out");
    copy_tree(&src, &dst)?;

    assert!(dst.join("SKILL.md").is_file());
    assert!(!dst.join(".agentpack").join("ignored.txt").exists());
    assert!(!dst.join(".git").join("config").exists());

    Ok(())
}

#[test]
fn copy_tree_missing_only_does_not_skip_when_src_path_contains_dot_agentpack() -> anyhow::Result<()>
{
    let tmp = tempfile::tempdir()?;

    let src = tmp
        .path()
        .join(".agentpack")
        .join("repo")
        .join("modules")
        .join("skills")
        .join("imported")
        .join("foo");
    fs::create_dir_all(&src)?;
    fs::write(src.join("SKILL.md"), "# test skill\n")?;
    fs::create_dir_all(src.join(".agentpack"))?;
    fs::write(src.join(".agentpack").join("ignored.txt"), "ignored")?;
    fs::create_dir_all(src.join(".git"))?;
    fs::write(src.join(".git").join("config"), "ignored")?;

    let dst = tmp.path().join("out");
    fs::create_dir_all(&dst)?;
    fs::write(dst.join("already.txt"), "keep")?;

    copy_tree_missing_only(&src, &dst)?;

    assert!(dst.join("SKILL.md").is_file());
    assert_eq!(fs::read_to_string(dst.join("already.txt"))?, "keep");
    assert!(!dst.join(".agentpack").join("ignored.txt").exists());
    assert!(!dst.join(".git").join("config").exists());

    Ok(())
}
