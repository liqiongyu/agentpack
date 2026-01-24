## ADDED Requirements

### Requirement: Policy violation errors are machine-actionable
When a `--json` invocation fails due to policy violations, the system SHALL include additive, machine-actionable fields under `errors[0].details`:
- `reason_code: string` (stable, enum-like)
- `next_actions: string[]` (stable, enum-like action identifiers)

This requirement applies to the stable error code `E_POLICY_VIOLATIONS`.

#### Scenario: policy violations include guidance fields
- **GIVEN** `policy lint` detects one or more violations
- **WHEN** the user runs `agentpack policy lint --json`
- **THEN** stdout is valid JSON with `ok=false`
- **AND** `errors[0].code` is `E_POLICY_VIOLATIONS`
- **AND** `errors[0].details.reason_code` is present
- **AND** `errors[0].details.next_actions` is present
