## ADDED Requirements

### Requirement: Desired-state conflict errors are machine-actionable
When a `--json` invocation fails due to conflicting desired outputs, the system SHALL include additive, machine-actionable fields under `errors[0].details`:
- `reason_code: string` (stable, enum-like)
- `next_actions: string[]` (stable, enum-like action identifiers)

This requirement applies to the stable error code `E_DESIRED_STATE_CONFLICT`.

#### Scenario: desired-state conflict includes guidance fields
- **GIVEN** multiple modules produce different content for the same `(target, path)`
- **WHEN** the user runs a command that produces `E_DESIRED_STATE_CONFLICT` in `--json` mode
- **THEN** stdout is valid JSON with `ok=false`
- **AND** `errors[0].details.reason_code` is present
- **AND** `errors[0].details.next_actions` is present
