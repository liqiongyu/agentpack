# Change: Refactor doctor JSON data construction into app layer

## Why
CLI `doctor --json` and the MCP `doctor` tool both build the same `data` payload:
- `machine_id`
- `roots`
- `gitignore_fixes`
- optional `next_actions`

Today, the final `data` object construction is duplicated across the CLI and MCP handlers. Centralizing it reduces the risk of output drift over time while keeping the stable `--json` contract unchanged.

## What Changes
- Add a small shared helper in `src/app/` to construct the `doctor` JSON `data` object deterministically.
- Update CLI `doctor --json` and the MCP `doctor` tool to reuse it.

## Impact
- Affected specs: `agentpack-cli`, `agentpack-mcp` (doctor JSON payload remains consistent).
- Affected code:
  - `src/app/*`
  - `src/cli/commands/doctor.rs`
  - `src/mcp/tools.rs`
- No user-facing behavior change expected.
