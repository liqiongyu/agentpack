---
## ADDED Requirements

### Requirement: CLI error formatting reuses shared UserError extraction helper
The system SHALL refactor CLI error formatting paths to reuse a shared helper for extracting embedded `UserError` values from an `anyhow::Error` chain while preserving output behavior.

#### Scenario: CLI error behavior remains unchanged after refactor
- **GIVEN** the CLI emits stable `--json` envelopes and stable human/TUI error formatting
- **WHEN** UserError extraction is centralized behind a shared helper
- **THEN** CLI output remains unchanged for both `UserError` and non-`UserError` failures
