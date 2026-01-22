# agentpack-mcp (delta)

## ADDED Requirements

### Requirement: MCP tool handlers are modular
The MCP tool handler implementations SHALL be organized as dedicated Rust modules to keep the server maintainable, while preserving the existing external MCP behavior and Agentpack JSON envelope semantics.

#### Scenario: Status tool handler extracted into a module
- **WHEN** the MCP server is built
- **THEN** the `status` tool handler implementation lives in `src/mcp/tools/status.rs`
- **AND** the `tools/list` and `tools/call` behavior remains unchanged
