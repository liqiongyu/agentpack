# agentpack-mcp (delta)

## ADDED Requirements

### Requirement: MCP deploy_apply tool handler is modular
The system SHALL implement the MCP `deploy_apply` tool handler in a dedicated Rust module, while preserving the existing external MCP behavior and Agentpack JSON envelope semantics.

#### Scenario: Deploy apply tool handler extracted into a module
- **WHEN** the MCP server is built
- **THEN** the `deploy_apply` tool handler implementation lives in `src/mcp/tools/deploy_apply.rs`
- **AND** the `tools/list` and `tools/call` behavior remains unchanged
