## 1. Implementation

- [x] Enforce `supply_chain_policy.allowed_git_remotes` for `policy_pack.source` (git) in `policy lint`.
- [x] Enforce `supply_chain_policy.allowed_git_remotes` for `policy_pack.source` (git) in `policy lock`.
- [x] Update docs to clarify the allowlist also applies to policy pack git sources.
- [x] Add integration tests for:
  - `policy lint` failing when policy pack remote is not allowlisted
  - `policy lock` failing when policy pack remote is not allowlisted

## 2. Validation

- [x] `openspec validate update-policy-pack-allowlist-remotes --strict --no-interactive`
- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy --all-targets --all-features -- -D warnings`
- [x] `cargo test --all --locked`
