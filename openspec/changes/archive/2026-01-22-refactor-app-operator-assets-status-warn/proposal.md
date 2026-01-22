# Change: Refactor status operator assets warnings to shared helper

## Why
CLI `status` and the MCP `status` tool both warn when embedded operator assets are missing or outdated (and suggest running `agentpack bootstrap`), but they currently duplicate the orchestration logic.

This duplication risks drift in which paths are checked, when warnings are emitted, and which bootstrap commands are suggested.

## What Changes
- Centralize the shared `warn_operator_assets_if_outdated` orchestration under the app layer.
- Update CLI `status` and MCP `status` tool to call the shared helper.

## Impact
- Affected specs: `agentpack-cli` (status warnings/next_actions behavior), `agentpack-mcp` (status tool warnings/next_actions behavior).
- Affected code:
  - `src/app/operator_assets.rs` (new shared helper)
  - `src/cli/commands/status.rs` (dedupe)
  - `src/mcp/tools.rs` (dedupe)
- No user-facing behavior change expected.
