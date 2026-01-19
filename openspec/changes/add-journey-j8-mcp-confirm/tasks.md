## 1. Implementation

- [x] 1.1 Add Journey J8 integration test covering MCP deploy confirm_token + apply + rollback.

## 2. Spec deltas

- [x] 2.1 Add a delta requirement describing Journey J8 coverage (archive with `--skip-specs` since this is tests-only).

## 3. Validation

- [x] 3.1 `openspec validate add-journey-j8-mcp-confirm --strict`
- [x] 3.2 `cargo fmt --all -- --check`
- [x] 3.3 `cargo clippy --all-targets --all-features -- -D warnings`
- [x] 3.4 `cargo test --all --locked`
