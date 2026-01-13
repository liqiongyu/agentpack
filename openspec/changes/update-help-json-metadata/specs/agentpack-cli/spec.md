# agentpack-cli (delta)

## MODIFIED Requirements

### Requirement: help --json is self-describing
The system SHALL provide `agentpack help --json` which emits machine-consumable documentation, including:
- a `commands` list describing available commands/subcommands, and
- `mutating_commands` listing mutating command IDs that require `--yes` in `--json` mode.

Each item in `data.commands[]` SHALL include:
- `id` (stable command id),
- `path[]` (command path segments),
- `mutating` (whether the base invocation mutates), and
- `supports_json` (whether the command supports `--json` output).

`data.commands[]` SHOULD include `args[]` describing command-specific arguments (excluding global args).

The output SHOULD also include `global_args[]` describing global flags/options.

#### Scenario: help --json returns command metadata
- **WHEN** the user runs `agentpack help --json`
- **THEN** stdout is valid JSON with `ok=true`
- **AND** `data.commands` exists
- **AND** `data.commands[0].supports_json` exists
- **AND** `data.global_args` exists
- **AND** `data.mutating_commands` exists
