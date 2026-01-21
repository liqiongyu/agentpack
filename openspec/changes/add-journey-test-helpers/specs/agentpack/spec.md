## ADDED Requirements

### Requirement: Journey tests use shared helpers

The repository SHALL provide shared helper utilities for journey tests to standardize running the CLI in a temporary environment, parsing `--json` output, and asserting stable error codes.

#### Scenario: Journey tests avoid duplicated boilerplate
- **GIVEN** the journey test suite exists
- **WHEN** multiple journey tests run `agentpack` commands in `--json` mode
- **THEN** they share common helper functions for running commands and parsing JSON output
