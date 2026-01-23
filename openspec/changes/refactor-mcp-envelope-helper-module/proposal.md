# Change: Extract MCP envelope helpers into a dedicated module

## Why
`src/mcp/tools.rs` still contains shared MCP envelope and tool-result helpers used across multiple tool implementations. Moving these helpers into a dedicated module keeps `tools.rs` focused on routing and schemas and makes future refactors easier to review.

## What Changes
- Move MCP envelope/tool-result helpers (`envelope_error`, `tool_result_from_envelope`, `tool_result_from_user_error`) out of `src/mcp/tools.rs` into `src/mcp/tools/envelope.rs`.
- Keep MCP tool behavior and JSON envelopes identical (pure refactor).

## Impact
- Affected specs: `agentpack-mcp` (no behavioral change expected).
- Affected code:
  - `src/mcp/tools.rs`
  - `src/mcp/tools/envelope.rs` (new)
