fn extract_git_tag_versions(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in text.lines() {
        let Some(idx) = line.find("--tag v") else {
            continue;
        };
        let after = &line[idx + "--tag v".len()..];
        let version = after.split_whitespace().next().unwrap_or("").trim();
        if !version.is_empty() {
            out.push(version.to_string());
        }
    }
    out
}

#[test]
fn install_snippets_tag_matches_crate_version() {
    let expected_version = env!("CARGO_PKG_VERSION").to_string();

    for path in [
        "README.md",
        "README.zh-CN.md",
        "docs/tutorials/quickstart.md",
        "docs/zh-CN/tutorials/quickstart.md",
    ] {
        let text = std::fs::read_to_string(path).unwrap_or_else(|err| {
            panic!("read {path}: {err}");
        });

        let versions = extract_git_tag_versions(&text);
        assert_eq!(
            versions,
            vec![expected_version.clone()],
            "{path}: expected exactly one `--tag v...` snippet, and it must match the crate version ({expected_version})"
        );
    }
}
