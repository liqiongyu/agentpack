# Change: Refactor status next_actions suggestions to shared helper

## Why
CLI `status` and the MCP `status` tool both suggest `next_actions` based on the same drift summary signals (e.g. when to suggest `preview --diff`, `deploy --apply`, or `evolve propose`).

Today this suggestion logic is duplicated across the two surfaces, which increases the risk of drift over time.

## What Changes
- Centralize the status `next_actions` suggestion logic under the app layer.
- Update CLI `status` and the MCP `status` tool to reuse the shared helper.

## Impact
- Affected specs: `agentpack-cli`, `agentpack-mcp` (status next_actions suggestions).
- Affected code:
  - `src/app/` (new shared helper)
  - `src/cli/commands/status.rs` (dedupe)
  - `src/mcp/tools.rs` (dedupe)
- No user-facing behavior change expected.
