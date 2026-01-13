## ADDED Requirements

### Requirement: status emits actionable next_actions (additive)
When invoked as `agentpack status --json`, the system SHALL include an additive `data.next_actions` field that suggests follow-up commands.

`data.next_actions` SHALL be a list of command strings (`string[]`), each describing a safe follow-up command the user/agent can run.

This change MUST be additive for `schema_version=1` (no rename/remove of existing fields).

#### Scenario: status suggests bootstrap when operator assets are missing
- **GIVEN** operator assets are missing for the selected target/scope
- **WHEN** the user runs `agentpack status --json`
- **THEN** `data.next_actions[]` includes an action that runs `agentpack bootstrap`

#### Scenario: status suggests deploy --apply when desired-state drift exists
- **GIVEN** `status` detects `modified` or `missing` drift
- **WHEN** the user runs `agentpack status --json`
- **THEN** `data.next_actions[]` includes an action that runs `agentpack deploy --apply`

## MODIFIED Requirements

### Requirement: schema command documents JSON output contract
The `agentpack schema --json` payload SHALL document `next_actions` as an additive `status` data field.

#### Scenario: schema lists status next_actions field
- **WHEN** the user runs `agentpack schema --json`
- **THEN** the schema output documents `status` data fields including `next_actions`
