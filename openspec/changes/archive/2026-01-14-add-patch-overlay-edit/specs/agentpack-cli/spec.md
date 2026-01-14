# agentpack-cli (delta)

## ADDED Requirements

### Requirement: overlay edit supports creating patch overlays
The system SHALL support creating a patch overlay skeleton via `agentpack overlay edit <moduleId> --kind patch`, which creates the overlay directory and required patch-overlay metadata without copying upstream files.

When invoked with `--kind patch`, the command SHALL:
- ensure the overlay directory exists,
- ensure `.agentpack/baseline.json` exists,
- ensure `.agentpack/module_id` exists,
- write `.agentpack/overlay.json` with `overlay_kind=patch`, and
- ensure `.agentpack/patches/` exists.

When invoked with `--kind patch`, the command MUST NOT copy upstream files into the overlay directory.

#### Scenario: overlay edit --kind patch creates patch overlay skeleton
- **GIVEN** a module `<moduleId>` exists and resolves to an upstream root
- **WHEN** the user runs `agentpack overlay edit <moduleId> --scope global --kind patch`
- **THEN** the overlay directory exists
- **AND** it contains `.agentpack/baseline.json`
- **AND** it contains `.agentpack/module_id`
- **AND** it contains `.agentpack/overlay.json` with `overlay_kind=patch`
- **AND** it contains `.agentpack/patches/`
- **AND** it does not contain copied upstream files by default
