# agentpack (delta)

## MODIFIED Requirements

### Requirement: Doctor Self-Check
Agentpack MUST provide a `doctor` command that outputs a deterministic `machineId` and validates target paths for existence and writability with actionable guidance. Additionally, when a target root is inside a git repository, `doctor` MUST warn if `.agentpack.manifest.json` is not ignored, and `doctor --fix` MUST be able to idempotently add it to `.gitignore`.

#### Scenario: Doctor warns about committing target manifests
- **GIVEN** a target root inside a git repository
- **AND** `.agentpack.manifest.json` is not ignored
- **WHEN** the user runs `agentpack doctor`
- **THEN** the output includes a warning recommending adding it to `.gitignore`
