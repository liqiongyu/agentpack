## 1. Implementation
- [x] 1.1 Add feature flag `target-export-dir` (default off)
- [x] 1.2 Implement `export_dir` target adapter + registry wiring
- [x] 1.3 Add mapping docs for `export_dir` (from `docs/TARGET_MAPPING_TEMPLATE.md`)
- [x] 1.4 Update targets reference docs to list `export_dir` as experimental
- [x] 1.5 Add conformance coverage for `export_dir`
- [x] 1.6 Add CI conformance matrix entry for `target-export-dir`
- [x] 1.7 Run:
  - `cargo test --test conformance_targets --no-default-features --features target-export-dir`
  - `cargo test --test docs_markdown_links`
