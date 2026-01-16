## ADDED Requirements

### Requirement: init --guided creates a minimal working manifest

The system SHALL provide `agentpack init --guided` as an interactive wizard to generate a minimal `repo/agentpack.yaml` that can be used immediately by common workflows.

The wizard SHOULD ask, at minimum:
- which targets to configure (`codex`, `claude_code`, `cursor`, `vscode`)
- the target scope (`project` or `both`)
- whether to bootstrap operator assets after init

#### Scenario: guided init writes a manifest in a TTY
- **GIVEN** a clean `$AGENTPACK_HOME`
- **AND** stdin and stdout are terminals
- **WHEN** the user runs `agentpack init --guided`
- **THEN** the command exits zero
- **AND** `repo/agentpack.yaml` exists and is parseable

### Requirement: guided init refuses to run without a TTY

When `agentpack init --guided` is invoked without a TTY (stdin or stdout is not a terminal), the system MUST fail early and MUST NOT write any files.

In `--json` mode, the system MUST return a stable error code so automation can branch safely.

#### Scenario: guided init fails in non-TTY JSON mode
- **GIVEN** stdin or stdout is not a terminal
- **WHEN** the user runs `agentpack init --guided --json`
- **THEN** the command exits non-zero
- **AND** stdout is valid JSON with `ok=false`
- **AND** `errors[0].code` equals `E_TTY_REQUIRED`
