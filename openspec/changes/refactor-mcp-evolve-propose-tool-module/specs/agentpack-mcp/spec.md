## ADDED Requirements

### Requirement: MCP evolve_propose tool implementation is modularized without behavior change
The system SHALL allow the MCP `evolve_propose` tool implementation to be moved into a dedicated module file while preserving the tool schema, output envelopes, error behavior, and confirmation/guardrails semantics.

#### Scenario: MCP evolve_propose tool continues to behave the same after refactor
- **GIVEN** an MCP client that calls the `evolve_propose` tool
- **WHEN** the implementation is refactored to a dedicated module file
- **THEN** the responses (envelope fields and `data` payload shape) remain unchanged
