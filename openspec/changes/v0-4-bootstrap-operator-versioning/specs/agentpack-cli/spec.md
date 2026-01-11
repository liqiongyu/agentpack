# agentpack-cli (delta)

## ADDED Requirements

### Requirement: Bootstrap operator assets are version-stamped
The system SHALL stamp bootstrap-installed operator assets with an `agentpack_version: x.y.z` marker (frontmatter or comment) matching the running `agentpack` version.

#### Scenario: bootstrap-installed assets include agentpack_version
- **WHEN** the user runs `agentpack bootstrap`
- **THEN** the installed operator assets contain an `agentpack_version` marker

### Requirement: status warns when operator assets are missing or outdated
The system SHALL warn when operator assets for the selected target are missing or have an `agentpack_version` that does not match the running `agentpack` version.

#### Scenario: status warns for missing assets
- **GIVEN** operator assets are not installed for the selected target
- **WHEN** the user runs `agentpack status --json`
- **THEN** `warnings[]` includes a message recommending `agentpack bootstrap`

#### Scenario: status warns for outdated assets
- **GIVEN** operator assets exist but have `agentpack_version` that differs from `agentpack --version`
- **WHEN** the user runs `agentpack status --json`
- **THEN** `warnings[]` includes a message recommending `agentpack bootstrap`
