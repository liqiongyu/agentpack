## MODIFIED Requirements

### Requirement: score tolerates malformed events.jsonl lines
`agentpack score` MUST tolerate malformed/partial lines in `events.jsonl` by skipping bad lines and emitting warnings, instead of failing the whole command.

Additionally, the system SHOULD provide a structured summary of how many lines were skipped and why (e.g. read errors, malformed JSON, unsupported schema_version) so operators and automation can diagnose log health.

#### Scenario: malformed line is skipped
- **GIVEN** `events.jsonl` contains both valid and invalid JSON lines
- **WHEN** the user runs `agentpack score --json`
- **THEN** the command exits successfully with `ok=true`
- **AND** `warnings` includes an entry referencing the skipped line

#### Scenario: score reports skipped counts
- **GIVEN** `events.jsonl` contains malformed JSON lines and unsupported schema versions
- **WHEN** the user runs `agentpack score --json`
- **THEN** stdout is valid JSON with `ok=true`
- **AND** `data.read_stats.skipped_total` is greater than 0
- **AND** `data.read_stats.skipped_malformed_json` is greater than 0

## ADDED Requirements

### Requirement: events.jsonl is forward-compatible
`events.jsonl` MUST be treated as an evolvable audit log:
- Readers MUST ignore unknown top-level fields (additive changes).
- Readers MUST skip unsupported `schema_version` entries and emit warnings (do not fail the whole command).

#### Scenario: unknown fields are ignored
- **GIVEN** an `events.jsonl` entry includes additional top-level fields that are not recognized by this version
- **WHEN** the user runs `agentpack score --json`
- **THEN** the entry is parsed successfully and does not cause a failure

#### Scenario: unsupported schema_version is skipped
- **GIVEN** an `events.jsonl` entry with `schema_version` not supported by this version
- **WHEN** the user runs `agentpack score --json`
- **THEN** the entry is skipped
- **AND** a warning is emitted
