## ADDED Requirements

### Requirement: TTY-required refusals are machine-actionable
When a `--json` invocation is refused because a command requires a TTY (i.e., `E_TTY_REQUIRED`), the system SHALL include additive, machine-actionable fields under `errors[0].details`:
- `reason_code: string` (stable, enum-like)
- `next_actions: string[]` (stable, enum-like action identifiers)

#### Scenario: init --guided --json without a TTY includes refusal details
- **GIVEN** stdin or stdout is not a terminal
- **WHEN** the user runs `agentpack init --guided --json`
- **THEN** the command exits non-zero
- **AND** stdout is valid JSON with `ok=false`
- **AND** `errors[0].code` equals `E_TTY_REQUIRED`
- **AND** `errors[0].details.reason_code` is present
- **AND** `errors[0].details.next_actions` is present
