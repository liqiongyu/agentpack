## ADDED Requirements

### Requirement: Adopt-confirm refusals are machine-actionable
When `deploy_apply` is refused due to missing explicit adopt confirmation (`E_ADOPT_CONFIRM_REQUIRED`), the system SHALL include additive, machine-actionable fields under `errors[0].details`:
- `reason_code: string` (stable, enum-like)
- `next_actions: string[]` (stable, enum-like action identifiers)

#### Scenario: deploy_apply without adopt includes refusal details
- **GIVEN** the deploy plan contains at least one `adopt_update`
- **WHEN** a client calls tool `deploy_apply` with `yes=true`, a valid `confirm_token`, and `adopt=false`
- **THEN** the tool result has `isError=true`
- **AND** the Agentpack envelope includes `errors[0].code = E_ADOPT_CONFIRM_REQUIRED`
- **AND** `errors[0].details.reason_code` is present
- **AND** `errors[0].details.next_actions` is present
