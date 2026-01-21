## ADDED Requirements

### Requirement: Docs include a rendered architecture diagram

The repository SHALL include a Mermaid architecture diagram (renderable by GitHub) that explains the core Agentpack pipeline and links it from a primary entry surface.

#### Scenario: New user can see the pipeline at a glance
- **GIVEN** a new user opens `README.md` or the docs entrypoint
- **WHEN** they scroll to the architecture section
- **THEN** they can see an inline Mermaid diagram describing manifest/lock/overlays → render → plan/diff → apply → snapshots/manifests/events
