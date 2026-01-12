# agentpack (delta)

## ADDED Requirements

### Requirement: evolve propose reports skipped drift
When drift exists but cannot be safely mapped back to a single module (e.g., multi-module aggregated outputs) or when the deployed file is missing, `agentpack evolve propose` MUST report the drift as skipped instead of claiming there is no drift.

#### Scenario: missing drift is reported as skipped
- **GIVEN** an expected managed output is missing on disk
- **WHEN** the user runs `agentpack evolve propose --dry-run --json`
- **THEN** stdout is valid JSON with `ok=true`
- **AND** `data.reason` equals `no_proposeable_drift`
- **AND** `data.skipped[]` contains an item with `reason=missing`
