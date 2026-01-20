## ADDED Requirements

### Requirement: policy lint/lock JSON output is protected by golden snapshots

The repository SHALL provide golden snapshot tests for the JSON `data` payloads of:
- `agentpack policy lock --json --yes`
- `agentpack policy lint --json`

The snapshots SHALL normalize temporary paths to ensure deterministic results across OS.

#### Scenario: policy lock/lint JSON data is stable for automation
- **WHEN** the test suite runs in CI
- **THEN** golden tests validate the normalized JSON `data` payloads for `policy lock` and `policy lint`
- **AND** the snapshots remain deterministic across OS (normalized temp paths, stable ordering)
