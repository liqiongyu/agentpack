# Change: Extract MCP deploy-plan helper into a dedicated module

## Why
`src/mcp/tools.rs` still contains shared logic used by multiple tools. Moving the deploy planning helper into its own module keeps `tools.rs` focused on routing, schemas, and shared envelopes, making the code easier to navigate and safer to refactor further.

## What Changes
- Move the shared MCP helper `deploy_plan_envelope_in_process` out of `src/mcp/tools.rs` into a dedicated module file.
- Keep behavior and JSON envelopes identical (pure refactor).

## Impact
- Affected specs: `agentpack-mcp` (no behavioral change expected).
- Affected code:
  - `src/mcp/tools.rs`
  - `src/mcp/tools/deploy_plan.rs` (new)
