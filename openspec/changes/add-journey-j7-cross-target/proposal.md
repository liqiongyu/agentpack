# Change: Add journey test J7 (cross-target consistency)

## Why

Agentpack supports deploying the same module to multiple targets (e.g., Codex + Claude Code). We need an end-to-end journey test that validates the multi-target path is consistent and that snapshots/manifests enable correct rollback across all targets.

## What Changes

- Add an integration test for Journey J7 that:
  - defines a single `skill:*` module targeting both `codex` and `claude_code`
  - deploys to `--target all` and asserts both target outputs exist
  - performs a second deployment with modified content
  - rolls back to the first snapshot and asserts both targets revert

## Impact

- Affected specs: none (tests-only)
- Affected code: `tests/` only
- Affected runtime behavior: none
