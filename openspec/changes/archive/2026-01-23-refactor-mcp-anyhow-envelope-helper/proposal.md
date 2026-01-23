# Change: Centralize mapping from `anyhow::Error` to MCP tool error envelopes

## Why
Several MCP tools repeat the same logic to:
1) Scan an `anyhow::Error` chain for `UserError`
2) Choose `code`/`message`/`details` accordingly
3) Build an Agentpack JSON error envelope via `envelope_error(...)`

This duplication is noisy, easy to drift, and makes otherwise-straightforward handlers harder to read.

## What Changes
- Add a shared helper (based on `envelope_error`) for converting an `anyhow::Error` into the standard MCP tool error envelope.
- Refactor a first batch of read-only MCP tool handlers to use the helper.
- No behavior / schema changes intended (pure refactor).

## Impact
- Affected specs: `agentpack-mcp` (no behavioral change expected).
- Affected code:
  - `src/mcp/tools/envelope.rs`
  - `src/mcp/tools/read_only.rs`
  - `src/mcp/tools/preview.rs`
  - `src/mcp/tools/status.rs`
  - `src/mcp/tools/doctor.rs`
  - `src/mcp/tools/rollback.rs`
  - `src/mcp/tools/explain.rs`
