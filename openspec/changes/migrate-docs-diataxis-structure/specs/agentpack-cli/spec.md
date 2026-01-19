## ADDED Requirements

### Requirement: Docs follow a Diátaxis directory structure

The repository SHALL organize user-facing documentation under `docs/` using a Diátaxis-style directory structure:
- `docs/tutorials/`
- `docs/howto/`
- `docs/reference/`
- `docs/explanation/`

The canonical entrypoint (`docs/index.md`) SHALL link only to docs in these directories (or their `docs/zh-CN/` counterparts).

Previously published top-level doc paths that may be externally referenced (e.g., `docs/CLI.md`) SHALL remain as tombstone pages pointing to the new location.

#### Scenario: docs/index links only to Diátaxis paths
- **WHEN** a user follows links from `docs/index.md`
- **THEN** those links resolve to docs under `docs/tutorials/`, `docs/howto/`, `docs/reference/`, or `docs/explanation/`

#### Scenario: legacy doc paths redirect to new locations
- **GIVEN** a user opens a legacy doc path (e.g., `docs/CLI.md`)
- **WHEN** they read the page
- **THEN** it points them to the new canonical location (e.g., `docs/reference/cli.md`)
