#[test]
#[cfg(not(feature = "tui"))]
fn cli_reference_markdown_matches_docs_file() {
    let expected =
        std::fs::read_to_string("docs/reference/cli.md").expect("read docs/reference/cli.md");
    let actual = agentpack::docs::render_cli_reference_markdown();

    assert_eq!(
        actual, expected,
        "docs/reference/cli.md is out of date.\n\nRegenerate with:\n  cargo run --quiet -- help --markdown > docs/reference/cli.md\n"
    );
}
