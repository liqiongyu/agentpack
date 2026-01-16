## ADDED Requirements

### Requirement: evolve propose default branch names include scope and module attribution
When `agentpack evolve propose` creates a proposal branch without an explicit `--branch`, the system SHALL use a deterministic, informative branch name that includes:
- the proposal scope (`global|machine|project`), and
- module attribution (either a specific `module_id` when it can be determined, or `multi`), and
- a timestamp component for uniqueness.

The branch name MUST be git-safe (no spaces; no characters invalid for ref names).

#### Scenario: module-filtered proposal includes module_id and scope in branch name
- **GIVEN** drift exists for module `instructions:one`
- **WHEN** the user runs `agentpack evolve propose --module-id instructions:one --scope global`
- **THEN** the created branch name includes `global` and `instructions:one` (or a sanitized equivalent)
