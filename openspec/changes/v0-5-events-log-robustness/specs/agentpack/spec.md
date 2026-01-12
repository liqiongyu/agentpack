# agentpack (delta)

## ADDED Requirements

### Requirement: score tolerates malformed events.jsonl lines
`agentpack score` MUST tolerate malformed/partial lines in `events.jsonl` by skipping bad lines and emitting warnings, instead of failing the whole command.

#### Scenario: malformed line is skipped
- **GIVEN** `events.jsonl` contains both valid and invalid JSON lines
- **WHEN** the user runs `agentpack score --json`
- **THEN** the command exits successfully with `ok=true`
- **AND** `warnings` includes an entry referencing the skipped line
