# agentpack (delta)

## ADDED Requirements

### Requirement: Cursor target writes project rules
The system SHALL support a built-in `cursor` target (files mode) that renders `instructions` modules into Cursor project rule files under `.cursor/rules`.

#### Scenario: deploy writes cursor rule files and a manifest
- **GIVEN** an enabled `instructions` module targeting `cursor`
- **WHEN** the user runs `agentpack --target cursor deploy --apply`
- **THEN** at least one `.mdc` rule file exists under `<project_root>/.cursor/rules`
- **AND** `<project_root>/.cursor/rules/.agentpack.manifest.json` exists

#### Scenario: rule filenames are stable and unique
- **GIVEN** two different enabled `instructions` modules targeting `cursor`
- **WHEN** the user runs `agentpack --target cursor deploy --apply`
- **THEN** the generated filenames are distinct and derived from each module’s `module_fs_key`

## MODIFIED Requirements

### Requirement: Target rendering is routed via a TargetAdapter registry
The system SHALL centralize target-specific rendering and validation behind a `TargetAdapter` abstraction, so adding a new target does not require scattering conditional logic across the engine and CLI.

#### Scenario: Known targets are resolved via registry
- **GIVEN** the system supports the `codex`, `claude_code`, and `cursor` targets
- **WHEN** the engine renders desired state for a selected target
- **THEN** the corresponding target adapter is used to compute target roots and desired output paths

### Requirement: Target conformance tests cover critical safety semantics
The repository SHALL include conformance tests that validate critical cross-target safety semantics, including:
- delete protection (only manifest-managed paths can be deleted)
- per-root manifest write/read
- drift classification (missing/modified/extra)
- rollback restoring previous outputs

#### Scenario: conformance tests exist for built-in targets
- **GIVEN** built-in targets `codex`, `claude_code`, and `cursor`
- **WHEN** the test suite is run
- **THEN** conformance tests execute these semantics for all built-in targets
