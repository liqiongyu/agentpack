## ADDED Requirements

### Requirement: Policy config errors are machine-actionable
When a `--json` invocation fails due to a missing/invalid/unsupported policy config, the system SHALL include additive, machine-actionable fields under `errors[0].details`:
- `reason_code: string` (stable, enum-like)
- `next_actions: string[]` (stable, enum-like action identifiers)

This requirement applies to these stable error codes:
- `E_POLICY_CONFIG_MISSING`
- `E_POLICY_CONFIG_INVALID`
- `E_POLICY_CONFIG_UNSUPPORTED_VERSION`

#### Scenario: policy config missing includes guidance fields
- **GIVEN** the config repo does not contain `repo/agentpack.org.yaml`
- **WHEN** the user runs `agentpack policy lock --json --yes`
- **THEN** stdout is valid JSON with `ok=false`
- **AND** `errors[0].code` is `E_POLICY_CONFIG_MISSING`
- **AND** `errors[0].details.reason_code` is present
- **AND** `errors[0].details.next_actions` is present

#### Scenario: policy config invalid includes guidance fields
- **GIVEN** the config repo contains an invalid `repo/agentpack.org.yaml`
- **WHEN** the user runs `agentpack policy lock --json --yes`
- **THEN** stdout is valid JSON with `ok=false`
- **AND** `errors[0].code` is `E_POLICY_CONFIG_INVALID`
- **AND** `errors[0].details.reason_code` is present
- **AND** `errors[0].details.next_actions` is present

#### Scenario: policy config unsupported version includes guidance fields
- **GIVEN** the config repo contains `repo/agentpack.org.yaml` with an unsupported `version`
- **WHEN** the user runs `agentpack policy lock --json --yes`
- **THEN** stdout is valid JSON with `ok=false`
- **AND** `errors[0].code` is `E_POLICY_CONFIG_UNSUPPORTED_VERSION`
- **AND** `errors[0].details.reason_code` is present
- **AND** `errors[0].details.next_actions` is present
