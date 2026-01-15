## Why

Organizations want to distribute and audit governance rules independently of core Agentpack configuration. A “policy pack” reference that can be pinned (lockfile) makes CI enforcement reproducible and reviewable.

Governance MUST remain explicit opt-in and MUST NOT affect personal-user defaults or core deploy semantics.

## What changes

- Add a governance config file: `repo/agentpack.org.yaml`
  - Allows referencing an optional `policy_pack` source (local path or git).
- Add a governance lockfile: `repo/agentpack.org.lock.json`
  - Pins the referenced policy pack to an immutable version (e.g. git commit) plus a content hash for auditability.
- Add `agentpack policy lock`
  - Resolves the policy pack reference and writes/updates the lockfile.
- Integrate with `agentpack policy lint`
  - When a policy pack is configured, lint SHOULD use the pinned lockfile and SHOULD avoid network access.

## Scope (v1)

- Source types supported for policy packs:
  - `local:` (repo-relative path)
  - `git:` (url + ref + optional subdir), pinned to a commit in the lockfile
- Deterministic lockfile generation:
  - stable ordering
  - deterministic hashing of policy pack content
- Governance-only isolation:
  - only `agentpack policy ...` reads `agentpack.org.yaml` / `agentpack.org.lock.json`
  - core commands (`plan/diff/deploy/...`) remain unchanged

## Non-goals

- No new “policy DSL” beyond wiring a referenced pack and pinning it.
- No automatic enforcement in core deploy/apply flows (governance remains isolated).
- No requirement for network access in `policy lint` (it should be CI-friendly; missing lock may be surfaced as an issue).

## Impact

- New opt-in files in the config repo: `agentpack.org.yaml` and `agentpack.org.lock.json`.
- New mutating command `agentpack policy lock` (must obey `--json` + `--yes` guardrails).
- Additional stable error codes for governance config/lock failures may be introduced.
