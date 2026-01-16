# agentpack (delta)

## ADDED Requirements

### Requirement: Conformance tests cover Windows path and permission edge cases

The repository SHALL include conformance tests that exercise Windows path and permission boundary cases, including:
- invalid path characters
- path too long
- read-only destination files
- permission denied writes

#### Scenario: conformance suite validates Windows edge cases
- **GIVEN** the test suite is running on Windows
- **WHEN** conformance tests execute
- **THEN** failures return stable JSON error codes with actionable messages
