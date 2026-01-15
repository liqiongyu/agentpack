## 1. Spec (contract)
- [ ] Define `agentpack policy lint` behavior (human + `--json`)
- [ ] Define policy lint output shape (issues list + summary)
- [ ] Define stable error code for policy violations (`E_POLICY_VIOLATIONS` or equivalent)
- [ ] Run `openspec validate add-policy-lint --strict --no-interactive`

## 2. Implementation (CLI)
- [ ] Add `policy` subcommand and `policy lint`
- [ ] Implement lint checks (skills, commands, dangerous defaults)
- [ ] Ensure `--json` is machine-readable and stable
- [ ] Add tests (unit + integration)
- [ ] Update `agentpack help --json` golden snapshot if command list changes

## 3. Docs
- [ ] Update `docs/SPEC.md` to document `policy lint`
- [ ] Update `docs/ERROR_CODES.md` with the new stable error code
- [ ] Update docs index as needed (`docs/README.md`)

## 4. Archive
- [ ] After shipping: `openspec archive add-policy-lint --yes`
- [ ] Run `openspec validate --all --strict --no-interactive`
