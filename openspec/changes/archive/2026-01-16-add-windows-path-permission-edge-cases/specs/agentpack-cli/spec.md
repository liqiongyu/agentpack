## ADDED Requirements

### Requirement: JSON-mode filesystem write failures use stable error codes

When invoked with `--json`, write-capable commands (e.g., `deploy`, `rollback`, `bootstrap`, `evolve restore`) SHALL classify common filesystem write failures into stable error codes (documented in `docs/ERROR_CODES.md`) instead of falling back to `E_UNEXPECTED`.

At minimum, the system MUST provide stable codes for:
- permission denied / read-only write failures
- invalid path / invalid characters
- path too long

#### Scenario: deploy returns stable error code on permission denied
- **GIVEN** the destination path is not writable
- **WHEN** the user runs `agentpack deploy --apply --json`
- **THEN** the command exits non-zero
- **AND** `errors[0].code` is a stable documented code for permission denied

#### Scenario: deploy returns stable error code on invalid path
- **GIVEN** the destination path is invalid for the platform
- **WHEN** the user runs `agentpack deploy --apply --json`
- **THEN** the command exits non-zero
- **AND** `errors[0].code` is a stable documented code for invalid path

#### Scenario: deploy returns stable error code on path too long
- **GIVEN** the destination path exceeds platform limits
- **WHEN** the user runs `agentpack deploy --apply --json`
- **THEN** the command exits non-zero
- **AND** `errors[0].code` is a stable documented code for path too long
