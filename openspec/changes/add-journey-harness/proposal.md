# Change: Add reusable journey/E2E test harness helpers

## Why

Upcoming journey (E2E) tests need a consistent, reusable way to set up isolated temp environments (home, config repo, project repo) and to run `agentpack` commands deterministically.

Without a shared harness, each journey test will re-implement environment setup and command helpers, making tests harder to read and maintain.

## What Changes

- Add `tests/journeys/common` with a `TestEnv` builder:
  - temp HOME + temp `AGENTPACK_HOME`
  - temp project repo with stable origin (deterministic `project_id`)
  - helpers to run `agentpack` via `assert_cmd` with safe env defaults
- Add a small smoke test to ensure the harness compiles and can run `agentpack init` end-to-end.

## Impact

- Affected specs: none (tests-only)
- Affected code: `tests/` only
- Affected runtime behavior: none
