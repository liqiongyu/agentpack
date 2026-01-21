use std::path::Path;

#[test]
fn docs_demo_assets_exist_and_are_referenced() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    let tape = root.join("docs/assets/demo.tape");
    assert!(tape.exists(), "missing tape: {}", tape.display());

    let gif = root.join("docs/assets/demo.gif");
    assert!(gif.exists(), "missing gif: {}", gif.display());

    let assets_readme = root.join("docs/assets/README.md");
    assert!(
        assets_readme.exists(),
        "missing assets README: {}",
        assets_readme.display()
    );

    let root_readme = std::fs::read_to_string(root.join("README.md")).expect("read README.md");
    assert!(
        root_readme.contains("docs/assets/demo.gif"),
        "expected README.md to reference docs/assets/demo.gif"
    );
}
