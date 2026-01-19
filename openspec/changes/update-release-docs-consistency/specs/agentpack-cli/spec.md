## ADDED Requirements

### Requirement: Release installation docs stay consistent with crate version

The repository SHALL keep installation snippets in `README.md` and `docs/QUICKSTART.md` consistent with the crate version (`CARGO_PKG_VERSION`) to prevent drift to nonexistent or stale tags.

#### Scenario: CI catches stale install tags
- **GIVEN** a change that updates `CARGO_PKG_VERSION`
- **WHEN** `README.md` or `docs/QUICKSTART.md` still references a different `--tag v<version>`
- **THEN** CI fails with an actionable message describing the mismatch
