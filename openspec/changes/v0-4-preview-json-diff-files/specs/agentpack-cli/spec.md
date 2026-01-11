# agentpack-cli (delta)

## ADDED Requirements

### Requirement: preview --json --diff includes structured per-file diffs
When invoked as `agentpack preview --json --diff`, the system SHALL include a structured diff payload under `data.diff`:
- `summary` (counts)
- `files[]` with, at minimum: `target`, `root`, `path`, `op`, `before_hash`, `after_hash`

`unified` diffs are optional and MAY be omitted; if omitted due to size limits, the system SHOULD add a warning.

#### Scenario: preview --json --diff includes diff.files
- **WHEN** the user runs `agentpack preview --json --diff`
- **THEN** stdout is valid JSON with `ok=true`
- **AND** `data.diff.summary` exists
- **AND** `data.diff.files` exists
