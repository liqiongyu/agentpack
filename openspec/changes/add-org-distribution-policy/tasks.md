## 1. Spec (contract)
- [ ] Define `distribution_policy` schema in `repo/agentpack.org.yaml`.
- [ ] Define `agentpack policy lint` enforcement rules + machine-readable violation details.
- [ ] Confirm stable error-code behavior (reuse `E_POLICY_VIOLATIONS`).
- [ ] Run `openspec validate add-org-distribution-policy --strict --no-interactive`.

## 2. Implementation
- [ ] Extend `OrgConfig` parsing with `distribution_policy`.
- [ ] Implement `policy lint` checks against `repo/agentpack.yaml` (required targets/modules).
- [ ] Add/update tests for distribution policy violations and success cases.

## 3. Docs
- [ ] Update `docs/SPEC.md` (org config schema + policy lint rules).
- [ ] Update `docs/GOVERNANCE.md` (distribution policy overview + CI usage).

## 4. Archive
- [ ] After shipping: `openspec archive add-org-distribution-policy --yes`.
- [ ] Run `openspec validate --all --strict --no-interactive`.
