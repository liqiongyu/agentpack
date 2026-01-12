## 1. Spec & Docs

- [x] Update OpenSpec deltas for events/score behavior
- [x] Update `docs/SPEC.md` for events.jsonl compatibility + optional fields

## 2. Implementation

- [x] Extend `RecordedEvent` with optional metadata fields
- [x] Track structured read/skip stats when reading events.jsonl
- [x] Surface skip stats in `agentpack score` (human + `--json`)

## 3. Tests

- [x] Add/extend tests for score skip stats and compatibility behavior

## 4. Validation

- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy --all-targets --all-features -- -D warnings`
- [x] `cargo test --all --locked`
- [x] `openspec validate update-events-audit-log --strict --no-interactive`
