# Change: Refactor status drift summary helpers into app layer

## Why
The `status` drift summary helpers (notably `summary_by_root` and the filtered `summary` recomputation) are duplicated between:
- CLI `status --json` (`src/cli/commands/status.rs`)
- MCP `status` tool (`src/mcp/tools.rs`)

This duplication increases the risk of drift (ordering, counts, and grouping behavior) and makes future changes harder to review.

## What Changes
- Introduce small shared helpers under `src/app/` for:
  - computing drift summaries (`summary`, `summary_by_root`)
- Wire both CLI status and MCP status to use this shared module.

## Impact
- Affected specs: `agentpack-cli` (status json), `agentpack-mcp` (status tool reuses the CLI envelope).
- Affected code:
  - `src/app/status_drift.rs` (new)
  - `src/cli/commands/status.rs` (dedupe helpers)
  - `src/mcp/tools.rs` (dedupe helpers)
- No user-facing behavior change expected.
