# Change: Refactor deploy JSON data construction into app layer

## Why
CLI `deploy --json` and the MCP `deploy` / `deploy_apply` tools all construct the same `data` payload for the `deploy` envelope:
- `applied` (boolean)
- optional `reason` (e.g., `no_changes`)
- optional `snapshot_id`
- `profile`
- `targets`
- `changes`
- `summary`

Today, the final `data` object construction is duplicated across the CLI and MCP handlers. Centralizing it reduces the risk of output drift over time while keeping the stable `--json` contract unchanged.

## What Changes
- Add a small shared helper in `src/app/` to construct the `deploy` JSON `data` object deterministically.
- Update CLI `deploy --json` and the MCP `deploy` / `deploy_apply` tools to reuse it.

## Impact
- Affected specs: `agentpack-cli`, `agentpack-mcp` (deploy JSON payload remains consistent).
- Affected code:
  - `src/app/*`
  - `src/cli/commands/deploy.rs`
  - `src/mcp/tools.rs`
- No user-facing behavior change expected.
