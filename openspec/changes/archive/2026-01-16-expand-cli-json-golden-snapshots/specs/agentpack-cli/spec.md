## ADDED Requirements

### Requirement: Core CLI commands are covered by JSON golden snapshots

The repository SHALL include JSON golden snapshots that cover the success-path `data` payloads for the core CLI commands, so contract changes to `--json` output are detected by CI and reviewed explicitly.

At minimum, the snapshots MUST include coverage for:
- `init`, `update`
- `plan`, `diff`, `preview`
- `deploy`, `status`, `doctor`, `rollback`
- `overlay path`
- `evolve` (at least one representative scenario)

#### Scenario: CI fails when core command JSON changes
- **GIVEN** a change modifies a core command’s `--json` `data` payload
- **WHEN** CI runs the JSON golden snapshot tests
- **THEN** the tests fail with a diff that requires an explicit golden update
