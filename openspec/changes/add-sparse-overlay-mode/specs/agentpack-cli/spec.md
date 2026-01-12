# agentpack-cli (delta)

## ADDED Requirements

### Requirement: overlay edit supports sparse overlays
The system SHALL support creating a sparse overlay via `agentpack overlay edit --sparse`, which creates the overlay directory and required metadata without copying the entire upstream tree.

#### Scenario: overlay edit --sparse creates metadata only
- **GIVEN** a module `<moduleId>` exists and resolves to an upstream root
- **WHEN** the user runs `agentpack overlay edit <moduleId> --scope global --sparse`
- **THEN** the overlay directory exists
- **AND** it contains `.agentpack/baseline.json`
- **AND** it contains `.agentpack/module_id`
- **AND** it does not contain copied upstream files by default

### Requirement: overlay edit supports materializing upstream files
The system SHALL support materializing upstream files into an overlay directory via `agentpack overlay edit --materialize` without overwriting existing overlay edits.

#### Scenario: overlay edit --materialize does not overwrite edits
- **GIVEN** an overlay directory exists with an edited file
- **WHEN** the user runs `agentpack overlay edit <moduleId> --materialize`
- **THEN** upstream files missing from the overlay are copied in
- **AND** existing overlay files are not overwritten
