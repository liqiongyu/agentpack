## ADDED Requirements

### Requirement: operator assets warnings remain consistent across CLI and MCP
The system SHALL centralize the operator assets checking logic used by CLI `status` and the MCP `status` tool so that warnings and suggested actions remain consistent over time.

#### Scenario: status operator assets warnings remain consistent
- **GIVEN** a repo/profile/target where operator assets are missing or outdated
- **WHEN** the user runs `agentpack status`
- **AND** an MCP client calls the `status` tool
- **THEN** both surfaces emit equivalent warnings and suggested remediation actions
