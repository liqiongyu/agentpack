## Why

Organizations often need to restrict where agentpack-managed modules can be sourced from (supply-chain control). Today, `agentpack policy lint` can validate operator asset hygiene and basic distribution policy, but it does not enforce any allowlist for git remotes referenced by `repo/agentpack.yaml`.

This makes it easy to accidentally introduce modules sourced from unexpected remotes.

## What changes

- Extend `repo/agentpack.org.yaml` with an optional `supply_chain_policy` section.
- When `supply_chain_policy.allowed_git_remotes` is configured, `agentpack policy lint` validates that every git-sourced module in `repo/agentpack.yaml` uses a remote that matches the allowlist.
- Violations are reported as standard policy lint issues and fail with `E_POLICY_VIOLATIONS` (no new error codes).

## Impact

- Opt-in only: no behavior change unless `supply_chain_policy.allowed_git_remotes` is configured.
- CI-friendly: no network access required; checks are based on config contents.
