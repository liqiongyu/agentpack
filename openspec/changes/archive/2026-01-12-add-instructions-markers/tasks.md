## 1. Implementation
- [x] Add stable section marker format for aggregated instructions outputs.
- [x] Extend `evolve propose` to parse marker sections for `AGENTS.md` and propose per-module overlay updates.
- [x] Keep conservative fallback: if markers are missing/unparseable, keep reporting `multi_module_output` as skipped.

## 2. Tests
- [x] Add a CLI test that:
  - deploys combined `AGENTS.md` with markers
  - edits only one marked module section
  - asserts `evolve propose --dry-run --json` returns a candidate for that module (not `multi_module_output`)

## 3. Docs
- [x] Update `docs/SPEC.md` to document marker format and the evolve workflow for aggregated instructions.

## 4. Validation
- [x] Run `cargo fmt --all -- --check`.
- [x] Run `cargo clippy --all-targets --all-features -- -D warnings`.
- [x] Run `cargo test --all --locked`.
- [x] Run `openspec validate add-instructions-markers --strict --no-interactive`.
