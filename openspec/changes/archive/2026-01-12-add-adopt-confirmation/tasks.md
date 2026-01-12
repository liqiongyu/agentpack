## 1. Spec + Docs
- [x] Add OpenSpec delta requirement for adopt confirmation
- [x] Update `docs/SPEC.md` to document adopt behavior and error code

## 2. Implementation
- [x] Extend plan output to mark adopt updates (machine-readable)
- [x] Add deploy flag to explicitly allow adopt updates (separate from `--yes`)
- [x] Refuse apply when adopt updates exist without explicit adopt flag
- [x] Add stable JSON error code `E_ADOPT_CONFIRM_REQUIRED`

## 3. Tests
- [x] Update `tests/golden/plan_codex.json` for new plan fields
- [x] Add CLI test that deploy --json --yes refuses adopt without explicit adopt flag

## 4. Validation
- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy --all-targets --all-features -- -D warnings`
- [x] `cargo test --all --locked`
- [x] `openspec validate add-adopt-confirmation --strict --no-interactive`
