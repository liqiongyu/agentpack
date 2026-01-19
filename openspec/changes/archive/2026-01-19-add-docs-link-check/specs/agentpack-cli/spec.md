## ADDED Requirements

### Requirement: Docs internal links are validated in CI

The repository SHALL include an automated check that detects broken internal links in documentation under `docs/` (no network required).

#### Scenario: CI fails on broken doc links
- **WHEN** a documentation page links to a non-existent file path
- **THEN** CI fails with an actionable message identifying the source file and broken link target
