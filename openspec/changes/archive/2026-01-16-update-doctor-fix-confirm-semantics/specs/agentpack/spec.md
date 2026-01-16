# agentpack (delta)

## ADDED Requirements

### Requirement: doctor --fix requires --yes in --json mode

When invoked with `--json`, `agentpack doctor --fix` MUST require explicit `--yes`. If `--yes` is missing, it MUST return `E_CONFIRM_REQUIRED` and MUST NOT perform any writes.

#### Scenario: doctor --fix --json without --yes is refused
- **WHEN** the user runs `agentpack doctor --fix --json` without `--yes`
- **THEN** the command exits non-zero
- **AND** stdout is valid JSON with `ok=false`
- **AND** `errors[0].code` equals `E_CONFIRM_REQUIRED`
