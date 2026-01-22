# Change: Refactor status next_actions ordering into app layer

## Why
The “next actions” helpers (ordering + action codes) are currently duplicated between:
- CLI `status --json` (`src/cli/commands/status.rs`)
- MCP `status` tool (`src/mcp/tools.rs`)

This duplication increases the risk of drift (different ordering, different `next_actions_detailed.action` codes) and makes future changes harder to review.

## What Changes
- Introduce a small shared helper module under `src/app/` for:
  - ordering next action commands
  - mapping commands → stable action codes (for `next_actions_detailed`)
- Wire both CLI status and MCP status to use this shared module.

## Impact
- Affected specs: `agentpack-cli` (status additive fields), `agentpack-mcp` (status tool reuses the CLI envelope).
- Affected code:
  - `src/app/next_actions.rs` (new)
  - `src/cli/commands/status.rs` (dedupe helpers)
  - `src/mcp/tools.rs` (dedupe helpers)
- No user-facing behavior change expected.
