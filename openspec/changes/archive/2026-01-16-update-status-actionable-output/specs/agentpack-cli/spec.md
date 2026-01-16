## ADDED Requirements

### Requirement: status emits grouped drift summary (additive)

When invoked as `agentpack status --json`, the system SHALL include an additive `data.summary_by_root` field to make drift counts easy to consume without re-grouping client-side.

`data.summary_by_root` SHALL be a list of items with:
- `target: string`
- `root: string`
- `root_posix: string`
- `summary: {modified, missing, extra}`

The list SHOULD be deterministic (stable ordering), ordered by `(target, root_posix)` ascending.

#### Scenario: status groups drift by root
- **GIVEN** drift exists under at least two different roots
- **WHEN** the user runs `agentpack status --json`
- **THEN** `data.summary_by_root` contains at least two entries
- **AND** each entry’s `summary` counts match the corresponding `data.drift[]` items

### Requirement: status emits structured next actions (additive)

When invoked as `agentpack status --json`, the system SHALL include an additive `data.next_actions_detailed` field that provides structured next actions for automation.

`data.next_actions_detailed` SHALL be a list of objects with:
- `action: string` (stable, enum-like)
- `command: string` (a safe follow-up command)

`data.next_actions_detailed[].command` values SHOULD correspond 1:1 to the commands in `data.next_actions[]` when both are present, and SHOULD have the same ordering.

Initial `action` codes emitted by `status` SHOULD include:
- `bootstrap`
- `preview_diff`
- `deploy_apply`
- `evolve_propose`

#### Scenario: status suggests structured bootstrap action
- **GIVEN** operator assets are missing or outdated
- **WHEN** the user runs `agentpack status --json`
- **THEN** `data.next_actions_detailed[]` contains an item with `action = "bootstrap"`
- **AND** that item’s `command` contains `agentpack bootstrap`

## MODIFIED Requirements

### Requirement: schema command documents JSON output contract

The `agentpack schema --json` payload SHALL document `status` additive fields including:
- `next_actions?`
- `next_actions_detailed?`
- `summary_by_root?`

#### Scenario: schema lists status additive status fields
- **WHEN** the user runs `agentpack schema --json`
- **THEN** the schema output documents `status` data fields including `next_actions_detailed?`
- **AND** the schema output documents `status` data fields including `summary_by_root?`
