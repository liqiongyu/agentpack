# agentpack-cli (delta)

## MODIFIED Requirements

### Requirement: Target adapters can be feature-gated at build time
The system SHALL support building agentpack with a subset of target adapters enabled via Cargo features:
- `target-codex`
- `target-claude-code`
- `target-cursor`
- `target-vscode`
- `target-jetbrains`
- `target-zed`

The default feature set SHOULD include all built-in targets (to preserve the default user experience).

If a user selects a target that is not compiled into the running binary, the CLI MUST treat it as unsupported.

#### Scenario: selecting a non-compiled target fails with E_TARGET_UNSUPPORTED
- **GIVEN** an agentpack binary built without `target-cursor`
- **WHEN** the user runs `agentpack plan --target cursor --json`
- **THEN** stdout is valid JSON with `ok=false`
- **AND** `errors[0].code` equals `E_TARGET_UNSUPPORTED`
