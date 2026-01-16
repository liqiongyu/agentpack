## 1. Contract (#321)
- [x] Define template + docs expectations in spec delta
- [x] Run `openspec validate update-minimal-repo-template --strict --no-interactive`

## 2. Implementation
- [ ] Add a minimal Codex skill module under `docs/examples/minimal_repo/modules/skills/.../SKILL.md`
- [ ] Update `docs/examples/minimal_repo/agentpack.yaml` to reference the new module
- [ ] Update `docs/examples/minimal_repo/README.md` with a one-screen quickstart (including `bootstrap`)
- [ ] Update `docs/WORKFLOWS.md` (and/or `docs/CLI.md`) to link the example template + quickstart

## 3. Tests
- [ ] Add a CLI test that runs `agentpack --repo docs/examples/minimal_repo plan` successfully

## 4. Archive
- [ ] After shipping: `openspec archive update-minimal-repo-template --yes`
- [ ] Run `openspec validate --all --strict --no-interactive`
