## ADDED Requirements

### Requirement: Overlay baseline errors are machine-actionable
When a `--json` invocation fails due to missing/unsupported overlays or overlay baseline metadata, the system SHALL include additive, machine-actionable fields under `errors[0].details`:
- `reason_code: string` (stable, enum-like)
- `next_actions: string[]` (stable, enum-like action identifiers)

This requirement applies to these stable error codes:
- `E_OVERLAY_NOT_FOUND`
- `E_OVERLAY_BASELINE_MISSING`
- `E_OVERLAY_BASELINE_UNSUPPORTED`

#### Scenario: overlay not found includes guidance fields
- **GIVEN** a module exists but its overlay directory does not exist
- **WHEN** the user runs `agentpack overlay rebase <module_id> --scope global --json --yes`
- **THEN** stdout is valid JSON with `ok=false`
- **AND** `errors[0].code` is `E_OVERLAY_NOT_FOUND`
- **AND** `errors[0].details.reason_code` is present
- **AND** `errors[0].details.next_actions` is present

#### Scenario: overlay baseline missing includes guidance fields
- **GIVEN** an overlay directory exists but its baseline metadata is missing
- **WHEN** the user runs `agentpack overlay rebase <module_id> --scope global --json --yes`
- **THEN** stdout is valid JSON with `ok=false`
- **AND** `errors[0].code` is `E_OVERLAY_BASELINE_MISSING`
- **AND** `errors[0].details.reason_code` is present
- **AND** `errors[0].details.next_actions` is present

#### Scenario: overlay baseline unsupported includes guidance fields
- **GIVEN** an overlay directory exists but its baseline cannot locate a merge base
- **WHEN** the user runs `agentpack overlay rebase <module_id> --scope global --json --yes`
- **THEN** stdout is valid JSON with `ok=false`
- **AND** `errors[0].code` is `E_OVERLAY_BASELINE_UNSUPPORTED`
- **AND** `errors[0].details.reason_code` is present
- **AND** `errors[0].details.next_actions` is present
