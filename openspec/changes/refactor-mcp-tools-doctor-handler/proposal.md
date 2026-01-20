# Change: Refactor MCP doctor tool to call handlers directly

## Why

The MCP server currently shells out to `agentpack --json` for `doctor`. This adds overhead and prevents MCP from directly reusing the same in-process business logic as the CLI/TUI, which is a core goal of M3-REF-004.

## What Changes

- Update MCP tool `doctor` to compute results in-process via `src/handlers/doctor.rs` (no `agentpack --json` subprocess).
- Preserve the stable Agentpack JSON envelope and CLI-style error code mapping via `UserError`.

## Impact

- Affected specs: none (refactor-only; expected no user-facing behavior change)
- Affected code: `src/mcp/tools.rs`
