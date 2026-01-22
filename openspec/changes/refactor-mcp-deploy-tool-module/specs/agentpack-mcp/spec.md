# agentpack-mcp (delta)

## ADDED Requirements

### Requirement: MCP deploy tool handler is modular
The system SHALL implement the MCP `deploy` tool handler in a dedicated Rust module, while preserving the existing external MCP behavior and Agentpack JSON envelope semantics.

#### Scenario: Deploy tool handler extracted into a module
- **WHEN** the MCP server is built
- **THEN** the `deploy` tool handler implementation lives in `src/mcp/tools/deploy.rs`
- **AND** the `tools/list` and `tools/call` behavior remains unchanged
