# Change: Refactor status JSON data construction into app layer

## Why
CLI `status --json` and the MCP `status` tool both build the same `data` payload:
- drift (`drift`, `summary`, `summary_by_root`, and optional `summary_total`)
- suggested next actions (`next_actions`, `next_actions_detailed`)

Today, the final `data` object construction is duplicated across the CLI and MCP handlers. Centralizing it reduces the risk of output drift over time while keeping the stable `--json` contract unchanged.

## What Changes
- Add a small shared helper in `src/app/` to construct the `status` JSON `data` object deterministically.
- Update CLI `status --json` and the MCP `status` tool to reuse it.

## Impact
- Affected specs: `agentpack-cli`, `agentpack-mcp` (status JSON payload remains consistent).
- Affected code:
  - `src/app/*`
  - `src/cli/commands/status.rs`
  - `src/mcp/tools.rs`
- No user-facing behavior change expected.
