# Change: Adopt `envelope_from_anyhow_error` across remaining MCP tools

## Why
`src/mcp/tools/envelope.rs` now provides `envelope_from_anyhow_error()` to map `anyhow::Error` (including embedded `UserError`) into the standard Agentpack MCP tool error envelope.

A few MCP tools still inline the old `UserError`-extraction boilerplate. Converging on the shared helper reduces duplication and keeps error envelope behavior consistent.

## What Changes
- Refactor the remaining MCP tools that still inline `UserError` extraction to use `envelope_from_anyhow_error()`.
- No behavior / schema changes intended (pure refactor).

## Impact
- Affected specs: `agentpack-mcp` (no behavioral change expected).
- Affected code:
  - `src/mcp/tools/deploy_plan.rs`
  - `src/mcp/tools/deploy_apply.rs`
  - `src/mcp/tools/evolve_propose.rs`
  - `src/mcp/tools/evolve_restore.rs`
