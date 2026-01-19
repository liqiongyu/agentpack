# Change: Add journey test J8 (MCP confirm_token)

## Why

In MCP mode, mutating operations must be explicitly approved and bound to a reviewed plan via `confirm_token`. We need an end-to-end journey test that exercises the two-stage `deploy` -> `deploy_apply` flow and validates rollback, including refusal paths for missing/mismatched tokens.

## What Changes

- Add an integration test for Journey J8 that:
  - starts `agentpack mcp serve` over stdio
  - calls `deploy` to obtain `data.confirm_token`
  - verifies `deploy_apply` is refused without a token / with a mismatched token
  - applies a deployment with the correct token
  - performs a second deployment and validates `rollback` restores the prior snapshot

## Impact

- Affected specs: none (tests-only)
- Affected code: `tests/` only
- Affected runtime behavior: none
