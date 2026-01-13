# agentpack-cli (delta)

## ADDED Requirements

### Requirement: init can optionally bootstrap operator assets
The system SHALL support `agentpack init --bootstrap` to install operator assets immediately after initializing the repo (equivalent to running `agentpack bootstrap`).

#### Scenario: init --bootstrap installs operator assets
- **GIVEN** a fresh machine state (empty `AGENTPACK_HOME`)
- **WHEN** the user runs `agentpack init --bootstrap`
- **THEN** operator assets are installed into configured target locations
