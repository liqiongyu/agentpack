# agentpack-cli (delta)

## ADDED Requirements

### Requirement: overlay path outputs a filesystem-safe directory
The system SHALL make `agentpack overlay path <module_id>` return an overlay directory that is filesystem-safe on the current platform.

#### Scenario: overlay path is Windows-safe
- **GIVEN** a module id `instructions:base`
- **WHEN** the user runs `agentpack overlay path instructions:base --scope global --json`
- **THEN** `data.overlay_dir` does not contain a path segment with `:` on Windows
