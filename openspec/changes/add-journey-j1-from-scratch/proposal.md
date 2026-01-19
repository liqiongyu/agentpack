# Change: Add journey test J1 (from-scratch deploy flow)

## Why

We need a deterministic, offline “happy path” journey test that exercises the core CLI lifecycle end-to-end:
`init → update → preview --diff → deploy --apply → status → rollback`.

This catches integration regressions that unit/contract tests miss (targets wiring, snapshot lifecycle, and JSON envelope stability under real command sequences).

## What Changes

- Add a new integration test for Journey J1 (from scratch first deploy) using `tests/journeys/common::TestEnv`.
- Use `--json` for stable assertions and include required `--yes` for mutating commands.

## Impact

- Affected specs: none (tests-only)
- Affected code: `tests/` only
- Affected runtime behavior: none
