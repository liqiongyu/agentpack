## 1. Spec + Docs
- [ ] Add OpenSpec delta requirement for adopt confirmation
- [ ] Update `docs/SPEC.md` to document adopt behavior and error code

## 2. Implementation
- [ ] Extend plan output to mark adopt updates (machine-readable)
- [ ] Add deploy flag to explicitly allow adopt updates (separate from `--yes`)
- [ ] Refuse apply when adopt updates exist without explicit adopt flag
- [ ] Add stable JSON error code `E_ADOPT_CONFIRM_REQUIRED`

## 3. Tests
- [ ] Update `tests/golden/plan_codex.json` for new plan fields
- [ ] Add CLI test that deploy --json --yes refuses adopt without explicit adopt flag

## 4. Validation
- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --all-targets --all-features -- -D warnings`
- [ ] `cargo test --all --locked`
- [ ] `openspec validate add-adopt-confirmation --strict --no-interactive`
