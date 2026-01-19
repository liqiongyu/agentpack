## ADDED Requirements

### Requirement: Single active Codex execution guide

The repository SHALL maintain `docs/dev/codex.md` as the single active execution guide for AI coding agents contributing to Agentpack.

Legacy Codex planning/workplan docs SHALL be archived under `docs/archive/plans/` and SHALL include YAML frontmatter marking them as `status: superseded` with `superseded_by: docs/dev/codex.md`.

#### Scenario: Contributors have one canonical execution guide
- **WHEN** a contributor looks for Codex execution guidance
- **THEN** `docs/dev/codex.md` exists and is the only active Codex execution guide

#### Scenario: Legacy Codex docs are archived with a pointer
- **GIVEN** a legacy Codex doc exists in `docs/archive/plans/`
- **WHEN** a contributor opens it
- **THEN** it is marked `status: superseded`
- **AND** it points to `docs/dev/codex.md`
