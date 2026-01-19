## ADDED Requirements

### Requirement: Single user entrypoint in docs

The repository SHALL provide `docs/index.md` as the canonical user entrypoint, linking to tutorials/how-to guides/reference docs such that a new user can reliably find the next step for common workflows (from-scratch setup, import, daily loop, automation).

#### Scenario: Entry point enables first-day success
- **WHEN** a new user opens `docs/index.md`
- **THEN** they can choose a path (from-scratch vs import) and reach a next action in ≤ 3 clicks
