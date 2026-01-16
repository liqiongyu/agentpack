## ADDED Requirements

### Requirement: import dry-run reports conflicts before apply

When `agentpack import` is run in dry-run mode, the system SHALL report conflicts that would prevent a safe apply.

At minimum, the system SHALL detect and report:
- destination path conflicts (a planned destination path already exists inside the config repo)
- module id collisions within the scan (multiple candidates map to the same `module_id`)

The conflict report MUST be deterministic and machine-readable in `--json` mode (additive fields are allowed).

#### Scenario: import --json reports destination conflicts during dry-run
- **GIVEN** a config repo where an import destination path already exists (e.g. `repo/modules/prompts/imported/prompt1.md`)
- **WHEN** the user runs `agentpack import --json`
- **THEN** stdout is valid JSON with `ok=true`
- **AND** `command` equals `"import"`
- **AND** `data.plan` includes the conflicting item
- **AND** the output includes a machine-readable indication that the destination already exists

#### Scenario: import --apply fails safely on destination conflicts
- **GIVEN** a config repo where an import destination path already exists
- **WHEN** the user runs `agentpack import --apply --yes --json`
- **THEN** the command exits non-zero
- **AND** stdout is valid JSON with `ok=false`
- **AND** `errors[0].code` equals `E_IMPORT_CONFLICT`
