# agentpack-cli (delta)

## MODIFIED Requirements

### Requirement: help --json is self-describing
The system SHALL provide `agentpack help --json` which emits machine-consumable documentation, including:
- a `commands` list describing available commands/subcommands,
- `mutating_commands` listing mutating command IDs that require `--yes` in `--json` mode, and
- `targets` listing compiled-in target adapters.

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
- **AND** `data.targets` exists

## ADDED Requirements

### Requirement: Target adapters can be feature-gated at build time
The system SHALL support building agentpack with a subset of target adapters enabled via Cargo features:
- `target-codex`
- `target-claude-code`
- `target-cursor`
- `target-vscode`

The default feature set SHOULD include all built-in targets (to preserve the default user experience).

If a user selects a target that is not compiled into the running binary, the CLI MUST treat it as unsupported.

#### Scenario: selecting a non-compiled target fails with E_TARGET_UNSUPPORTED
- **GIVEN** an agentpack binary built without `target-cursor`
- **WHEN** the user runs `agentpack plan --target cursor --json`
- **THEN** stdout is valid JSON with `ok=false`
- **AND** `errors[0].code` equals `E_TARGET_UNSUPPORTED`
