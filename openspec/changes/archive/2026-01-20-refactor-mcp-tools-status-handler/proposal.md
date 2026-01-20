# Change: Refactor MCP status tool to call handlers directly

## Why

The MCP server still shells out to `agentpack status --json` for the `status` tool. This duplicates argument/default handling, adds overhead, and blocks progress toward M3-REF-004 (single-source business logic across CLI/MCP/TUI).

## What Changes

- Update MCP tool `status` to compute results in-process via the same handler logic used by the CLI (`src/handlers/status.rs` + existing engine/planning utilities).
- Preserve the exact Agentpack `--json` envelope shape (including `data.next_actions` and `data.next_actions_detailed`) and stable error code behavior by mapping `UserError` into the MCP tool envelope.

## Impact

- Affected specs: none (refactor-only; expected no user-facing behavior change)
- Affected code: `src/mcp/tools.rs`
