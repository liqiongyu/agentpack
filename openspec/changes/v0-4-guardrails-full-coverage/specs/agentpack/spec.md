# agentpack (delta)

## ADDED Requirements

### Requirement: JSON-mode write confirmation covers all write-capable commands
When invoked with `--json`, any command that performs writes (filesystem or git) MUST require an explicit `--yes` confirmation. If `--yes` is missing, the system MUST return a JSON error with a stable code `E_CONFIRM_REQUIRED` and MUST NOT perform the write.

This includes (at minimum): `init`, `lock`, `fetch`, `overlay edit`, `remote set`, `sync`, `record`, and `rollback` (in addition to existing covered commands like `add/remove/update/deploy/bootstrap/doctor --fix/evolve propose`).

#### Scenario: init --json without --yes is refused
- **WHEN** the user runs `agentpack init --json` without `--yes`
- **THEN** the command exits non-zero
- **AND** stdout is valid JSON with `ok=false`
- **AND** `errors[0].code` equals `E_CONFIRM_REQUIRED`

#### Scenario: overlay edit --json without --yes is refused
- **WHEN** the user runs `agentpack overlay edit <moduleId> --json` without `--yes`
- **THEN** the command exits non-zero
- **AND** stdout is valid JSON with `ok=false`
- **AND** `errors[0].code` equals `E_CONFIRM_REQUIRED`
