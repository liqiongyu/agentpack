# agentpack-mcp (delta)

## ADDED Requirements

### Requirement: MCP preview tool handler is modular
The system SHALL implement the MCP `preview` tool handler in a dedicated Rust module, while preserving the existing external MCP behavior and Agentpack JSON envelope semantics.

#### Scenario: Preview tool handler extracted into a module
- **WHEN** the MCP server is built
- **THEN** the `preview` tool handler implementation lives in `src/mcp/tools/preview.rs`
- **AND** the `tools/list` and `tools/call` behavior remains unchanged
