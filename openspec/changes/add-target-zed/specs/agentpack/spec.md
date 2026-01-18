# agentpack (delta)

## ADDED Requirements

### Requirement: Zed target writes AI rules file
The system SHALL support a built-in `zed` target (files mode) that renders `instructions` modules into a repo-root `.rules` file for Zed AI Rules consumption.

#### Scenario: deploy writes zed rules and a manifest
- **GIVEN** at least one enabled `instructions` module targeting `zed`
- **WHEN** the user runs `agentpack --target zed deploy --apply`
- **THEN** `<project_root>/.rules` exists
- **AND** `<project_root>/.agentpack.manifest.zed.json` exists

## MODIFIED Requirements

### Requirement: Target rendering is routed via a TargetAdapter registry
The system SHALL centralize target-specific rendering and validation behind a `TargetAdapter` abstraction, so adding a new target does not require scattering conditional logic across the engine and CLI.

#### Scenario: Known targets are resolved via registry
- **GIVEN** the system supports the `codex`, `claude_code`, `cursor`, `vscode`, `jetbrains`, and `zed` targets
- **WHEN** the engine renders desired state for a selected target
- **THEN** the corresponding target adapter is used to compute target roots and desired output paths

### Requirement: Target conformance tests cover critical safety semantics
The repository SHALL include conformance tests that validate critical cross-target safety semantics, including:
- delete protection (only manifest-managed paths can be deleted)
- per-root manifest write/read
- drift classification (missing/modified/extra)
- rollback restoring previous outputs

#### Scenario: conformance tests exist for built-in targets
- **GIVEN** built-in targets `codex`, `claude_code`, `cursor`, `vscode`, `jetbrains`, and `zed`
- **WHEN** the test suite is run
- **THEN** conformance tests execute these semantics for all built-in targets
