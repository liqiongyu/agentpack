## ADDED Requirements

### Requirement: Adopt-confirm refusals are machine-actionable
When a `--json` invocation is refused due to missing explicit adopt confirmation (i.e., `E_ADOPT_CONFIRM_REQUIRED`), the system SHALL include additive, machine-actionable fields under `errors[0].details`:
- `reason_code: string` (stable, enum-like)
- `next_actions: string[]` (stable, enum-like action identifiers)

#### Scenario: deploy --apply --json without --adopt includes refusal details
- **GIVEN** the deploy plan contains at least one `adopt_update`
- **WHEN** the user runs `agentpack deploy --apply --json --yes` without `--adopt`
- **THEN** the command exits non-zero
- **AND** stdout is valid JSON with `ok=false`
- **AND** `errors[0].code` equals `E_ADOPT_CONFIRM_REQUIRED`
- **AND** `errors[0].details.reason_code` is present
- **AND** `errors[0].details.next_actions` is present
