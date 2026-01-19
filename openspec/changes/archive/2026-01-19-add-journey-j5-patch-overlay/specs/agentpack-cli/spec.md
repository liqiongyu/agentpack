# agentpack-cli Delta

## ADDED Requirements

### Requirement: Journey J5 is covered by an integration test

The project SHALL include a deterministic, offline integration test for Journey J5 (patch overlay generate/rebase/apply) that validates patch application and rebase conflict behavior.

#### Scenario: patch overlay can apply and rebase with conflict artifacts
- **GIVEN** a patch overlay created via `agentpack overlay edit <module_id> --kind patch`
- **WHEN** the user adds a unified diff patch under `.agentpack/patches/`
- **THEN** `agentpack deploy --apply --json --yes` deploys patched content
- **WHEN** upstream changes conflict with the patch and the user runs `agentpack overlay rebase --json --yes`
- **THEN** the command fails with `errors[0].code = E_OVERLAY_REBASE_CONFLICT` and conflict artifacts exist under `.agentpack/conflicts/<relpath>`
