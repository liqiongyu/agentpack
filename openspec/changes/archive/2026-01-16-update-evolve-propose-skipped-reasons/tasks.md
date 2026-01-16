## 1. Contract (M1-E3-T2 / #315)
- [x] Define additive skipped item fields and stable reason_code set
- [x] Run `openspec validate update-evolve-propose-skipped-reasons --strict --no-interactive`

## 2. Implementation
- [x] Add `reason_code`, `reason_message`, and `next_actions` to `evolve.propose --json` skipped items
- [x] Keep existing `reason`/`suggestions` fields for compatibility

## 3. Docs
- [x] Update `docs/JSON_API.md` evolve.propose skipped payload documentation
- [x] Update `docs/SPEC.md` evolve propose notes to reference structured skipped reasons

## 4. Tests
- [x] Update `tests/cli_evolve_propose_skipped.rs` assertions to cover new fields
- [x] Run `cargo test --all --locked`

## 5. Archive
- [x] After shipping: `openspec archive update-evolve-propose-skipped-reasons --yes`
- [x] Run `openspec validate --all --strict --no-interactive`
