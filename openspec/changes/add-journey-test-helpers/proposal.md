# Change: Add journey test helpers

## Why
The journey (end-to-end) tests are the primary regression guardrails for critical safety semantics (confirmation, adopt protection, overlay rebase conflicts, rollback, MCP confirm tokens). Today, they duplicate a lot of boilerplate (run command, parse JSON, assert common envelope fields), which makes them harder to maintain and easier to drift.

## What Changes
- Add shared helpers under `tests/journeys/common/` for:
  - running `agentpack` and capturing output,
  - parsing `--json` stdout reliably,
  - common assertions for success/failure envelopes and stable error codes.
- Add (or rename) a small smoke test target `journeys_smoke` that validates the harness works.
- Refactor existing `journey_j1..j7` tests to use the shared helpers (J8 remains a dedicated MCP stdio harness).

## Impact
- Affected specs: `agentpack` (test harness consistency)
- Affected code: tests only
- Compatibility: no CLI/JSON behavior changes
