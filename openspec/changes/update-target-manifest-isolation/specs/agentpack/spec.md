# agentpack (delta)

## MODIFIED Requirements

### Requirement: target manifests define the managed file boundary
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
- **WHEN** the user runs `agentpack --target zed status --json`
- **THEN** that manifest is treated as missing for `zed`

### Requirement: doctor enforces gitignore safety for target manifests
When a target root is inside a git repository, `doctor` MUST warn if target manifest files are not ignored, and `doctor --fix` MUST be able to idempotently add an ignore rule.

The ignore rule SHOULD cover both legacy and per-target manifest filenames:
- `.agentpack.manifest*.json`

#### Scenario: doctor --fix adds wildcard ignore rule
- **GIVEN** a target root directory is inside a git repo
- **AND** target manifest files are not ignored
- **WHEN** the user runs `agentpack doctor --fix --yes`
- **THEN** the repo’s `.gitignore` contains an ignore entry for `.agentpack.manifest*.json`
