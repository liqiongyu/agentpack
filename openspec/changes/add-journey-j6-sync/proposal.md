# Change: Add journey test J6 (multi-machine sync)

## Why

Real users often run Agentpack on multiple machines, keeping their config repo in sync via a git remote. We need an end-to-end journey test that exercises `agentpack sync --rebase` against a shared bare remote to ensure the workflow is deterministic and robust.

## What Changes

- Add an integration test for Journey J6 using two `tests/journeys/common::TestEnv` instances and a shared bare git remote:
  - initialize/push config repo from machine A
  - clone config repo on machine B
  - create divergent commits and reconcile via `agentpack sync --rebase --json --yes`
  - assert the merged result is visible on both machines

## Impact

- Affected specs: none (tests-only)
- Affected code: `tests/` only
- Affected runtime behavior: none
