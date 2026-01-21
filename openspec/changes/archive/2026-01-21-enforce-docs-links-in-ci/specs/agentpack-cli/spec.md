## ADDED Requirements

### Requirement: CI fails on broken markdown links

The repository SHALL validate internal markdown links in `docs/` and top-level README files in CI, and SHALL fail CI when a link target resolves to a missing file.

#### Scenario: Docs-only PR with a broken link is blocked
- **GIVEN** a PR changes markdown content under `docs/` or `README*.md`
- **WHEN** it introduces a link to a missing file
- **THEN** CI fails due to the markdown link check
