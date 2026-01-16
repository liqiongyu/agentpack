# agentpack-cli (delta)

## ADDED Requirements

### Requirement: import command produces an import plan
The system SHALL provide a new CLI command `agentpack import` that scans existing assets and produces an import plan.

The command SHALL be read-only by default (no writes).

#### Scenario: import --json is parseable and includes a plan
- **WHEN** the user runs `agentpack import --json`
- **THEN** stdout is valid JSON with `ok=true`
- **AND** `command` equals `"import"`
- **AND** `data.plan` exists

### Requirement: import apply writes only to the config repo
When invoked with `--apply`, `agentpack import` SHALL write imported assets into the config repo as `local_path` modules and SHALL update `agentpack.yaml` accordingly.

The command SHALL NOT write to target roots (e.g. `~/.codex`, `~/.claude`) as part of the import operation.

In `--json` mode, `agentpack import --apply` MUST require an explicit `--yes` confirmation; if `--yes` is missing, the system MUST return `E_CONFIRM_REQUIRED` and MUST NOT perform writes.

#### Scenario: import --apply --json without --yes is refused
- **WHEN** the user runs `agentpack import --apply --json` without `--yes`
- **THEN** the command exits non-zero
- **AND** stdout is valid JSON with `ok=false`
- **AND** `errors[0].code` equals `E_CONFIRM_REQUIRED`

### Requirement: import supports a home root override for deterministic tests
The system SHALL support `agentpack import --home-root <path>` to override the home directory used for scanning user-scope assets.

#### Scenario: import reads from home-root instead of the real home
- **GIVEN** a temporary home directory containing Codex assets under `.codex/`
- **WHEN** the user runs `agentpack import --home-root <tmp> --json`
- **THEN** `data.plan` includes items sourced from `<tmp>/.codex/...`
