# Change: Refactor evolve.restore JSON data construction into app layer

## Why
CLI `evolve restore --json` and the MCP `evolve_restore` tool both build the same `data` payload:
- `restored`
- `summary`
- `reason`

Today, the final `data` object construction is duplicated across the CLI and MCP handlers. Centralizing it reduces the risk of output drift over time while keeping the stable `--json` contract unchanged.

## What Changes
- Add a small shared helper in `src/app/` to construct the `evolve.restore` JSON `data` object deterministically.
- Update CLI `evolve restore --json` and the MCP `evolve_restore` tool to reuse it.

## Impact
- Affected specs: `agentpack-cli`, `agentpack-mcp` (evolve.restore JSON payload remains consistent).
- Affected code:
  - `src/app/*`
  - `src/cli/commands/evolve.rs`
  - `src/mcp/tools.rs`
- No user-facing behavior change expected.
