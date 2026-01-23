---
## ADDED Requirements

### Requirement: MCP tool unexpected-error envelopes are constructed via a shared helper
The system SHALL centralize construction of MCP tool `E_UNEXPECTED` structured errors so tool implementations remain readable while preserving MCP behavior and schemas.

#### Scenario: MCP tool behavior remains unchanged after centralizing unexpected errors
- **GIVEN** the MCP server exposes tools with stable behavior and input schemas
- **WHEN** `E_UNEXPECTED` tool error mapping is refactored into a shared helper
- **THEN** the tools continue to behave the same and advertise the same `inputSchema` values
