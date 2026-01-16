## 1. Contract (M1-E2-T1 / #309)
- [x] Define `init --guided` behavior + prompts + output requirements
- [x] Define stable error code for non-TTY guided init in `--json` mode
- [x] Run `openspec validate add-init-guided --strict --no-interactive`

## 2. Implementation
- [x] Add CLI flag `init --guided`
- [x] Implement minimal TTY wizard (targets, scope, bootstrap)
- [x] Generate a valid `agentpack.yaml` (deterministic ordering) based on answers
- [x] Ensure non-TTY fails early (no writes)

## 3. Tests
- [x] `init --guided --json` fails with stable error code when not a TTY
- [x] Generated manifest can be loaded and used by `update/preview/deploy` in a temp env

## 4. Docs
- [x] Document `init --guided` in `docs/CLI.md` and `docs/WORKFLOWS.md`
- [x] Add the stable error code to `docs/ERROR_CODES.md`

## 5. Archive
- [x] After shipping: `openspec archive add-init-guided --yes`
- [x] Run `openspec validate --all --strict --no-interactive`
