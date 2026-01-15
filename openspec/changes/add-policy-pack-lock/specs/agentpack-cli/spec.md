# agentpack-cli (delta)

## ADDED Requirements

### Requirement: Provide a policy pack lock command
The system SHALL provide a governance lock command:

`agentpack policy lock`

The command MUST read `repo/agentpack.org.yaml` and MUST write/update `repo/agentpack.org.lock.json`.

In `--json` mode, the command MUST require `--yes` for safety (otherwise return `E_CONFIRM_REQUIRED`).

#### Scenario: policy lock writes a deterministic lockfile
- **GIVEN** `repo/agentpack.org.yaml` references a policy pack source
- **WHEN** the user runs `agentpack policy lock` twice
- **THEN** `repo/agentpack.org.lock.json` content is identical across runs

#### Scenario: policy lock --json without --yes is refused
- **GIVEN** `repo/agentpack.org.yaml` references a policy pack source
- **WHEN** the user runs `agentpack policy lock --json` without `--yes`
- **THEN** the command exits non-zero
- **AND** stdout is valid JSON with `ok=false`
- **AND** `errors[0].code` equals `E_CONFIRM_REQUIRED`

### Requirement: policy lint can validate policy pack pinning
When a policy pack is configured, `agentpack policy lint` SHALL avoid network access and SHALL validate that the policy pack is pinned via the governance lockfile.

#### Scenario: lint fails when a policy pack is configured but no lock exists
- **WHEN** the user runs `agentpack policy lint --json`
- **AND** `repo/agentpack.org.yaml` configures a policy pack
- **AND** `repo/agentpack.org.lock.json` is missing
- **THEN** the command exits non-zero
- **AND** stdout is valid JSON with `ok=false`
- **AND** `errors[0].code` equals `E_POLICY_VIOLATIONS`
- **AND** the output includes machine-readable details about the missing lock
