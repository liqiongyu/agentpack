## ADDED Requirements

### Requirement: Lockfile errors are machine-actionable
When a `--json` invocation fails due to a missing/invalid/unsupported lockfile, the system SHALL include additive, machine-actionable fields under `errors[0].details`:
- `reason_code: string` (stable, enum-like)
- `next_actions: string[]` (stable, enum-like action identifiers)

This requirement applies to these stable error codes:
- `E_LOCKFILE_MISSING`
- `E_LOCKFILE_INVALID`
- `E_LOCKFILE_UNSUPPORTED_VERSION`

#### Scenario: lockfile missing includes guidance fields
- **GIVEN** the config repo does not contain `repo/agentpack.lock.json`
- **WHEN** the user runs `agentpack fetch --json --yes`
- **THEN** stdout is valid JSON with `ok=false`
- **AND** `errors[0].code` is `E_LOCKFILE_MISSING`
- **AND** `errors[0].details.reason_code` is present
- **AND** `errors[0].details.next_actions` is present

#### Scenario: lockfile invalid includes guidance fields
- **GIVEN** the config repo contains an invalid `repo/agentpack.lock.json`
- **WHEN** the user runs `agentpack fetch --json --yes`
- **THEN** stdout is valid JSON with `ok=false`
- **AND** `errors[0].code` is `E_LOCKFILE_INVALID`
- **AND** `errors[0].details.reason_code` is present
- **AND** `errors[0].details.next_actions` is present

#### Scenario: lockfile unsupported version includes guidance fields
- **GIVEN** the config repo contains `repo/agentpack.lock.json` with an unsupported `version`
- **WHEN** the user runs `agentpack fetch --json --yes`
- **THEN** stdout is valid JSON with `ok=false`
- **AND** `errors[0].code` is `E_LOCKFILE_UNSUPPORTED_VERSION`
- **AND** `errors[0].details.reason_code` is present
- **AND** `errors[0].details.next_actions` is present
