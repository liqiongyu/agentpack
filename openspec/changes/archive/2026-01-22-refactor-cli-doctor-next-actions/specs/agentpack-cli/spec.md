## ADDED Requirements

### Requirement: doctor next_actions ordering remains consistent
The system SHALL reuse the shared `next_actions` ordering helper for CLI `doctor` so that ordering behavior stays consistent across commands and interfaces over time.

#### Scenario: doctor next_actions remain consistently ordered
- **GIVEN** a `doctor` report that emits multiple `next_actions`
- **WHEN** the user runs `agentpack doctor --json`
- **THEN** `data.next_actions` are ordered consistently with the shared ordering helper used by other commands
