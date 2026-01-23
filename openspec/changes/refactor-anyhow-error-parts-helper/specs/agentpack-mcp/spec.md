---
## ADDED Requirements

### Requirement: MCP tool error envelopes reuse a shared anyhow error-parts helper
The system SHALL refactor MCP tool error envelope construction to reuse a shared helper for mapping an `anyhow::Error` into `(code, message, details)` while preserving envelope payloads and schemas.

#### Scenario: MCP tool error envelopes remain unchanged after refactor
- **GIVEN** the MCP server exposes tools with stable behavior and stable `inputSchema` values
- **WHEN** the `anyhow::Error` -> `(code, message, details)` mapping is centralized behind a shared helper
- **THEN** MCP tool error envelopes remain identical at the payload level (`errors[0].code/message/details`)
