# Change: events.jsonl robustness + optional richer fields (v0.5)

## Why
`agentpack score` should not fail entirely if `events.jsonl` contains malformed/partial lines (e.g., truncated writes, manual edits). For automation and long-running usage, best-effort parsing with warnings is safer.

## What Changes
- Parse `events.jsonl` line-by-line; skip malformed/unsupported lines with warnings.
- Keep backwards compatibility: schema_version remains `1`; only add optional top-level fields.
- Add optional derived fields to `RecordedEvent`:
  - `module_id` (if derivable)
  - `success` (if derivable)
- Prefer these optional fields in scoring, falling back to extracting from `event`.

## Impact
- Affected specs: `agentpack`
- Affected code: `src/events.rs`, `agentpack score`, `agentpack record`
- Affected tests: add coverage for robustness and warnings
