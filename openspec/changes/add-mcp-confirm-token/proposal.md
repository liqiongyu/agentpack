# Change: agentpack-mcp two-stage confirmation (confirm_token)

## Why
MCP tools represent arbitrary code execution; hosts should require explicit user approval for mutating operations.
For `deploy`, we want a two-stage flow that ties the eventual apply to the exact plan the user approved, reducing TOCTOU and prompt-injection risks.

## What Changes
- `deploy` tool returns the normal Agentpack `deploy --json` envelope **plus** an additive `data.confirm_token` field.
- `deploy_apply` requires:
  - explicit approval (`yes=true`, existing behavior)
  - `confirm_token` from the most recent matching `deploy` plan
- `deploy_apply` refuses to run when the token is missing/expired/mismatched, returning stable error codes.

## Impact
- Affected specs: `openspec/specs/agentpack-mcp/spec.md`
- Affected code: `src/mcp.rs`, MCP stdio tests
- Contract: additive-only change to tool payloads (envelope gains new optional fields in MCP context)
