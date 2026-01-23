---
## ADDED Requirements

### Requirement: CLI JSON error envelopes reuse a shared anyhow error-parts helper
The system SHALL refactor CLI JSON error envelope construction to reuse a shared helper for mapping an `anyhow::Error` into `(code, message, details)` while preserving envelope payloads and schemas.

#### Scenario: CLI JSON error envelopes remain unchanged after refactor
- **GIVEN** the CLI emits stable `--json` envelopes
- **WHEN** the `anyhow::Error` -> `(code, message, details)` mapping is centralized behind a shared helper
- **THEN** CLI JSON error envelopes remain identical at the payload level (`errors[0].code/message/details`)
