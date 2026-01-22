# Change: Refactor explain JSON data construction into app layer

## Why
CLI `agentpack explain * --json` and the MCP `explain` tool both construct `data` payloads for:
- `explain.plan` (used by `explain plan` and `explain diff`)
- `explain.status`

Today, the final JSON `data` object construction is duplicated across the CLI and MCP handlers. Centralizing it reduces the risk of output drift over time while keeping the stable `--json` contract unchanged.

## What Changes
- Add small shared helpers in `src/app/` to construct the `explain.plan` and `explain.status` JSON `data` objects deterministically.
- Update CLI `explain` and the MCP `explain` tool to reuse the shared construction.

## Impact
- Affected specs: `agentpack-cli`, `agentpack-mcp` (explain JSON payload remains consistent).
- Affected code:
  - `src/app/*`
  - `src/cli/commands/explain.rs`
  - `src/mcp/tools.rs`
- No user-facing behavior change expected.
