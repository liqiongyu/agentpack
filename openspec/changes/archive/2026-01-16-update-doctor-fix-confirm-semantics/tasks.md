## 1. Contract (M1-E3-T3 / #316)
- [x] Define `doctor --fix --json` guardrail requirement
- [x] Run `openspec validate update-doctor-fix-confirm-semantics --strict --no-interactive`

## 2. Implementation
- [x] Route `doctor --fix` through the central `--json` mutation guardrail (`E_CONFIRM_REQUIRED` without `--yes`)

## 3. Tests
- [x] Add `doctor --fix` coverage to `tests/cli_guardrails.rs`
- [x] Run `cargo test --all --locked`

## 4. Docs
- [x] Update `docs/SPEC.md` if behavior changes

## 5. Archive
- [x] After shipping: `openspec archive update-doctor-fix-confirm-semantics --yes`
- [x] Run `openspec validate --all --strict --no-interactive`
