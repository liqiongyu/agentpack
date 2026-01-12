# Change: Update events.jsonl as an evolvable audit log

## Why
`state/logs/events.jsonl` is the foundation for “AI-first observability” (`record` → `score` → `explain` → `evolve`). It should remain usable and diagnosable even when logs contain partial corruption or mixed schema versions, and it should support additive metadata for richer reporting.

## What Changes
- Define and document `events.jsonl` schema_version compatibility rules (forward/backwards behavior).
- Add optional top-level metadata fields to recorded events (additive, backwards-compatible).
- Enhance `agentpack score` to report counts of skipped lines and skip reasons (in both human output and `--json` data).

## Impact
- Affected specs: `openspec/specs/agentpack/spec.md`
- Affected docs: `docs/SPEC.md`
- Affected code: `src/events.rs`, `src/cli.rs`
- Affected tests: CLI integration tests for `score`
