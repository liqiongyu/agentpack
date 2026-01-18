## 1. Contract (#391)
- [ ] Update OpenSpec deltas (agentpack + agentpack-cli) for `zed`
- [ ] Run `openspec validate add-target-zed --strict --no-interactive`

## 2. Implementation
- [ ] Add Cargo feature `target-zed` (default-enabled)
- [ ] Validate `targets.zed` config (project scope only)
- [ ] Add `zed` to target registry + adapter routing
- [ ] Implement rendering:
  - root: `<project_root>` (`scan_extras=false`)
  - output: `<project_root>/.rules` from `instructions` modules
- [ ] Ensure manifest is written as `<project_root>/.agentpack.manifest.zed.json`

## 3. Tests
- [ ] Add conformance smoke test for `zed`
- [ ] Update golden snapshots impacted by target list changes (e.g. `help --json`)

## 4. Docs
- [ ] Update `docs/TARGETS.md` + `docs/zh-CN/TARGETS.md` with `zed` mapping
- [ ] Update any other relevant docs/spec references

## 5. Archive
- [ ] After shipping: `openspec archive add-target-zed --yes`
- [ ] Run `openspec validate --all --strict --no-interactive`
