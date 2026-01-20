# Change: Refactor MCP explain tool to call handlers directly

## Why

The MCP server still shells out to `agentpack explain ... --json` for the `explain` tool. This duplicates argument/default handling, adds overhead, and blocks progress toward M3-REF-004 (single-source business logic across CLI/MCP/TUI).

## What Changes

- Update MCP tool `explain` to compute results in-process using the same underlying logic as the CLI (`src/cli/commands/explain.rs` + shared utilities).
- Preserve the exact Agentpack `--json` envelope shape and stable error code behavior by mapping `UserError` into the MCP tool envelope.

## Impact

- Affected specs: none (refactor-only; expected no user-facing behavior change)
- Affected code: `src/mcp/tools.rs`
