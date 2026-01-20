# Change: Refactor MCP preview tool to call handlers directly

## Why

The MCP server currently shells out to `agentpack --json` for the `preview` tool. This duplicates CLI argument/default handling and blocks progress toward M3-REF-004 (single-source business logic across CLI/MCP/TUI).

## What Changes

- Update MCP `preview` tool to execute in-process via existing handler logic (`src/handlers/read_only.rs`), computing the same JSON envelope as `agentpack preview --json` (including `--diff` support).
- Preserve the exact Agentpack `--json` envelope shape and stable error code behavior by mapping `UserError` into the MCP tool envelope.

## Impact

- Affected specs: none (refactor-only; expected no user-facing behavior change)
- Affected code: `src/mcp/tools.rs`
