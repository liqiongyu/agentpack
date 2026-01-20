## MODIFIED Requirements

### Requirement: Provide a policy pack lock command

The system SHALL provide a governance lock command:

`agentpack policy lock`

The command MUST read `repo/agentpack.org.yaml` and MUST write/update `repo/agentpack.org.lock.json`.

When `repo/agentpack.org.yaml` configures `supply_chain_policy.allowed_git_remotes` and `policy_pack.source` is a git source, `policy lock` MUST refuse to lock a policy pack from a non-allowlisted remote.

In `--json` mode, the command MUST require `--yes` for safety (otherwise return `E_CONFIRM_REQUIRED`).

#### Scenario: policy lock fails when policy pack remote is not allowlisted
- **GIVEN** `repo/agentpack.org.yaml` configures `supply_chain_policy.allowed_git_remotes=["github.com/your-org/"]`
- **AND** `repo/agentpack.org.yaml` configures `policy_pack.source` from a different git remote
- **WHEN** the user runs `agentpack policy lock --json --yes`
- **THEN** the command exits non-zero
- **AND** stdout is valid JSON with `ok=false`
- **AND** `errors[0].code` equals `E_POLICY_CONFIG_INVALID`
- **AND** `errors[0].details` includes the policy pack `remote` and the configured `allowed_git_remotes`
