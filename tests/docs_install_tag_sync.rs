fn read_to_string(path: &str) -> anyhow::Result<String> {
    std::fs::read_to_string(path).map_err(|e| anyhow::anyhow!("read {path}: {e}"))
}

fn assert_contains(path: &str, haystack: &str, needle: &str) -> anyhow::Result<()> {
    if haystack.contains(needle) {
        return Ok(());
    }
    anyhow::bail!("{path} is out of date.\n\nExpected to find:\n  {needle}");
}

#[test]
fn install_docs_tag_matches_crate_version() -> anyhow::Result<()> {
    let version = env!("CARGO_PKG_VERSION");
    let expected = format!("--tag v{version}");

    let readme = read_to_string("README.md")?;
    assert_contains("README.md", &readme, &expected)?;

    let readme_zh = read_to_string("README.zh-CN.md")?;
    assert_contains("README.zh-CN.md", &readme_zh, &expected)?;

    let quickstart = read_to_string("docs/QUICKSTART.md")?;
    assert_contains("docs/QUICKSTART.md", &quickstart, &expected)?;

    let quickstart_zh = read_to_string("docs/zh-CN/QUICKSTART.md")?;
    assert_contains("docs/zh-CN/QUICKSTART.md", &quickstart_zh, &expected)?;

    Ok(())
}
