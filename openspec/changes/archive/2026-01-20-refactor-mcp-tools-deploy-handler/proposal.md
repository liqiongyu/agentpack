# Change: Refactor MCP deploy tool to compute plan in-process

## Why

The MCP server currently shells out to `agentpack --json deploy` for the deploy planning stage. This adds overhead and makes it harder to ensure MCP reuses the same in-process business logic as the CLI/TUI (M3-REF-004).

## What Changes

- Update MCP tool `deploy` to compute the deploy plan envelope in-process via existing handler logic (no `agentpack --json` subprocess).
- Preserve the confirm-token flow by continuing to derive `confirm_plan_hash` from the stable envelope `data` and attaching `confirm_token` fields to the returned envelope.

## Impact

- Affected specs: none (refactor-only; expected no user-facing behavior change)
- Affected code: `src/mcp/tools.rs`
