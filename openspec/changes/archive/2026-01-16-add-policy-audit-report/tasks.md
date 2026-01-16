## 1. Implementation

- [x] 1.1 Add `agentpack policy audit` CLI entry (read-only)
- [x] 1.2 Define audit JSON payload (modules + optional change summary)
- [x] 1.3 Include org policy pack lock info when available
- [x] 1.4 Update docs (`docs/SPEC.md`, `docs/GOVERNANCE.md`, `docs/CLI.md`)
- [x] 1.5 Update `help --json` golden snapshot as needed
- [x] 1.6 Add tests for `policy audit --json`

## 2. Validation

- [x] 2.1 Run `openspec validate add-policy-audit-report --strict`
- [x] 2.2 Run `cargo fmt --all -- --check`
- [x] 2.3 Run `cargo clippy --all-targets --all-features -- -D warnings`
- [x] 2.4 Run `cargo test --all --locked`
