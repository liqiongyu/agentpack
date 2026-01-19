fn read_to_string(path: &str) -> anyhow::Result<String> {
    std::fs::read_to_string(path).map_err(|e| anyhow::anyhow!("read {path}: {e}"))
}

fn extract_install_tags(text: &str) -> Vec<String> {
    let needle = "--tag v";
    let mut out = Vec::new();

    let mut idx = 0;
    while let Some(pos) = text[idx..].find(needle) {
        idx += pos + needle.len();
        let rest = &text[idx..];
        let mut version = String::new();
        for ch in rest.chars() {
            if ch.is_ascii_digit() || ch == '.' {
                version.push(ch);
                continue;
            }
            break;
        }
        if !version.is_empty() {
            out.push(format!("{needle}{version}"));
        }
    }

    out
}

fn assert_all_install_tags_match(path: &str, haystack: &str, expected: &str) -> anyhow::Result<()> {
    let tags = extract_install_tags(haystack);
    if tags.is_empty() {
        anyhow::bail!("{path} is out of date.\n\nExpected to find:\n  {expected}");
    }
    let mismatched: Vec<String> = tags
        .into_iter()
        .filter(|t| t.as_str() != expected)
        .collect();
    if mismatched.is_empty() {
        return Ok(());
    }
    anyhow::bail!(
        "{path} contains install tags that do not match the crate version.\n\nExpected:\n  {expected}\n\nFound mismatches:\n  {}\n",
        mismatched.join("\n  ")
    );
}

#[test]
fn install_docs_tag_matches_crate_version() -> anyhow::Result<()> {
    let version = env!("CARGO_PKG_VERSION");
    let expected = format!("--tag v{version}");

    let readme = read_to_string("README.md")?;
    assert_all_install_tags_match("README.md", &readme, &expected)?;

    let readme_zh = read_to_string("README.zh-CN.md")?;
    assert_all_install_tags_match("README.zh-CN.md", &readme_zh, &expected)?;

    let quickstart = read_to_string("docs/tutorials/quickstart.md")?;
    assert_all_install_tags_match("docs/tutorials/quickstart.md", &quickstart, &expected)?;

    let quickstart_zh = read_to_string("docs/zh-CN/tutorials/quickstart.md")?;
    assert_all_install_tags_match(
        "docs/zh-CN/tutorials/quickstart.md",
        &quickstart_zh,
        &expected,
    )?;

    Ok(())
}
