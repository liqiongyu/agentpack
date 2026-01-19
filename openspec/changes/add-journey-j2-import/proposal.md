# Change: Add journey test J2 (import existing assets)

## Why

`agentpack import` is a core onboarding path for existing users. We need an end-to-end journey test that verifies:
- dry-run does not write to disk, and
- apply requires explicit confirmation in `--json` mode and produces a usable manifest/modules layout.

This protects against regressions in scanning logic (user vs project scope) and ensures imported assets can be previewed/deployed.

## What Changes

- Add an integration test for Journey J2 using `tests/journeys/common::TestEnv`:
  - seed a temp HOME + temp project repo with realistic Codex/Claude asset files
  - run `agentpack import` in dry-run, then `--apply`
  - verify imported destinations exist and preview/deploy succeeds (project profile when needed)

## Impact

- Affected specs: none (tests-only)
- Affected code: `tests/` only
- Affected runtime behavior: none
