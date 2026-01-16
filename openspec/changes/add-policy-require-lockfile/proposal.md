## Why

For supply-chain control, teams often want to ensure that any git-sourced modules are pinned to immutable commits for reproducibility and auditability. Agentpack’s `repo/agentpack.lock.json` already pins git sources to commits, but there is currently no governance check that the lockfile exists and is in sync when git sources are present.

Adding an opt-in policy lint check keeps the default personal-user experience unchanged while enabling CI enforcement for org-managed repos.

## What changes

- Extend `repo/agentpack.org.yaml` `supply_chain_policy` with an optional boolean `require_lockfile`.
- When `require_lockfile=true` and `repo/agentpack.yaml` contains enabled git-sourced modules:
  - `agentpack policy lint` MUST require `repo/agentpack.lock.json` to exist and be valid.
  - The lockfile MUST contain entries for the enabled git modules, and the locked remote URL MUST match the configured remote URL.
- Violations are reported as policy lint issues and fail with `E_POLICY_VIOLATIONS` (no new error codes).

## Impact

- Opt-in only; no behavior change unless `supply_chain_policy.require_lockfile` is enabled.
- CI-friendly; does not require network access.
