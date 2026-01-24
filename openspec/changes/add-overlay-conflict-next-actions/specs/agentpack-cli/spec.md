## ADDED Requirements

### Requirement: Overlay conflict errors are machine-actionable
When a `--json` invocation fails due to an overlay conflict, the system SHALL include additive, machine-actionable fields under `errors[0].details`:
- `reason_code: string` (stable, enum-like)
- `next_actions: string[]` (stable, enum-like action identifiers)

This requirement applies to these stable error codes:
- `E_OVERLAY_REBASE_CONFLICT`
- `E_OVERLAY_PATCH_APPLY_FAILED`

#### Scenario: overlay conflict errors include guidance fields
- **GIVEN** an overlay operation fails due to conflicts or patch apply failures
- **WHEN** the user runs a command that produces `E_OVERLAY_REBASE_CONFLICT` or `E_OVERLAY_PATCH_APPLY_FAILED` in `--json` mode
- **THEN** stdout is valid JSON with `ok=false`
- **AND** `errors[0].details.reason_code` is present
- **AND** `errors[0].details.next_actions` is present
