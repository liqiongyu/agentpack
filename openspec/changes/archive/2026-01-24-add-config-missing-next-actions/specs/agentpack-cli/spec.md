## ADDED Requirements

### Requirement: Config missing errors are machine-actionable
When a `--json` invocation fails due to a missing config manifest, the system SHALL include additive, machine-actionable fields under `errors[0].details`:
- `reason_code: string` (stable, enum-like)
- `next_actions: string[]` (stable, enum-like action identifiers)

This requirement applies to the stable error code `E_CONFIG_MISSING`.

#### Scenario: config missing includes guidance fields
- **GIVEN** the config repo does not contain `repo/agentpack.yaml`
- **WHEN** the user runs `agentpack plan --json`
- **THEN** stdout is valid JSON with `ok=false`
- **AND** `errors[0].code` is `E_CONFIG_MISSING`
- **AND** `errors[0].details.reason_code` is present
- **AND** `errors[0].details.next_actions` is present
