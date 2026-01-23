# Change: Extract MCP read-only helper into a dedicated module

## Why
`src/mcp/tools.rs` still contains the generic “read-only in-process” helper used by multiple MCP tools. Moving it into a dedicated module keeps `tools.rs` focused on routing and makes future refactors easier to review.

## What Changes
- Move the MCP read-only helper (`call_read_only_in_process`) out of `src/mcp/tools.rs` into `src/mcp/tools/read_only.rs`.
- Keep MCP tool behavior and JSON envelopes identical (pure refactor).

## Impact
- Affected specs: `agentpack-mcp` (no behavioral change expected).
- Affected code:
  - `src/mcp/tools.rs`
  - `src/mcp/tools/read_only.rs` (new)
