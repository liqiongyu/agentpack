# Change: Deduplicate unexpected-error handling in MCP tool router

## Why
`src/mcp/tools/router.rs` repeats the same "unexpected error" mapping pattern for many tools (build `E_UNEXPECTED` structured errors via `envelope_error`). This makes the router harder to scan and increases the chance of inconsistency when adding new tools.

## What Changes
- Introduce a small helper in `src/mcp/tools/router.rs` for the repeated `E_UNEXPECTED` error mapping.
- Keep MCP tool behavior, input schemas, and JSON envelopes unchanged (pure refactor).

## Impact
- Affected specs: `agentpack-mcp` (no behavioral change expected).
- Affected code:
  - `src/mcp/tools/router.rs`
