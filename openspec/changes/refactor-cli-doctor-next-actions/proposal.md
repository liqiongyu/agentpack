# Change: Refactor doctor next_actions ordering to shared helper

## Why
CLI `doctor` currently has its own `ordered_next_actions` helper, while other surfaces (e.g. status / MCP) use the shared app-layer helper.

This duplication increases the risk of ordering drift across commands and makes future changes harder to review.

## What Changes
- Reuse `src/app/next_actions.rs` ordering helper from CLI `doctor`.
- Remove the local duplicated ordering implementation from `src/cli/commands/doctor.rs`.

## Impact
- Affected specs: `agentpack-cli` (doctor JSON field ordering), `agentpack-mcp` (no change; already uses shared helper).
- Affected code:
  - `src/cli/commands/doctor.rs` (dedupe)
- No user-facing behavior change expected.
