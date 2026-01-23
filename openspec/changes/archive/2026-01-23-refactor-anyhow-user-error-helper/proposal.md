# Change: Centralize `UserError` extraction for CLI + MCP error envelopes

## Why
Multiple surfaces (CLI JSON, CLI human/TUI formatting, MCP tool envelopes) currently duplicate the same `anyhow::Error` chain-walk to find an embedded `UserError`.

Centralizing this logic reduces duplication and keeps error mapping behavior consistent across surfaces.

## What Changes
- Introduce a shared helper for extracting `UserError` from an `anyhow::Error`.
- Refactor callers (CLI + MCP) to reuse the shared helper.
- No behavior / schema changes intended (pure refactor).

## Impact
- Affected specs: `agentpack-cli`, `agentpack-mcp` (no behavioral change expected).
- Affected code:
  - `src/user_error.rs`
  - `src/cli/json.rs`
  - `src/cli/human.rs`
  - `src/cli/commands/tui.rs`
  - `src/mcp/tools/envelope.rs`
