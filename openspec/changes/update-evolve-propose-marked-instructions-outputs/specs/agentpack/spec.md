## MODIFIED Requirements

### Requirement: Combined instructions outputs include per-module markers
When multiple `instructions` modules contribute to a combined deployed instructions output (for example Codex `AGENTS.md` or VS Code `.github/copilot-instructions.md`), the system SHALL include stable, per-module section markers so drift in the combined file can be mapped back to a specific module.

#### Scenario: Combined AGENTS.md contains markers
- **GIVEN** two `instructions` modules are enabled for the `codex` target
- **WHEN** the system renders desired state for Codex agent instructions
- **THEN** the combined `AGENTS.md` content contains a section marker per module id

#### Scenario: Combined VS Code copilot-instructions.md contains markers
- **GIVEN** two `instructions` modules are enabled for the `vscode` target
- **WHEN** the system renders desired state for VS Code Copilot instructions
- **THEN** the combined `.github/copilot-instructions.md` content contains a section marker per module id

### Requirement: evolve propose can map marked instructions drift back to a module
When a deployed combined instructions output contains valid per-module section markers, `agentpack evolve propose` SHALL treat drifted sections as proposeable and generate overlay updates for the corresponding `instructions` module(s).

#### Scenario: drifted marked section becomes a propose candidate (Codex AGENTS.md)
- **GIVEN** a deployed combined `AGENTS.md` containing section markers for `instructions:one` and `instructions:two`
- **AND** only the `instructions:one` section content is edited on disk
- **WHEN** the user runs `agentpack evolve propose --dry-run --json`
- **THEN** `data.candidates[]` contains an item with `module_id="instructions:one"`
- **AND** the drift is not reported as `multi_module_output` skipped for that output

#### Scenario: drifted marked section becomes a propose candidate (VS Code copilot-instructions.md)
- **GIVEN** a deployed combined `.github/copilot-instructions.md` containing section markers for `instructions:one` and `instructions:two`
- **AND** only the `instructions:one` section content is edited on disk
- **WHEN** the user runs `agentpack evolve propose --dry-run --json`
- **THEN** `data.candidates[]` contains an item with `module_id="instructions:one"`
- **AND** the drift is not reported as `multi_module_output` skipped for that output
