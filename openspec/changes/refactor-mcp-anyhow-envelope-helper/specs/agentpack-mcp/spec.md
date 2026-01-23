---
## ADDED Requirements

### Requirement: MCP tool error envelopes reuse a shared helper for `anyhow::Error` chains
The system SHALL centralize the mapping from `anyhow::Error` chains (including embedded `UserError`) to MCP tool error envelopes so tool handlers stay readable while preserving behavior and schemas.

#### Scenario: MCP tool behavior remains unchanged after centralizing anyhow error mapping
- **GIVEN** the MCP server exposes tools with stable behavior and input schemas
- **WHEN** `anyhow::Error` -> envelope mapping is refactored into a shared helper
- **THEN** the tools continue to behave the same and advertise the same `inputSchema` values
