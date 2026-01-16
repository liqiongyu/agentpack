## 1. Implementation

- [ ] 1.1 Add `supply_chain_policy.require_lockfile` to org config schema (additive)
- [ ] 1.2 Enforce lockfile presence/sync in `agentpack policy lint` for enabled git modules
- [ ] 1.3 Add tests for missing/invalid/out-of-sync lockfile cases
- [ ] 1.4 Update docs (`docs/SPEC.md`, `docs/GOVERNANCE.md`)

## 2. Validation

- [ ] 2.1 Run `openspec validate add-policy-require-lockfile --strict`
- [ ] 2.2 Run `cargo fmt --all -- --check`
- [ ] 2.3 Run `cargo clippy --all-targets --all-features -- -D warnings`
- [ ] 2.4 Run `cargo test --all --locked`
