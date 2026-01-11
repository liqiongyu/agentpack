# Change: Add `agentpack update` composite command

## Why
Daily usage often requires running `lock` and/or `fetch` in sequence. A single `update` command reduces friction and makes the common workflow easier for both humans and AI agents.

## What Changes
- Add `agentpack update` as a composite command that orchestrates `lock` and `fetch`.
- Default behavior:
  - If `agentpack.lock.json` is missing: run `lock` then `fetch`.
  - If `agentpack.lock.json` exists: run `fetch` only.
- Provide flags to force/skip each step: `--lock`, `--fetch`, `--no-lock`, `--no-fetch`.
- In `--json` mode, return aggregated step output (`data.steps`) and require `--yes` when the command will write.

## Scope
- CLI only; no change to lockfile format or store layout.
- Affected spec: `agentpack-cli`.

## Acceptance
- `agentpack update` runs the correct default steps depending on lockfile existence.
- `agentpack update --json` without `--yes` is refused with `E_CONFIRM_REQUIRED` when it would perform writes.
- Behavior is covered by tests and documented in `docs/SPEC.md`.
