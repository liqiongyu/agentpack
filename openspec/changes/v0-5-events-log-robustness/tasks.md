## 1. Implementation
- [x] Make reading `events.jsonl` tolerant of malformed/partial lines.
- [x] Add optional top-level `module_id` and `success` fields while keeping schema_version=1.
- [x] Surface parse warnings from `agentpack score` (human + `--json` warnings).

## 2. Tests
- [x] Add a CLI test asserting `score --json` succeeds with malformed lines and emits warnings.

## 3. Validation
- [x] Run `cargo fmt`, `cargo clippy`, `cargo test`.
- [x] Run `openspec validate v0-5-events-log-robustness --strict --no-interactive`.
