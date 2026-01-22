# Change: Refactor plan/diff JSON data construction into app layer

## Why
CLI `plan --json` / `diff --json` and the MCP `plan` / `diff` tools all construct the same `data` payload:
- `profile`
- `targets`
- `changes`
- `summary`

Today, the final `data` object construction is duplicated across the CLI and MCP handlers. Centralizing it reduces the risk of output drift over time while keeping the stable `--json` contract unchanged.

## What Changes
- Add a small shared helper in `src/app/` to construct the plan/diff JSON `data` object deterministically.
- Update CLI `plan --json` / `diff --json` and the MCP `plan` / `diff` tools to reuse it.

## Impact
- Affected specs: `agentpack-cli`, `agentpack-mcp` (plan/diff JSON payload remains consistent).
- Affected code:
  - `src/app/*`
  - `src/cli/commands/plan.rs`
  - `src/cli/commands/diff.rs`
  - `src/mcp/tools.rs`
- No user-facing behavior change expected.
