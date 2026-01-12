## ADDED Requirements

### Requirement: overlay rebase updates overlays against upstream changes
The system SHALL provide `agentpack overlay rebase <module_id>` to update an existing overlay against the current upstream module content.

The command SHALL:
- use `<overlay_dir>/.agentpack/baseline.json` as the merge base,
- merge upstream changes into overlay edits using a 3-way merge, and
- update overlay baseline metadata after a successful rebase.

#### Scenario: rebase updates an unmodified file copy
- **GIVEN** an overlay contains an unmodified copy of an upstream file (identical to the baseline)
- **AND** the upstream file changes
- **WHEN** the user runs `agentpack overlay rebase <moduleId> --scope global`
- **THEN** the overlay file is updated so it no longer pins the old upstream content

#### Scenario: rebase conflict yields stable error code
- **GIVEN** an overlay edits a file that is also changed upstream since the baseline
- **AND** the changes overlap and cannot be merged cleanly
- **WHEN** the user runs `agentpack overlay rebase <moduleId> --json --yes`
- **THEN** the command exits non-zero
- **AND** stdout is valid JSON with `ok=false`
- **AND** `errors[0].code` equals `E_OVERLAY_REBASE_CONFLICT`
