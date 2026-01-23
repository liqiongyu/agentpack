## ADDED Requirements

### Requirement: MCP tool argument types are modularized
The system SHALL keep MCP tool argument structs/enums implemented in a dedicated module file so `src/mcp/tools.rs` stays focused on routing while preserving MCP tool behavior and schemas.

#### Scenario: MCP tool schemas remain unchanged after args refactor
- **GIVEN** the MCP server exposes tools with stable input schemas
- **WHEN** tool argument types are refactored into a dedicated module
- **THEN** the advertised tool `inputSchema` values remain unchanged
