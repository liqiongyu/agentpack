# agentpack (delta)

## ADDED Requirements

### Requirement: Support an opt-in governance config and lockfile
The system SHALL support an opt-in governance configuration file at:

`repo/agentpack.org.yaml`

The system SHALL support an opt-in governance lockfile at:

`repo/agentpack.org.lock.json`

Core commands (`plan`, `diff`, `deploy`, etc.) MUST NOT read the governance config or lockfile.

The governance config MAY reference a policy pack source and the system SHALL be able to pin that source via the governance lockfile for auditability.

#### Scenario: core commands ignore governance config
- **GIVEN** `repo/agentpack.org.yaml` exists
- **WHEN** the user runs `agentpack plan`
- **THEN** core behavior is unchanged (no governance config is read)
