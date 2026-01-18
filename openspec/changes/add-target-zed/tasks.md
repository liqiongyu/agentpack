## 1. Contract (#391)
- [x] Update OpenSpec deltas (agentpack + agentpack-cli) for `zed`
- [x] Run `openspec validate add-target-zed --strict --no-interactive`

## 2. Implementation
- [x] Add Cargo feature `target-zed` (default-enabled)
- [x] Validate `targets.zed` config (project scope only)
- [x] Add `zed` to target registry + adapter routing
- [x] Implement rendering:
  - root: `<project_root>` (`scan_extras=false`)
  - output: `<project_root>/.rules` from `instructions` modules
- [x] Ensure manifest is written as `<project_root>/.agentpack.manifest.zed.json`

## 3. Tests
- [x] Add conformance smoke test for `zed`
- [x] Update golden snapshots impacted by target list changes (e.g. `help --json`)

## 4. Docs
- [x] Update `docs/TARGETS.md` + `docs/zh-CN/TARGETS.md` with `zed` mapping
- [x] Update any other relevant docs/spec references

## 5. Archive
- [ ] After shipping: `openspec archive add-target-zed --yes`
- [ ] Run `openspec validate --all --strict --no-interactive`
