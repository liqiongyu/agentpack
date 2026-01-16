## 1. Contract (M1-E3-T1 / #314)
- [ ] Define additive `status --json` fields for grouped summary and structured next actions
- [ ] Run `openspec validate update-status-actionable-output --strict --no-interactive`

## 2. Implementation
- [ ] Add grouped drift summary output to `agentpack status --json`
- [ ] Add structured `next_actions` output (action code + command) to `agentpack status --json`
- [ ] Improve human `status` output to group by root with per-root summaries
- [ ] Update `agentpack schema --json` to document new `status` data fields

## 3. Docs
- [ ] Update `docs/JSON_API.md` status payload documentation
- [ ] Update `docs/SPEC.md` status JSON notes (additive fields)

## 4. Tests
- [ ] Update JSON golden snapshots for `status` and `schema`
- [ ] Run `cargo test --all --locked`

## 5. Archive
- [ ] After shipping: `openspec archive update-status-actionable-output --yes`
- [ ] Run `openspec validate --all --strict --no-interactive`
