# Change: Refactor MCP rollback tool to call handler directly

## Why

The MCP server currently shells out to `agentpack --json` for the `rollback` tool. This duplicates CLI argument/default handling and blocks progress toward M3-REF-004 (single-source business logic across CLI/MCP/TUI).

## What Changes

- Update MCP `rollback` tool to execute in-process via the existing rollback handler (`src/handlers/rollback.rs`) rather than spawning an `agentpack --json` subprocess.
- Preserve the exact Agentpack `--json` envelope shape and stable error code behavior by mapping `UserError` into the MCP tool envelope.

## Impact

- Affected specs: none (refactor-only; expected no user-facing behavior change)
- Affected code: `src/mcp/tools.rs`
