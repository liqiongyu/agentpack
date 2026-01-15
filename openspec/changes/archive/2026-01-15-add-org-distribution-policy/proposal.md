# Change: Org distribution policy (governance)

## Why
Teams may want to enforce **asset distribution rules** (what must be enabled, what must not be enabled) without changing personal-user defaults or core deploy semantics.

This change defines a minimal, CI-friendly “org distribution policy” that is **explicit opt-in** and is enforced only via `agentpack policy ...`.

## What Changes
- Add an optional `distribution_policy` section to `repo/agentpack.org.yaml` (governance config).
- Extend `agentpack policy lint` to validate the minimal distribution policy against `repo/agentpack.yaml` (targets/modules).
- Reuse existing governance failure behavior: policy violations fail with `E_POLICY_VIOLATIONS` in `--json`.

## Impact
- Affected specs:
  - `agentpack` (governance config schema)
  - `agentpack-cli` (`policy lint` behavior)
- Affected code (planned):
  - `src/policy_pack.rs` (org config parsing)
  - `src/policy.rs` (policy lint checks)
- Affected docs/tests (planned):
  - `docs/SPEC.md`, `docs/GOVERNANCE.md`
  - `tests/cli_policy_lint.rs`

## Non-goals
- No changes to core commands (`plan/diff/deploy/status/doctor/...`) behavior or defaults.
- No network access during `policy lint`.
