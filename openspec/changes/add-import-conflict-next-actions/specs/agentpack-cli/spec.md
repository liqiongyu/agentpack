## ADDED Requirements

### Requirement: Import-conflict refusals are machine-actionable
When a `--json` invocation is refused because an import destination already exists (i.e., `E_IMPORT_CONFLICT`), the system SHALL include additive, machine-actionable fields under `errors[0].details`:
- `reason_code: string` (stable, enum-like)
- `next_actions: string[]` (stable, enum-like action identifiers)

#### Scenario: import --apply --json fails with refusal details on destination conflicts
- **GIVEN** a config repo where an import destination path already exists
- **WHEN** the user runs `agentpack import --apply --yes --json`
- **THEN** the command exits non-zero
- **AND** stdout is valid JSON with `ok=false`
- **AND** `errors[0].code` equals `E_IMPORT_CONFLICT`
- **AND** `errors[0].details.reason_code` is present
- **AND** `errors[0].details.next_actions` is present
