# Change: Refactor status drift filtering into app layer

## Why
The `status --only ...` drift filtering logic is duplicated between:
- CLI `status` (`src/cli/commands/status.rs`)
- MCP `status` tool (`src/mcp/tools.rs`)

While the underlying drift summary helpers are already shared, the “filter and recompute” flow is still duplicated, increasing the risk of behavior drift.

## What Changes
- Extend the shared `src/app/status_drift.rs` helpers with a small function that:
  - filters drift items by kind
  - recomputes the filtered `summary`
  - returns `summary_total` when filtering is active
- Wire both CLI status and MCP status to use this shared helper.

## Impact
- Affected specs: `agentpack-cli` (status json fields), `agentpack-mcp` (status tool reuses the CLI envelope).
- Affected code:
  - `src/app/status_drift.rs` (add helper)
  - `src/cli/commands/status.rs` (dedupe filter logic)
  - `src/mcp/tools.rs` (dedupe filter logic)
- No user-facing behavior change expected.
