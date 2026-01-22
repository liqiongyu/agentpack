# Change: Refactor status next_actions_detailed generation into app layer

## Why
CLI `status --json` and the MCP `status` tool both emit:
- `next_actions` (ordered command strings)
- `next_actions_detailed` (objects with `action` + `command`)

While ordering and action-code mapping are already shared in `src/app/next_actions.rs`, the final `next_actions_detailed` construction is still duplicated in the CLI and MCP handlers.

Centralizing the `next_actions_detailed` construction reduces the risk of drift over time and keeps the JSON output stable.

## What Changes
- Add a small shared helper in `src/app/next_actions.rs` to build `next_actions` + `next_actions_detailed` together.
- Update CLI `status --json` and MCP `status` to reuse it.

## Impact
- Affected specs: `agentpack-cli`, `agentpack-mcp` (status next_actions_detailed behavior remains consistent).
- Affected code:
  - `src/app/next_actions.rs`
  - `src/cli/commands/status.rs`
  - `src/mcp/tools.rs`
- No user-facing behavior change expected.
