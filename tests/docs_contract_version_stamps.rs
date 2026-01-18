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
fn contract_docs_version_stamps_match_crate_version() -> anyhow::Result<()> {
    let version = env!("CARGO_PKG_VERSION");
    let header = format!("Current as of **v{version}**");

    let spec = read_to_string("docs/SPEC.md")?;
    assert_contains("docs/SPEC.md", &spec, &header)?;

    let json_api = read_to_string("docs/JSON_API.md")?;
    assert_contains("docs/JSON_API.md", &json_api, &header)?;
    assert_contains(
        "docs/JSON_API.md",
        &json_api,
        &format!("\"version\": \"{version}\""),
    )?;

    let error_codes = read_to_string("docs/ERROR_CODES.md")?;
    assert_contains("docs/ERROR_CODES.md", &error_codes, &header)?;

    Ok(())
}
