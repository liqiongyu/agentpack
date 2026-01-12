# agentpack (delta)

## ADDED Requirements

### Requirement: module_id is mapped to a filesystem-safe key
The system SHALL derive a stable, filesystem-safe `module_fs_key` from `module_id` and SHALL use `module_fs_key` when creating filesystem paths for module-scoped storage (e.g., overlays and cache/store directories).

#### Scenario: Windows-safe overlay directory
- **GIVEN** a module id `instructions:base`
- **WHEN** the system computes the overlay directory for that module
- **THEN** the directory path does not require using `:` as a path component

### Requirement: Legacy overlay and store paths remain usable
The system SHALL preserve backwards compatibility by preferring legacy (pre-v0.5) overlay and store paths when they already exist on disk.

#### Scenario: existing legacy overlay is used
- **GIVEN** a legacy overlay directory exists for a module id
- **WHEN** the system renders the module
- **THEN** it applies the legacy overlay directory
