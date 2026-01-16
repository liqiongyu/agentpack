# agentpack (delta)

## ADDED Requirements

### Requirement: evolve propose skipped items include structured reason fields (additive)

When invoked as `agentpack evolve propose --dry-run --json`, each `data.skipped[]` item MUST include:
- `reason_code` (stable, enum-like string)
- `reason_message` (human-readable explanation)
- `next_actions[]` (suggested follow-up commands; may be empty)

This change MUST be additive for `schema_version=1` (existing fields like `reason` remain).

#### Scenario: missing drift includes reason_code and next_actions
- **GIVEN** an expected managed output is missing on disk
- **WHEN** the user runs `agentpack evolve propose --dry-run --json`
- **THEN** `data.skipped[]` contains an item with `reason=missing`
- **AND** that item contains `reason_code=missing`
- **AND** that item contains a non-empty `reason_message`
- **AND** `next_actions[]` includes at least one safe follow-up command

## MODIFIED Requirements

### Requirement: evolve propose reports skipped drift

When drift exists but cannot be safely mapped back to a single module (e.g., multi-module aggregated outputs) or when the deployed file is missing, `agentpack evolve propose` MUST report the drift as skipped instead of claiming there is no drift.

#### Scenario: missing drift is reported as skipped
- **GIVEN** an expected managed output is missing on disk
- **WHEN** the user runs `agentpack evolve propose --dry-run --json`
- **THEN** stdout is valid JSON with `ok=true`
- **AND** `data.reason` equals `no_proposeable_drift`
- **AND** `data.skipped[]` contains an item with `reason=missing`
- **AND** that item contains `reason_code=missing`
