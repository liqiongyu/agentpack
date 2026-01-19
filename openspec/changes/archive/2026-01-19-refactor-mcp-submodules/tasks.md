## 1. Implementation

- [x] 1.1 Convert `src/mcp.rs` into `src/mcp/mod.rs`.
- [x] 1.2 Split MCP code into `server/tools/confirm` submodules.
- [x] 1.3 Preserve tool behavior and JSON/error-code stability.

## 2. Spec deltas

- [x] 2.1 Add a delta requirement describing MCP modularization (archive with `--skip-specs` since this is refactor-only).

## 3. Validation

- [x] 3.1 `openspec validate refactor-mcp-submodules --strict --no-interactive`
- [x] 3.2 `cargo fmt --all -- --check`
- [x] 3.3 `cargo clippy --all-targets --all-features -- -D warnings`
- [x] 3.4 `cargo test --all --locked`
