# agentpack-cli (delta)

## ADDED Requirements

### Requirement: Mutating command set is centrally maintained
The system SHALL centrally define the set of “mutating” operations/command IDs (writes to disk or git) and use that single source of truth for:
- enforcing `--json` + `--yes` guardrails, and
- self-description output (e.g., `help --json`) when present.

#### Scenario: lock --json without --yes is refused
- **WHEN** the user runs `agentpack lock --json` without `--yes`
- **THEN** the command exits non-zero
- **AND** stdout is valid JSON with `ok=false`
- **AND** `errors[0].code` equals `E_CONFIRM_REQUIRED`
