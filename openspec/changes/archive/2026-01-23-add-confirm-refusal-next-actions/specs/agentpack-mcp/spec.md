## ADDED Requirements

### Requirement: Confirm-token refusals are machine-actionable
When `deploy_apply` is refused due to confirmation token errors (`E_CONFIRM_TOKEN_REQUIRED`, `E_CONFIRM_TOKEN_EXPIRED`, `E_CONFIRM_TOKEN_MISMATCH`), the system SHALL include additive, machine-actionable fields under `errors[0].details`:
- `reason_code: string` (stable, enum-like)
- `next_actions: string[]` (stable, enum-like action identifiers)

#### Scenario: deploy_apply with mismatched token includes refusal details
- **WHEN** a client calls tool `deploy_apply` with `yes=true` and a token that does not match the current deploy plan
- **THEN** the tool result has `isError=true`
- **AND** the Agentpack envelope includes `errors[0].code = E_CONFIRM_TOKEN_MISMATCH`
- **AND** `errors[0].details.reason_code` is present
- **AND** `errors[0].details.next_actions` is present
