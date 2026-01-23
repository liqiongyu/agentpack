# Change: Extract MCP tool routing into a dedicated module

## Why
`src/mcp/tools.rs` currently contains the full `call_tool` routing match, which makes the file longer and increases review surface for unrelated changes. Moving the routing logic into a dedicated module keeps `tools.rs` focused on module wiring and exports while preserving MCP behavior.

## What Changes
- Move MCP tool routing logic (`call_tool`) out of `src/mcp/tools.rs` into `src/mcp/tools/router.rs`.
- Keep MCP tool behavior, input schemas, and JSON envelopes unchanged (pure refactor).

## Impact
- Affected specs: `agentpack-mcp` (no behavioral change expected).
- Affected code:
  - `src/mcp/tools.rs`
  - `src/mcp/tools/router.rs` (new)
