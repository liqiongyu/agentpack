## 1. Implementation

- [x] 1.1 Add an in-process handler path for `evolve propose` that returns CLI-compatible `--json` envelopes without printing to stdout.
- [x] 1.2 Update MCP tool dispatch for `evolve_propose` to use the in-process handler (no subprocess).
- [x] 1.3 Preserve CLI-style `evolve.propose --json` envelope fields and stable error codes.

## 2. Spec deltas

- [x] 2.1 Add a delta requirement describing MCP `evolve_propose` in-process execution (archive with `--skip-specs` since this is refactor-only).

## 3. Validation

- [x] 3.1 `openspec validate refactor-mcp-tools-evolve-propose-handler --strict --no-interactive`
- [x] 3.2 `cargo fmt --all -- --check`
- [x] 3.3 `cargo clippy --all-targets --all-features -- -D warnings`
- [x] 3.4 `cargo test --all --locked`
