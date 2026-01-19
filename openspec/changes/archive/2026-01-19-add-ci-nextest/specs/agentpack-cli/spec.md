## ADDED Requirements

### Requirement: CI provides an optional nextest path

The repository SHALL provide a CI path that runs tests via `cargo nextest run` (optional / gradual adoption) to improve test feedback time and stability.

#### Scenario: CI can run nextest without changing runtime behavior
- **WHEN** a PR changes Rust code or tests
- **THEN** CI runs a `cargo nextest run` job (in addition to existing checks)
- **AND** the job does not change the compiled artifact behavior
