# agentpack-cli (delta)

## ADDED Requirements

### Requirement: JSON envelope includes command metadata
When invoked with `--json`, the system SHALL include `command_id` and `command_path` as top-level envelope fields:
- `command_id`: stable command id (space-separated), aligned with `help --json` command ids and mutating guardrails
- `command_path`: tokenized command id (array of strings)

#### Scenario: error envelope includes subcommand id
- **WHEN** the user runs `agentpack remote set https://example.invalid/repo.git --json` without `--yes`
- **THEN** stdout is valid JSON with `ok=false`
- **AND** `command_id == "remote set"`
- **AND** `command_path == ["remote", "set"]`

#### Scenario: error envelope includes mutating variant id
- **WHEN** the user runs `agentpack doctor --fix --json` without `--yes`
- **THEN** stdout is valid JSON with `ok=false`
- **AND** `command_id == "doctor --fix"`
- **AND** `command_path == ["doctor", "--fix"]`
