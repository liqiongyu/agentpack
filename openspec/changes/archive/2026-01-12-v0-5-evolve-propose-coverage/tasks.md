## 1. Implementation
- [x] Report skipped drift reasons for missing and multi-module outputs.
- [x] Update operator template to recommend `--dry-run` first.

## 2. Tests
- [x] Add a CLI test that asserts `--dry-run --json` reports skipped drift instead of `no_drift`.

## 3. Docs
- [x] Update `docs/SPEC.md` to document conservative behavior and next steps.

## 4. Validation
- [x] Run `cargo fmt`, `cargo clippy`, `cargo test`.
- [x] Run `openspec validate v0-5-evolve-propose-coverage --strict --no-interactive`.
