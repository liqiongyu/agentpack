# Change: Extract MCP tool argument types into a dedicated module

## Why
`src/mcp/tools.rs` still contains a large number of MCP tool argument structs/enums, which makes it harder to review routing logic changes. Moving these types into a dedicated module keeps `tools.rs` focused on routing while preserving MCP behavior.

## What Changes
- Move MCP tool argument structs/enums out of `src/mcp/tools.rs` into `src/mcp/tools/args.rs`.
- Keep MCP tool behavior, input schemas, and JSON envelopes unchanged (pure refactor).

## Impact
- Affected specs: `agentpack-mcp` (no behavioral change expected).
- Affected code:
  - `src/mcp/tools.rs`
  - `src/mcp/tools/args.rs` (new)
