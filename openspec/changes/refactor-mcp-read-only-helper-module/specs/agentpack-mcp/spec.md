## ADDED Requirements

### Requirement: MCP read-only helper is modularized
The system SHALL keep the MCP read-only in-process helper implemented in a dedicated module file so `src/mcp/tools.rs` stays focused on routing while preserving MCP tool behavior and JSON envelopes.

#### Scenario: MCP plan/diff envelopes remain unchanged after helper refactor
- **GIVEN** the MCP server exposes read-only tools that return Agentpack JSON envelopes
- **WHEN** the read-only helper code is refactored into a dedicated module
- **THEN** the tools return the same JSON envelope structure and `ok`/error semantics as before
