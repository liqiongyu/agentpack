# Change: Refactor MCP into submodules

## Why

`src/mcp.rs` has grown into a large, mixed-responsibility module (server wiring, tool implementations, and confirm-token logic). This makes it harder to extend MCP safely and increases the risk of regressions when adding new tools.

Splitting MCP into focused submodules improves maintainability while preserving the existing MCP tool contract and stable confirmation behavior.

## What Changes

- Convert `src/mcp.rs` into a module directory `src/mcp/`.
- Split MCP implementation into focused submodules:
  - `server` (rmcp server wiring + request handling)
  - `tools` (tool definitions + dispatch)
  - `confirm` (confirm_token generation/validation + store)
- Preserve public API surface via re-exports from `src/mcp/mod.rs`.

## Impact

- Affected specs: none (no user-facing behavior change)
- Affected code: `src/mcp.rs` and MCP call sites
- Affected runtime behavior: none expected
