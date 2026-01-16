## 1. Implementation

- [ ] 1.1 Add `agentpack policy audit` CLI entry (read-only)
- [ ] 1.2 Define audit JSON payload (modules + optional change summary)
- [ ] 1.3 Include org policy pack lock info when available
- [ ] 1.4 Update docs (`docs/SPEC.md`, `docs/GOVERNANCE.md`, `docs/CLI.md`)
- [ ] 1.5 Update `help --json` golden snapshot as needed
- [ ] 1.6 Add tests for `policy audit --json`

## 2. Validation

- [ ] 2.1 Run `openspec validate add-policy-audit-report --strict`
- [ ] 2.2 Run `cargo fmt --all -- --check`
- [ ] 2.3 Run `cargo clippy --all-targets --all-features -- -D warnings`
- [ ] 2.4 Run `cargo test --all --locked`
