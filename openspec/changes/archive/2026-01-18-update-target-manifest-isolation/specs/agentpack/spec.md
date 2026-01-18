# agentpack (delta)

## MODIFIED Requirements

### Requirement: Target Manifest
`agentpack deploy --apply` MUST write a target manifest file into each managed target root directory that records the managed files and their hashes.

To avoid collisions when multiple targets share the same root, the manifest filename MUST be target-specific:
- `<root>/.agentpack.manifest.<target>.json`

For backwards compatibility, the implementation MAY also read the legacy manifest filename:
- `<root>/.agentpack.manifest.json`

The legacy manifest MUST be treated as belonging to the expected target only when the manifest `tool` field matches the selected target id. Otherwise, it MUST be ignored (treated as missing) to avoid cross-target deletion risks.

#### Scenario: per-target manifest is written and used
- **GIVEN** a deployment writes a manifest for target `codex`
- **WHEN** the user runs `agentpack deploy --apply`
- **THEN** the managed root contains `.agentpack.manifest.codex.json`

#### Scenario: legacy manifest is ignored when tool does not match target
- **GIVEN** a target root contains a legacy `.agentpack.manifest.json` whose `tool="codex"`
- **WHEN** the user runs `agentpack --target cursor status --json`
- **THEN** that manifest is treated as missing for `cursor`

### Requirement: Doctor Self-Check
Agentpack MUST provide a `doctor` command that outputs a deterministic `machine_id` and validates target paths for existence and writability with actionable guidance. Additionally, when a target root is inside a git repository, `doctor` MUST warn if `.agentpack.manifest*.json` is not ignored, and `doctor --fix` MUST be able to idempotently add it to `.gitignore`.

#### Scenario: Doctor warns about committing target manifests
- **GIVEN** a target root inside a git repository
- **AND** `.agentpack.manifest*.json` is not ignored
- **WHEN** the user runs `agentpack doctor`
- **THEN** the output includes a warning recommending adding it to `.gitignore`
