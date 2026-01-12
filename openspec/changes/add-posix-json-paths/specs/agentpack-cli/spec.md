# agentpack-cli (delta)

## ADDED Requirements

### Requirement: JSON outputs include POSIX-style path fields
When a `--json` payload includes a filesystem path, the system SHALL also emit a POSIX-style companion field (using `/` separators) so cross-platform automation can parse paths consistently.

This change MUST be additive: existing fields remain unchanged in `schema_version=1`.

#### Scenario: plan --json includes POSIX path fields
- **WHEN** the user runs `agentpack plan --json`
- **THEN** each change item includes a POSIX-style path field (e.g. `path_posix`)
