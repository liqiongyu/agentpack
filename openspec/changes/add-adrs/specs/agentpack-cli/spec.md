## ADDED Requirements

### Requirement: Architecture decisions are recorded as ADRs

The repository SHALL record major architecture decisions as Architecture Decision Records (ADRs) under `docs/adr/`, using stable numbering and a consistent structure (status, context, decision, consequences).

The maintainer roadmap (`docs/dev/roadmap.md`) SHOULD link to ADRs for “why” instead of duplicating long-form rationale inline.

#### Scenario: ADRs exist for core contracts and workflows
- **WHEN** a contributor needs to understand the rationale for key product contracts and workflows
- **THEN** ADRs exist that cover:
  - JSON contract stability (`--json` schema_version=1, additive-only)
  - Patch overlays design
  - MCP `confirm_token` two-phase apply design

#### Scenario: Roadmap links to ADRs for rationale
- **WHEN** a maintainer reads the roadmap’s principles/spec sections
- **THEN** the roadmap includes links to relevant ADRs for deeper rationale
