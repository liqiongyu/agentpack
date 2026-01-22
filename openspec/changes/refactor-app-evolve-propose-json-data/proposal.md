# Change: Refactor evolve.propose JSON data construction into app layer

## Why
CLI `evolve propose --json` and the MCP `evolve_propose` tool both build the same `data` payload for the `evolve.propose` command (with shapes that depend on the outcome: noop / dry-run / created).

Today, the final `data` object construction is duplicated across the CLI and MCP handlers. Centralizing it reduces the risk of output drift over time while keeping the stable `--json` contract unchanged.

## What Changes
- Add small shared helpers in `src/app/` to construct the `evolve.propose` JSON `data` objects deterministically.
- Update CLI `evolve propose --json` and the MCP `evolve_propose` tool to reuse the shared construction.

## Impact
- Affected specs: `agentpack-cli`, `agentpack-mcp` (evolve.propose JSON payload remains consistent).
- Affected code:
  - `src/app/*`
  - `src/cli/commands/evolve.rs`
  - `src/mcp/tools.rs`
- No user-facing behavior change expected.
