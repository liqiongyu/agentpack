## 1. Implementation

- [x] 1.1 Add `supply_chain_policy.allowed_git_remotes` to org config schema (additive)
- [x] 1.2 Enforce allowlist in `agentpack policy lint` for git-sourced modules
- [x] 1.3 Add tests for allowlist pass/fail cases
- [x] 1.4 Update docs (`docs/SPEC.md`, `docs/GOVERNANCE.md`)

## 2. Validation

- [x] 2.1 Run `openspec validate add-policy-allowlist-remotes --strict`
- [x] 2.2 Run `cargo fmt --all -- --check`
- [x] 2.3 Run `cargo clippy --all-targets --all-features -- -D warnings`
- [x] 2.4 Run `cargo test --all --locked`
