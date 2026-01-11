# agentpack (delta)

## MODIFIED Requirements

### Requirement: Reproducible materialization from lockfile
When a module is resolved from `agentpack.lock.json` and its git checkout directory is missing locally, the system MUST automatically populate the missing checkout (safe network fetch) or fail with an actionable error instructing the user to run `agentpack fetch/update`.

#### Scenario: Missing checkout is auto-fetched
- **GIVEN** `agentpack.lock.json` pins a git module to commit `<commit>`
- **AND** the local checkout directory for `<moduleId, commit>` does not exist
- **WHEN** the system materializes that module as part of `plan/diff/deploy`
- **THEN** the missing checkout is populated automatically
