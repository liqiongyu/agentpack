# Change: Enforce allowed git remotes for policy packs

## Why
Policy packs are an opt-in governance surface, and should have a minimal trust chain to prevent accidental or unsafe adoption of policy pack sources.

Agentpack already pins policy packs via `repo/agentpack.org.lock.json`, but it does not currently enforce a git remote allowlist for `policy_pack.source`. This makes it too easy for a repo to point at an unexpected/untrusted remote without CI catching it.

## What Changes
- Extend `supply_chain_policy.allowed_git_remotes` enforcement to `policy_pack.source` when it is a git source:
  - `agentpack policy lint --json` reports `E_POLICY_VIOLATIONS` when the policy pack remote is not allowlisted.
  - `agentpack policy lock` refuses to lock a policy pack from a non-allowlisted git remote (stable config error).
- Document the behavior and keep governance opt-in (core commands unchanged).

## Impact
- Affected code: `src/policy.rs`, `src/policy_pack.rs` (and shared allowlist helpers).
- Affected docs: `docs/SPEC.md`, `docs/GOVERNANCE.md`.
- External contract: additive / opt-in only; no changes to `--json` envelope (`schema_version=1`).

## Acceptance
- With `supply_chain_policy.allowed_git_remotes` configured:
  - `policy lint --json` fails with `E_POLICY_VIOLATIONS` and includes a machine-readable issue for a non-allowlisted policy pack remote.
  - `policy lock --json --yes` fails with a stable policy config error and includes `{remote, allowed_git_remotes}` details.
- When the allowlist is not configured, existing `policy lint` / `policy lock` behavior remains unchanged.
- Covered by tests (integration).
