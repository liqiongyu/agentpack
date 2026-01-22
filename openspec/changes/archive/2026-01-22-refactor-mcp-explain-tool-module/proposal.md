# Change: Extract MCP explain tool implementation into its own module

## Why
`src/mcp/tools.rs` contains implementations for many tools and has grown large. Extracting the `explain` tool handler into a dedicated module improves readability and makes future changes safer and easier to review.

## What Changes
- Move the `explain` tool implementation (`call_explain_in_process`) from `src/mcp/tools.rs` into `src/mcp/tools/explain.rs`.
- Keep the public MCP surface and JSON output unchanged.

## Impact
- Affected specs: `agentpack-mcp` (no behavioral change expected).
- Affected code:
  - `src/mcp/tools.rs`
  - `src/mcp/tools/explain.rs`
