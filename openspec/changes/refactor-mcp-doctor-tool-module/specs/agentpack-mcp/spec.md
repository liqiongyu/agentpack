# agentpack-mcp (delta)

## ADDED Requirements

### Requirement: MCP doctor tool handler is modular
The system SHALL implement the MCP `doctor` tool handler in a dedicated Rust module, while preserving the existing external MCP behavior and Agentpack JSON envelope semantics.

#### Scenario: Doctor tool handler extracted into a module
- **WHEN** the MCP server is built
- **THEN** the `doctor` tool handler implementation lives in `src/mcp/tools/doctor.rs`
- **AND** the `tools/list` and `tools/call` behavior remains unchanged
