## 1. Contract (M1-E3-T1 / #314)
- [x] Define additive `status --json` fields for grouped summary and structured next actions
- [x] Run `openspec validate update-status-actionable-output --strict --no-interactive`

## 2. Implementation
- [x] Add grouped drift summary output to `agentpack status --json`
- [x] Add structured `next_actions` output (action code + command) to `agentpack status --json`
- [x] Improve human `status` output to group by root with per-root summaries
- [x] Update `agentpack schema --json` to document new `status` data fields

## 3. Docs
- [x] Update `docs/JSON_API.md` status payload documentation
- [x] Update `docs/SPEC.md` status JSON notes (additive fields)

## 4. Tests
- [x] Update JSON golden snapshots for `status` and `schema`
- [x] Run `cargo test --all --locked`

## 5. Archive
- [x] After shipping: `openspec archive update-status-actionable-output --yes`
- [x] Run `openspec validate --all --strict --no-interactive`
