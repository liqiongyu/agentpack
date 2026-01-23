## ADDED Requirements

### Requirement: MCP tool registry is modularized
The system SHALL keep the MCP tool registry implemented in a dedicated module file so `src/mcp/tools.rs` stays focused on routing while preserving the advertised tool list and schemas.

#### Scenario: MCP tool list and schemas remain unchanged after registry refactor
- **GIVEN** the MCP server exposes a stable set of tools with stable input schemas
- **WHEN** the registry/list code is refactored into a dedicated module
- **THEN** the tool names and `inputSchema` values remain unchanged
