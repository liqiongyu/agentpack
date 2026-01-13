# agentpack-cli (delta)

## ADDED Requirements

### Requirement: init can optionally initialize a git repo
The system SHALL support `agentpack init --git` to initialize the created repo directory as a git repository and to write/update a minimal `.gitignore` file.

#### Scenario: init --git creates a git-backed repo skeleton
- **GIVEN** a fresh machine state (empty `AGENTPACK_HOME`)
- **WHEN** the user runs `agentpack init --git`
- **THEN** the repo directory contains a `.git/` directory
- **AND** the repo directory contains `.gitignore` that ignores `.agentpack.manifest.json`
