## 1. Spec (contract)
- [x] Define `repo/agentpack.org.yaml` (policy pack reference)
- [x] Define `repo/agentpack.org.lock.json` (pinning + hashing)
- [x] Define `agentpack policy lock` behavior (human + `--json`)
- [x] Define how `policy lint` uses the pinned lockfile (no network)
- [x] Define stable error codes for policy config/lock failures (if needed)
- [x] Run `openspec validate add-policy-pack-lock --strict --no-interactive`

## 2. Implementation (CLI)
- [x] Parse and validate `agentpack.org.yaml` (policy subcommands only)
- [x] Implement `policy lock` (resolve source + write lockfile)
- [x] Update `policy lint` to validate/pin policy pack usage
- [x] Register mutating command id and guardrails (`MUTATING_COMMAND_IDS`, help snapshot, tests)
- [x] Add tests (lock determinism, json error codes, lint integration)

## 3. Docs
- [x] Update `docs/SPEC.md` (governance config + lock + policy lock)
- [x] Update `docs/ERROR_CODES.md` (new stable codes, if added)
- [x] Update `docs/GOVERNANCE.md` (policy pack usage + CI guidance)

## 4. Archive
- [ ] After shipping: `openspec archive add-policy-pack-lock --yes`
- [ ] Run `openspec validate --all --strict --no-interactive`
