# agentpack-cli (delta)

## ADDED Requirements

### Requirement: Provide a policy lint command (opt-in governance)
The system SHALL provide an opt-in governance namespace as a CLI subcommand:

`agentpack policy ...`

The system SHALL provide a read-only lint command:

`agentpack policy lint`

The command MUST NOT perform writes and MUST NOT change behavior of existing core commands.

#### Scenario: policy lint succeeds when there are no violations
- **WHEN** the user runs `agentpack policy lint --json`
- **AND** there are no policy violations detected
- **THEN** stdout is valid JSON with `ok=true`

#### Scenario: policy lint fails with a stable error code when violations exist
- **WHEN** the user runs `agentpack policy lint --json`
- **AND** there is at least one policy violation detected
- **THEN** the command exits non-zero
- **AND** stdout is valid JSON with `ok=false`
- **AND** `errors[0].code` equals a stable policy-violation code (e.g., `E_POLICY_VIOLATIONS`)
- **AND** the output includes machine-readable details about the violations
