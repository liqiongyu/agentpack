## 1. Contract (M1-E2-T1 / #309)
- [ ] Define `init --guided` behavior + prompts + output requirements
- [ ] Define stable error code for non-TTY guided init in `--json` mode
- [ ] Run `openspec validate add-init-guided --strict --no-interactive`

## 2. Implementation
- [ ] Add CLI flag `init --guided`
- [ ] Implement minimal TTY wizard (targets, scope, bootstrap)
- [ ] Generate a valid `agentpack.yaml` (deterministic ordering) based on answers
- [ ] Ensure non-TTY fails early (no writes)

## 3. Tests
- [ ] `init --guided --json` fails with stable error code when not a TTY
- [ ] Generated manifest can be loaded and used by `update/preview/deploy` in a temp env

## 4. Docs
- [ ] Document `init --guided` in `docs/CLI.md` and `docs/WORKFLOWS.md`
- [ ] Add the stable error code to `docs/ERROR_CODES.md`

## 5. Archive
- [ ] After shipping: `openspec archive add-init-guided --yes`
- [ ] Run `openspec validate --all --strict --no-interactive`
