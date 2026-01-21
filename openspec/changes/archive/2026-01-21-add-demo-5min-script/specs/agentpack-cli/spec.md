## ADDED Requirements

### Requirement: Repository provides a non-destructive 5-minute demo

The repository SHALL provide a “5-minute demo” script and tutorial that runs Agentpack with temporary `HOME` and `AGENTPACK_HOME`, and demonstrates `doctor --json` plus `preview --diff --json` without writing to the user’s real environment.

#### Scenario: New user can run a safe demo
- **GIVEN** a user clones the repo
- **WHEN** they run the demo script
- **THEN** it exits successfully and prints a plan/diff in JSON mode
- **AND** it does not write to the user’s real home directory
