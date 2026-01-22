# Change: Refactor preview JSON data construction into app layer

## Why
CLI `preview --json` and the MCP `preview` tool both build the same `data` payload:
- `profile`
- `targets`
- `plan` (`changes`, `summary`)
- optional `diff` (`changes`, `summary`, `files`)

Today, the final `data` object construction is duplicated across the CLI and MCP handlers. Centralizing it reduces the risk of output drift over time while keeping the stable `--json` contract unchanged.

## What Changes
- Add a small shared helper in `src/app/` to construct the `preview` JSON `data` object deterministically.
- Update CLI `preview --json` and the MCP `preview` tool to reuse it.

## Impact
- Affected specs: `agentpack-cli`, `agentpack-mcp` (preview JSON payload remains consistent).
- Affected code:
  - `src/app/*`
  - `src/cli/commands/preview.rs`
  - `src/mcp/tools.rs`
- No user-facing behavior change expected.
