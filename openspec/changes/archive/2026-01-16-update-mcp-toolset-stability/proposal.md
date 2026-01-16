# Change: stabilize MCP tool set and schemas

## Why

Agentpack’s MCP server is intended to be a thin, structured execution surface that reuses the CLI as the single semantic source of truth. To be reliable across hosts (Codex/IDE clients), the MCP tool catalog needs a stable, documented set of tools and input schemas.

## What Changes

- Stabilize the MCP tool list to include the core read-only and mutating operations used in the operator loop:
  - `doctor`, `status`, `preview`, `diff`, `plan`, `deploy`, `deploy_apply`, `rollback`, `evolve_propose`, `evolve_restore`, `explain`
- Document each tool input schema in `openspec/specs/agentpack-mcp/spec.md` (outputs reuse the CLI `--json` envelope).
- Update MCP server implementation and tests to match the stabilized tool set.

## Non-Goals

- Do not introduce two-stage confirmation (`confirm_token`) in this change (tracked separately).

## Impact

- Affected specs: `openspec/specs/agentpack-mcp/spec.md`
- Affected code: `src/mcp.rs`
- Affected tests: `tests/mcp_server_stdio*.rs`
