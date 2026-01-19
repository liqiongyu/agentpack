## ADDED Requirements

### Requirement: Single active roadmap doc

The repository SHALL maintain `docs/dev/roadmap.md` as the single active roadmap for maintainers.

Legacy roadmap/spec snapshot documents SHALL be moved under `docs/archive/roadmap/` and SHALL include YAML frontmatter marking them as archived, with `superseded_by: docs/dev/roadmap.md`.

#### Scenario: Maintainers have one roadmap to update
- **WHEN** a maintainer looks for the current roadmap
- **THEN** `docs/dev/roadmap.md` exists and is the only active roadmap doc

#### Scenario: Legacy roadmap docs are archived with a pointer
- **GIVEN** a legacy roadmap doc exists under `docs/archive/roadmap/`
- **WHEN** it is opened
- **THEN** it is marked `status: archived`
- **AND** it points to `docs/dev/roadmap.md`
