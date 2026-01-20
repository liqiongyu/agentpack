# Change: Refactor MCP plan/diff tools to call handlers directly

## Why

The MCP server currently shells out to `agentpack --json` for read-only tools like `plan` and `diff`. This duplicates argument/default handling, adds overhead, and blocks progress toward M3-REF-004 (single-source business logic across CLI/MCP/TUI).

## What Changes

- Update MCP `plan` and `diff` tools to compute results in-process via existing read-only handlers (`src/handlers/read_only.rs`).
- Preserve the exact Agentpack `--json` envelope shape and stable error code behavior by mapping `UserError` into the MCP tool envelope (instead of treating failures as unexpected errors).

## Impact

- Affected specs: none (refactor-only; expected no user-facing behavior change)
- Affected code: `src/mcp/tools.rs`
