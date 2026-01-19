# agentpack-cli Delta

## ADDED Requirements

### Requirement: Journey J4 is covered by an integration test
The project SHALL include a deterministic, offline integration test for Journey J4 (overlay sparse/materialize/rebase/deploy) that validates rebase behavior and deploy output.

#### Scenario: sparse overlay can materialize, rebase, and deploy
- **GIVEN** a directory overlay created with `--sparse`
- **WHEN** the user materializes upstream files into the overlay
- **THEN** upstream files become editable under the overlay directory
- **WHEN** upstream changes conflict with overlay edits and the user runs `agentpack overlay rebase --json --yes`
- **THEN** the command fails with `errors[0].code = E_OVERLAY_REBASE_CONFLICT` and conflict-marked artifacts exist
- **WHEN** the conflicts are resolved
- **THEN** `agentpack deploy --apply --json --yes` deploys the overlay-composed content
