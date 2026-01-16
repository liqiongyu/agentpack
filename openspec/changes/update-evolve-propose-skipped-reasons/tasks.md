## 1. Contract (M1-E3-T2 / #315)
- [ ] Define additive skipped item fields and stable reason_code set
- [ ] Run `openspec validate update-evolve-propose-skipped-reasons --strict --no-interactive`

## 2. Implementation
- [ ] Add `reason_code`, `reason_message`, and `next_actions` to `evolve.propose --json` skipped items
- [ ] Keep existing `reason`/`suggestions` fields for compatibility

## 3. Docs
- [ ] Update `docs/JSON_API.md` evolve.propose skipped payload documentation
- [ ] Update `docs/SPEC.md` evolve propose notes to reference structured skipped reasons

## 4. Tests
- [ ] Update `tests/cli_evolve_propose_skipped.rs` assertions to cover new fields
- [ ] Run `cargo test --all --locked`

## 5. Archive
- [ ] After shipping: `openspec archive update-evolve-propose-skipped-reasons --yes`
- [ ] Run `openspec validate --all --strict --no-interactive`
