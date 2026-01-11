# agentpack-cli (delta)

## ADDED Requirements

### Requirement: help --json is self-describing
The system SHALL provide `agentpack help --json` which emits machine-consumable documentation, including:
- a `commands` list describing available commands/subcommands, and
- `mutating_commands` listing mutating command IDs that require `--yes` in `--json` mode.

#### Scenario: help --json returns commands and mutating_commands
- **WHEN** the user runs `agentpack help --json`
- **THEN** stdout is valid JSON with `ok=true`
- **AND** `data.commands` exists
- **AND** `data.mutating_commands` exists

### Requirement: schema command documents JSON output contract
The system SHALL provide `agentpack schema` which documents:
- the JSON envelope schema, and
- the minimum expected `data` fields for key read commands (at least: `plan`, `diff`, `preview`, `status`).

#### Scenario: schema --json returns envelope schema
- **WHEN** the user runs `agentpack schema --json`
- **THEN** stdout is valid JSON with `ok=true`
- **AND** `data.envelope` exists
