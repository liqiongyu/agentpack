## 1. Spec (contract)
- [x] Define `agentpack policy lint` behavior (human + `--json`)
- [x] Define policy lint output shape (issues list + summary)
- [x] Define stable error code for policy violations (`E_POLICY_VIOLATIONS` or equivalent)
- [x] Run `openspec validate add-policy-lint --strict --no-interactive`

## 2. Implementation (CLI)
- [x] Add `policy` subcommand and `policy lint`
- [x] Implement lint checks (skills, commands, dangerous defaults)
- [x] Ensure `--json` is machine-readable and stable
- [x] Add tests (unit + integration)
- [x] Update `agentpack help --json` golden snapshot if command list changes

## 3. Docs
- [x] Update `docs/SPEC.md` to document `policy lint`
- [x] Update `docs/ERROR_CODES.md` with the new stable error code
- [x] Update docs index as needed (`docs/README.md`)

## 4. Archive
- [ ] After shipping: `openspec archive add-policy-lint --yes`
- [ ] Run `openspec validate --all --strict --no-interactive`
