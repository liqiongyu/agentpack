# Change: Extract MCP tool schema helpers into a dedicated module

## Why
`src/mcp/tools.rs` still contains generic MCP tooling helpers (tool schema construction and argument deserialization). Moving these helpers into a dedicated module keeps `tools.rs` focused on routing, improving readability and making future refactors safer.

## What Changes
- Move MCP tool schema/args helpers (`tool_input_schema`, `tool`, `deserialize_args`) out of `src/mcp/tools.rs` into `src/mcp/tools/tool_schema.rs`.
- Keep MCP tool behavior and JSON envelopes identical (pure refactor).

## Impact
- Affected specs: `agentpack-mcp` (no behavioral change expected).
- Affected code:
  - `src/mcp/tools.rs`
  - `src/mcp/tools/tool_schema.rs` (new)
