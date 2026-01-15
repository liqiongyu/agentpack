## 1. Spec (contract)
- [ ] Define `repo/agentpack.org.yaml` (policy pack reference)
- [ ] Define `repo/agentpack.org.lock.json` (pinning + hashing)
- [ ] Define `agentpack policy lock` behavior (human + `--json`)
- [ ] Define how `policy lint` uses the pinned lockfile (no network)
- [ ] Define stable error codes for policy config/lock failures (if needed)
- [ ] Run `openspec validate add-policy-pack-lock --strict --no-interactive`

## 2. Implementation (CLI)
- [ ] Parse and validate `agentpack.org.yaml` (policy subcommands only)
- [ ] Implement `policy lock` (resolve source + write lockfile)
- [ ] Update `policy lint` to validate/pin policy pack usage
- [ ] Register mutating command id and guardrails (`MUTATING_COMMAND_IDS`, help snapshot, tests)
- [ ] Add tests (lock determinism, json error codes, lint integration)

## 3. Docs
- [ ] Update `docs/SPEC.md` (governance config + lock + policy lock)
- [ ] Update `docs/ERROR_CODES.md` (new stable codes, if added)
- [ ] Update `docs/GOVERNANCE.md` (policy pack usage + CI guidance)

## 4. Archive
- [ ] After shipping: `openspec archive add-policy-pack-lock --yes`
- [ ] Run `openspec validate --all --strict --no-interactive`
