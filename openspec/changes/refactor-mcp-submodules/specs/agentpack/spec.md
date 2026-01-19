# agentpack Delta

## ADDED Requirements

### Requirement: MCP implementation is modularized

The repository SHALL keep the MCP implementation split into focused submodules (`server`, `tools`, `confirm`) to reduce coupling and improve maintainability, without changing user-facing behavior.

#### Scenario: MCP refactor preserves behavior
- **WHEN** the MCP implementation is reorganized
- **THEN** the existing MCP integration tests continue to pass
