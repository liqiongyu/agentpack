## ADDED Requirements

### Requirement: MCP envelope helpers are modularized
The system SHALL keep MCP envelope and tool-result helper functions implemented in a dedicated module file so `src/mcp/tools.rs` stays focused on routing and schemas while preserving MCP tool behavior and JSON envelopes.

#### Scenario: MCP tool envelopes remain unchanged after helper refactor
- **GIVEN** MCP tool implementations that reuse shared envelope helper functions
- **WHEN** the helpers are refactored into a dedicated module for maintainability
- **THEN** tool results still reuse the Agentpack JSON envelope unchanged
