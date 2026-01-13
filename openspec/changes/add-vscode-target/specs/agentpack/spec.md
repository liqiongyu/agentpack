# agentpack (delta)

## ADDED Requirements

### Requirement: VS Code target writes Copilot instruction and prompt files
The system SHALL support a built-in `vscode` target (files mode) that renders:
- `instructions` modules into `.github/copilot-instructions.md`, and
- `prompt` modules into `.github/prompts/*.prompt.md`.

#### Scenario: deploy writes vscode files and manifests
- **GIVEN** at least one enabled module targeting `vscode`
- **WHEN** the user runs `agentpack --target vscode deploy --apply`
- **THEN** `<project_root>/.github/copilot-instructions.md` exists when `instructions` modules are present
- **AND** `<project_root>/.github/prompts/` contains `.prompt.md` files when `prompt` modules are present
- **AND** per-root `.agentpack.manifest.json` files exist under `.github/` and `.github/prompts/`

#### Scenario: prompt filenames end with .prompt.md
- **GIVEN** a `prompt` module whose source file is named `hello.md`
- **WHEN** the user runs `agentpack --target vscode deploy --apply`
- **THEN** the deployed file name ends with `.prompt.md`

## MODIFIED Requirements

### Requirement: Target rendering is routed via a TargetAdapter registry
The system SHALL centralize target-specific rendering and validation behind a `TargetAdapter` abstraction, so adding a new target does not require scattering conditional logic across the engine and CLI.

#### Scenario: Known targets are resolved via registry
- **GIVEN** the system supports the `codex`, `claude_code`, `cursor`, and `vscode` targets
- **WHEN** the engine renders desired state for a selected target
- **THEN** the corresponding target adapter is used to compute target roots and desired output paths

### Requirement: Target conformance tests cover critical safety semantics
The repository SHALL include conformance tests that validate critical cross-target safety semantics, including:
- delete protection (only manifest-managed paths can be deleted)
- per-root manifest write/read
- drift classification (missing/modified/extra)
- rollback restoring previous outputs

#### Scenario: conformance tests exist for built-in targets
- **GIVEN** built-in targets `codex`, `claude_code`, `cursor`, and `vscode`
- **WHEN** the test suite is run
- **THEN** conformance tests execute these semantics for all built-in targets
