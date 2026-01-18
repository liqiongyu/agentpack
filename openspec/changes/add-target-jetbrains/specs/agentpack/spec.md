# agentpack (delta)

## ADDED Requirements

### Requirement: JetBrains target writes Junie guidelines file
The system SHALL support a `jetbrains` target (files mode, project scope) that renders `instructions` modules into:
`<project_root>/.junie/guidelines.md`.

If multiple `instructions` modules target `jetbrains`, the output SHOULD preserve attribution via per-module section markers.

#### Scenario: deploy writes jetbrains guidelines and manifests
- **GIVEN** at least one enabled `instructions` module targeting `jetbrains`
- **WHEN** the user runs `agentpack --target jetbrains deploy --apply`
- **THEN** `<project_root>/.junie/guidelines.md` exists
- **AND** `<project_root>/.junie/.agentpack.manifest.json` exists

## MODIFIED Requirements

### Requirement: Target rendering is routed via a TargetAdapter registry
The system SHALL centralize target-specific rendering and validation behind a `TargetAdapter` abstraction, so adding a new target does not require scattering conditional logic across the engine and CLI.

#### Scenario: Known targets are resolved via registry
- **GIVEN** the system supports the `codex`, `claude_code`, `cursor`, `vscode`, and `jetbrains` targets
- **WHEN** the engine renders desired state for a selected target
- **THEN** the corresponding target adapter is used to compute target roots and desired output paths

### Requirement: Target conformance tests cover critical safety semantics
The repository SHALL include conformance tests that validate critical cross-target safety semantics, including:
- delete protection (only manifest-managed paths can be deleted)
- per-root manifest write/read
- drift classification (missing/modified/extra)
- rollback restoring previous outputs

#### Scenario: conformance tests exist for built-in targets
- **GIVEN** built-in targets `codex`, `claude_code`, `cursor`, `vscode`, and `jetbrains`
- **WHEN** the test suite is run
- **THEN** conformance tests execute these semantics for all built-in targets
