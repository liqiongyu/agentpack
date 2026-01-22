# Change: Refactor operator assets checks into app layer

## Why
The operator assets checks (used for `status` warnings / next actions) are duplicated between:
- CLI `status` (`src/cli/commands/status.rs`)
- MCP `status` tool (`src/mcp/tools.rs`)

This duplication increases the risk of drift (different warning messages, different suggestion behavior) and makes maintenance harder.

## What Changes
- Introduce a small shared helper module under `src/app/` for:
  - extracting `agentpack_version` from operator assets
  - checking operator asset files / command directories and emitting warnings + suggestions
- Wire both CLI status and MCP status to use the shared helpers.

## Impact
- Affected specs: `agentpack-cli` (status warnings are shared by MCP), `agentpack-mcp` (status tool reuses the CLI envelope).
- Affected code:
  - `src/app/operator_assets.rs` (new)
  - `src/cli/commands/status.rs` (dedupe helpers)
  - `src/mcp/tools.rs` (dedupe helpers)
- No user-facing behavior change expected.
