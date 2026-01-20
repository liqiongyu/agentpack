# Change: Refactor MCP deploy_apply tool to apply in-process

## Why

The MCP server still shells out to `agentpack deploy --apply --json` for the mutating `deploy_apply` tool. This adds overhead and blocks progress toward M3-REF-004 (single-source business logic across CLI/MCP/TUI), especially for the confirm-token apply path.

## What Changes

- Update MCP tool `deploy_apply` to execute the apply path in-process (using the existing deploy handler logic under `src/handlers/deploy.rs`).
- Preserve the exact Agentpack `--json` envelope shape (`command = "deploy"`) and stable error codes (including confirm-token mismatch/expired/required).

## Impact

- Affected specs: none (refactor-only; expected no user-facing behavior change)
- Affected code: `src/mcp/tools.rs`
