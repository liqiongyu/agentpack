## ADDED Requirements

### Requirement: Error code registry documents confirm guidance fields
The stable error code registry (`docs/reference/error-codes.md`) SHALL document that confirm-related stable errors include additive guidance fields under `errors[0].details`:
- `reason_code: string` (stable, enum-like)
- `next_actions: string[]` (stable, enum-like action identifiers)

This requirement applies to these stable error codes:
- `E_CONFIRM_REQUIRED`
- `E_CONFIRM_TOKEN_REQUIRED`
- `E_CONFIRM_TOKEN_EXPIRED`
- `E_CONFIRM_TOKEN_MISMATCH`

#### Scenario: confirm errors are documented with guidance fields
- **WHEN** a maintainer updates `docs/reference/error-codes.md`
- **THEN** the sections for the confirm-related error codes above mention `reason_code` and `next_actions` as additive guidance fields
