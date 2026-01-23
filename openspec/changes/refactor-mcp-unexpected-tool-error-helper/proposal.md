# Change: Centralize MCP tool `E_UNEXPECTED` structured error creation

## Why
Several MCP tool implementations repeat the same pattern for converting unexpected errors into an Agentpack JSON envelope (`ok=false`, `errors[0].code=E_UNEXPECTED`) wrapped as an MCP `CallToolResult`.

This duplication makes it harder to keep behavior consistent and adds noise to otherwise straightforward control-flow in tool handlers.

## What Changes
- Introduce a shared helper for constructing `E_UNEXPECTED` MCP tool errors (based on `envelope_error`).
- Replace repeated `CallToolResult::structured_error(envelope_error(...))` call-sites with the helper.
- Keep MCP tool behavior, input schemas, and JSON envelopes unchanged (pure refactor).

## Impact
- Affected specs: `agentpack-mcp` (no behavioral change expected).
- Affected code:
  - `src/mcp/tools/envelope.rs`
  - `src/mcp/tools/router.rs`
  - `src/mcp/tools/deploy.rs`
  - `src/mcp/tools/deploy_apply.rs`
