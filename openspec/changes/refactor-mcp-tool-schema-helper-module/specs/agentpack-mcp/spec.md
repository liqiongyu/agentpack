## ADDED Requirements

### Requirement: MCP tool schema helpers are modularized
The system SHALL keep MCP tool schema construction and argument deserialization helper functions implemented in a dedicated module file so `src/mcp/tools.rs` stays focused on routing and schemas while preserving MCP tool behavior and JSON envelopes.

#### Scenario: MCP tool schemas remain unchanged after helper refactor
- **GIVEN** the MCP server exposes tools with stable input schemas
- **WHEN** schema/argument helper code is refactored for maintainability
- **THEN** the advertised tool `inputSchema` values remain unchanged
