---
## ADDED Requirements

### Requirement: MCP router unexpected-error handling is centralized
The system SHALL centralize repeated unexpected-error handling in the MCP tool router so routing logic stays readable while preserving MCP tool behavior and schemas.

#### Scenario: MCP tool behavior remains unchanged after router error refactor
- **GIVEN** the MCP server exposes tools with stable behavior and input schemas
- **WHEN** unexpected-error mapping is refactored into a shared helper
- **THEN** the tools continue to behave the same and advertise the same `inputSchema` values
