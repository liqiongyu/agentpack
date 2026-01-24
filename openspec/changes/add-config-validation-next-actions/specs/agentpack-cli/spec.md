## ADDED Requirements

### Requirement: Config validation errors are machine-actionable
When a `--json` invocation fails due to an invalid/unsupported config manifest, the system SHALL include additive, machine-actionable fields under `errors[0].details`:
- `reason_code: string` (stable, enum-like)
- `next_actions: string[]` (stable, enum-like action identifiers)

This requirement applies to these stable error codes:
- `E_CONFIG_INVALID`
- `E_CONFIG_UNSUPPORTED_VERSION`

#### Scenario: config invalid includes guidance fields
- **GIVEN** the config repo contains an invalid `repo/agentpack.yaml`
- **WHEN** the user runs `agentpack plan --json`
- **THEN** stdout is valid JSON with `ok=false`
- **AND** `errors[0].code` is `E_CONFIG_INVALID`
- **AND** `errors[0].details.reason_code` is present
- **AND** `errors[0].details.next_actions` is present

#### Scenario: config unsupported version includes guidance fields
- **GIVEN** the config repo contains `repo/agentpack.yaml` with an unsupported `version`
- **WHEN** the user runs `agentpack plan --json`
- **THEN** stdout is valid JSON with `ok=false`
- **AND** `errors[0].code` is `E_CONFIG_UNSUPPORTED_VERSION`
- **AND** `errors[0].details.reason_code` is present
- **AND** `errors[0].details.next_actions` is present
