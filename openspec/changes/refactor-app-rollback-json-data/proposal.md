# Change: Refactor rollback JSON data construction into app layer

## Why
CLI `rollback --json` and the MCP `rollback` tool both build the same `data` payload:
- `rolled_back_to`
- `event_snapshot_id`

Today, the final `data` object construction is duplicated across the CLI and MCP handlers. Centralizing it reduces the risk of output drift over time while keeping the stable `--json` contract unchanged.

## What Changes
- Add a small shared helper in `src/app/` to construct the `rollback` JSON `data` object deterministically.
- Update CLI `rollback --json` and the MCP `rollback` tool to reuse it.

## Impact
- Affected specs: `agentpack-cli`, `agentpack-mcp` (rollback JSON payload remains consistent).
- Affected code:
  - `src/app/*`
  - `src/cli/commands/rollback.rs`
  - `src/mcp/tools.rs`
- No user-facing behavior change expected.
