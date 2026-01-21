## ADDED Requirements

### Requirement: Repository provides a reproducible CLI demo asset

The repository SHALL include a reproducible CLI demo recording (script + generated asset) that demonstrates the core workflow (update → preview/diff → deploy/apply → status → rollback) in an isolated environment.

#### Scenario: Maintainer can regenerate demo asset
- **GIVEN** a maintainer wants to refresh the recorded demo
- **WHEN** they run the documented regeneration command
- **THEN** the demo asset is regenerated deterministically from the tape/script
