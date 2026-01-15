## ADDED Requirements

### Requirement: Governance config supports distribution_policy
The system SHALL support an optional “org distribution policy” section in the governance config file:

`repo/agentpack.org.yaml`

The distribution policy MUST be scoped to governance commands (`agentpack policy ...`) and MUST NOT change the behavior of core commands (`plan`, `diff`, `deploy`, etc.).

The distribution policy MAY declare requirements over the repo’s manifest (`repo/agentpack.yaml`), including:
- required targets
- required modules (enabled)

#### Scenario: core commands ignore org distribution policy
- **GIVEN** `repo/agentpack.org.yaml` configures `distribution_policy`
- **WHEN** the user runs `agentpack plan`
- **THEN** core behavior is unchanged (no governance config is read)
