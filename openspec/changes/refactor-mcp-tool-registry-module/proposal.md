# Change: Extract MCP tool registry into a dedicated module

## Why
`src/mcp/tools.rs` still contains the MCP tool registry (the `tools()` list), mixing tool registration with routing logic. Moving the registry into a dedicated module keeps `tools.rs` focused on routing and makes future tool additions easier to review.

## What Changes
- Move the MCP tool registry (`tools()` list) out of `src/mcp/tools.rs` into `src/mcp/tools/tool_registry.rs`.
- Keep the tool list, tool schemas, and JSON envelope behavior unchanged (pure refactor).

## Impact
- Affected specs: `agentpack-mcp` (no behavioral change expected).
- Affected code:
  - `src/mcp/tools.rs`
  - `src/mcp/tools/tool_registry.rs` (new)
