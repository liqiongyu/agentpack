fn read_to_string(path: &str) -> anyhow::Result<String> {
    std::fs::read_to_string(path).map_err(|e| anyhow::anyhow!("read {path}: {e}"))
}

fn assert_contains(path: &str, haystack: &str, needle: &str) -> anyhow::Result<()> {
    if haystack.contains(needle) {
        return Ok(());
    }
    anyhow::bail!("{path} is out of sync.\n\nExpected to find:\n  {needle}");
}

#[test]
fn patch_overlays_docs_cover_cli_flag_and_error_codes() -> anyhow::Result<()> {
    let cli = read_to_string("docs/reference/cli.md")?;
    assert_contains(
        "docs/reference/cli.md",
        &cli,
        "overlay edit <module_id> [--scope global|machine|project] [--kind dir|patch]",
    )?;

    let overlays = read_to_string("docs/explanation/overlays.md")?;
    assert_contains("docs/explanation/overlays.md", &overlays, "Patch overlays")?;
    assert_contains(
        "docs/explanation/overlays.md",
        &overlays,
        "E_OVERLAY_PATCH_APPLY_FAILED",
    )?;
    assert_contains(
        "docs/explanation/overlays.md",
        &overlays,
        "E_OVERLAY_REBASE_CONFLICT",
    )?;
    assert_contains(
        "docs/explanation/overlays.md",
        &overlays,
        ".agentpack/conflicts/",
    )?;

    Ok(())
}
