## ADDED Requirements

### Requirement: Target unsupported errors are machine-actionable
When a `--json` invocation fails due to an unsupported target selection, the system SHALL include additive, machine-actionable fields under `errors[0].details`:
- `reason_code: string` (stable, enum-like)
- `next_actions: string[]` (stable, enum-like action identifiers)

This requirement applies to the stable error code `E_TARGET_UNSUPPORTED`.

#### Scenario: target unsupported includes guidance fields
- **WHEN** the user runs `agentpack plan --target nope --json`
- **THEN** stdout is valid JSON with `ok=false`
- **AND** `errors[0].code` is `E_TARGET_UNSUPPORTED`
- **AND** `errors[0].details.reason_code` is present
- **AND** `errors[0].details.next_actions` is present
