---
## ADDED Requirements

### Requirement: MCP tool routing is modularized
The system SHALL keep MCP tool routing implemented in a dedicated module file so `src/mcp/tools.rs` stays focused on wiring and exports while preserving MCP tool behavior and schemas.

#### Scenario: MCP tool behavior remains unchanged after routing refactor
- **GIVEN** the MCP server exposes tools with stable behavior and input schemas
- **WHEN** tool routing is refactored into a dedicated module
- **THEN** the tools continue to behave the same and advertise the same `inputSchema` values
