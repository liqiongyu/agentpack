# agentpack (delta)

## ADDED Requirements

### Requirement: Optional durability mode for atomic writes
When `AGENTPACK_FSYNC` is enabled, the system SHALL increase durability of atomic writes by syncing file contents before persist and (where supported) syncing the parent directory after the atomic replace.

#### Scenario: durability mode is enabled for atomic writes
- **GIVEN** `AGENTPACK_FSYNC=1` is set in the environment
- **WHEN** the system writes a file via an atomic write path
- **THEN** it syncs file contents to disk before the atomic replace
- **AND** where supported, it syncs the parent directory after the atomic replace
