# Change: Extract MCP evolve_propose tool implementation into its own module

## Why
`src/mcp/tools.rs` contains implementations for many tools and is large. Extracting the `evolve_propose` tool handler into a dedicated module improves readability and makes future changes safer and easier to review.

## What Changes
- Move the `evolve_propose` tool implementation (`call_evolve_propose_in_process`) from `src/mcp/tools.rs` into `src/mcp/tools/evolve_propose.rs`.
- Keep the public MCP surface and JSON output unchanged.

## Impact
- Affected specs: `agentpack-mcp` (no behavioral change expected).
- Affected code:
  - `src/mcp/tools.rs`
  - `src/mcp/tools/evolve_propose.rs`
