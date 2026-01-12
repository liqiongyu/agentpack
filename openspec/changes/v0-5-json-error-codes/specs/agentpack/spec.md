# agentpack (delta)

## ADDED Requirements

### Requirement: JSON mode uses stable error codes for common failures
In `--json` mode, user-facing failures that are common and actionable MUST return stable error codes (not just `E_UNEXPECTED`) so automation can branch reliably.

At minimum, the following scenarios MUST return stable codes:
- Missing config manifest: `E_CONFIG_MISSING`
- Invalid config manifest: `E_CONFIG_INVALID`
- Unsupported config version: `E_CONFIG_UNSUPPORTED_VERSION`
- Missing lockfile when required: `E_LOCKFILE_MISSING`
- Invalid lockfile JSON: `E_LOCKFILE_INVALID`
- Unsupported `--target`: `E_TARGET_UNSUPPORTED`

#### Scenario: missing config yields stable error code
- **GIVEN** `agentpack.yaml` is missing
- **WHEN** the user runs `agentpack plan --json`
- **THEN** `errors[0].code` equals `E_CONFIG_MISSING`
