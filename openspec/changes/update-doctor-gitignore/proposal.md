# Change: Doctor warns about manifest gitignore (optional `--fix`)

## Why
`agentpack deploy --apply` writes `.agentpack.manifest.json` into target roots. If a target root is inside a git repo and this file is not ignored, users can accidentally commit it.

## What Changes
- Enhance `agentpack doctor` to detect target roots that are inside a git repo where `.agentpack.manifest.json` is not ignored, and emit a warning.
- Add `agentpack doctor --fix` to idempotently append `.agentpack.manifest.json` to the detected repo’s `.gitignore`.
- In `--json` mode, `doctor --fix` requires `--yes` (refuse with `E_CONFIRM_REQUIRED`).

## Acceptance
- Doctor reports a warning when a target root is inside a git repo and the manifest is not ignored.
- `doctor --fix` makes the warning go away and does not add duplicate entries.
- Behavior is tested and documented in `docs/SPEC.md`.
