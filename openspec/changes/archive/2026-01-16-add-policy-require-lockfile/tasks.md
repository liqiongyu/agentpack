## 1. Implementation

- [x] 1.1 Add `supply_chain_policy.require_lockfile` to org config schema (additive)
- [x] 1.2 Enforce lockfile presence/sync in `agentpack policy lint` for enabled git modules
- [x] 1.3 Add tests for missing/invalid/out-of-sync lockfile cases
- [x] 1.4 Update docs (`docs/SPEC.md`, `docs/GOVERNANCE.md`)

## 2. Validation

- [x] 2.1 Run `openspec validate add-policy-require-lockfile --strict`
- [x] 2.2 Run `cargo fmt --all -- --check`
- [x] 2.3 Run `cargo clippy --all-targets --all-features -- -D warnings`
- [x] 2.4 Run `cargo test --all --locked`
