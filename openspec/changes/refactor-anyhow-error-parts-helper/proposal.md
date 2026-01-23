# Change: Centralize `anyhow::Error` -> `(code, message, details)` mapping

## Why
Both the CLI JSON error envelope and MCP tool error envelope paths map an `anyhow::Error` into a `(code, message, details)` triple (prefer embedded `UserError`, otherwise `E_UNEXPECTED`).

Today this mapping logic is duplicated across callers. Centralizing it reduces duplication and helps keep envelope behavior consistent.

## What Changes
- Add a shared helper that converts an `anyhow::Error` into `(code, message, details)` suitable for envelopes.
- Refactor CLI JSON error printing and MCP tool envelope helpers to reuse the shared helper.
- No behavior / schema changes intended (pure refactor).

## Impact
- Affected specs: `agentpack-cli`, `agentpack-mcp` (no behavioral change expected).
- Affected code:
  - `src/user_error.rs`
  - `src/cli/json.rs`
  - `src/mcp/tools/envelope.rs`
