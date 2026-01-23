---
## ADDED Requirements

### Requirement: Remaining MCP tools reuse the shared anyhow error envelope helper
The system SHALL refactor remaining MCP tools that inline `UserError` extraction to use the shared `anyhow::Error` -> envelope helper while preserving tool behavior and schemas.

#### Scenario: MCP tool behavior remains unchanged after adopting the shared helper
- **GIVEN** the MCP server exposes tools with stable behavior and input schemas
- **WHEN** remaining tool error mapping is refactored to use the shared helper
- **THEN** the tools continue to behave the same and advertise the same `inputSchema` values
