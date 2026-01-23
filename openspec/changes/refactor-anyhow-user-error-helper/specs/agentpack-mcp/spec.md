---
## ADDED Requirements

### Requirement: MCP tool error envelopes reuse shared UserError extraction helper
The system SHALL refactor MCP tool error envelope construction to reuse a shared helper for extracting embedded `UserError` values from an `anyhow::Error` chain while preserving envelope payloads and schemas.

#### Scenario: MCP tool error envelopes remain unchanged after refactor
- **GIVEN** the MCP server exposes tools with stable behavior and stable `inputSchema` values
- **WHEN** UserError extraction is centralized behind a shared helper
- **THEN** MCP tool error envelopes remain identical at the payload level (`errors[0].code/message/details`)
