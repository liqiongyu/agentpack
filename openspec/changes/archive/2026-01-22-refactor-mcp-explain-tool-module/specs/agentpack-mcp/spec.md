## ADDED Requirements

### Requirement: MCP explain tool implementation is modularized without behavior change
The system SHALL allow the MCP `explain` tool implementation to be moved into a dedicated module file while preserving the tool schema, output envelopes, error behavior, and confirmation/guardrails semantics.

#### Scenario: MCP explain tool continues to behave the same after refactor
- **GIVEN** an MCP client that calls the `explain` tool with `kind=plan` and `kind=status`
- **WHEN** the implementation is refactored to a dedicated module file
- **THEN** the responses (envelope fields and `data` payload shape) remain unchanged
