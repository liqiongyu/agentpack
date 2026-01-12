# agentpack (delta)

## ADDED Requirements

### Requirement: DesiredState path conflicts are refused
If multiple modules attempt to produce different content for the same `(target, path)`, the system MUST fail fast instead of silently overwriting based on insertion order.

If the bytes are identical, the system SHOULD merge module provenance so the output can still be attributed to all contributing modules.

In `--json` mode, the system MUST return a stable error code `E_DESIRED_STATE_CONFLICT`.

#### Scenario: conflict is detected before apply
- **GIVEN** two modules render different bytes to the same output path
- **WHEN** the user runs `agentpack plan --json` (or `preview/deploy`)
- **THEN** the command exits non-zero
- **AND** stdout is valid JSON with `ok=false`
- **AND** `errors[0].code` equals `E_DESIRED_STATE_CONFLICT`
