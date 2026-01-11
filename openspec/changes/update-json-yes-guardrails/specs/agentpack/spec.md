# agentpack (delta)

## ADDED Requirements

### Requirement: JSON-mode write confirmation
When invoked with `--json`, any command that performs writes (filesystem or git) MUST require an explicit `--yes` confirmation. If `--yes` is missing, the system MUST return a JSON error with a stable code `E_CONFIRM_REQUIRED` and MUST NOT perform the write.

#### Scenario: add --json without --yes is refused
- **WHEN** the user runs `agentpack add ... --json` without `--yes`
- **THEN** the command exits non-zero
- **AND** stdout is valid JSON with `ok=false`
- **AND** `errors[0].code` equals `E_CONFIRM_REQUIRED`
