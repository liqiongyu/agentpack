## 1. Spec (contract)
- [x] Define `distribution_policy` schema in `repo/agentpack.org.yaml`.
- [x] Define `agentpack policy lint` enforcement rules + machine-readable violation details.
- [x] Confirm stable error-code behavior (reuse `E_POLICY_VIOLATIONS`).
- [x] Run `openspec validate add-org-distribution-policy --strict --no-interactive`.

## 2. Implementation
- [x] Extend `OrgConfig` parsing with `distribution_policy`.
- [x] Implement `policy lint` checks against `repo/agentpack.yaml` (required targets/modules).
- [x] Add/update tests for distribution policy violations and success cases.

## 3. Docs
- [x] Update `docs/SPEC.md` (org config schema + policy lint rules).
- [x] Update `docs/GOVERNANCE.md` (distribution policy overview + CI usage).

## 4. Archive
- [ ] After shipping: `openspec archive add-org-distribution-policy --yes`.
- [ ] Run `openspec validate --all --strict --no-interactive`.
