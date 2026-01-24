## ADDED Requirements

### Requirement: IO write errors are machine-actionable
When a `--json` invocation fails due to a filesystem write error, the system SHALL include additive, machine-actionable fields under `errors[0].details`:
- `reason_code: string` (stable, enum-like)
- `next_actions: string[]` (stable, enum-like action identifiers)

This requirement applies to these stable error codes:
- `E_IO_PERMISSION_DENIED`
- `E_IO_INVALID_PATH`
- `E_IO_PATH_TOO_LONG`

#### Scenario: deploy --json includes guidance fields on IO failure
- **GIVEN** a target root that will fail on write (invalid path, read-only destination, or platform path limit)
- **WHEN** the user runs `agentpack deploy --apply --yes --json`
- **THEN** stdout is valid JSON with `ok=false`
- **AND** `errors[0].code` is one of the stable IO error codes above
- **AND** `errors[0].details.reason_code` is present
- **AND** `errors[0].details.next_actions` is present
