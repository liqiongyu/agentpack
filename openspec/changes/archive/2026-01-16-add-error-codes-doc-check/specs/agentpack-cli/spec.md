## ADDED Requirements

### Requirement: ERROR_CODES.md stays consistent with emitted JSON error codes

The repository SHALL include an automated check that ensures `docs/ERROR_CODES.md` contains exactly the set of error codes that can be emitted as `errors[0].code` by the CLI in `--json` mode.

#### Scenario: docs registry matches emitted codes
- **WHEN** CI runs the consistency check
- **THEN** it passes when the sets match
- **AND** it fails with an actionable message when codes are missing or extra
