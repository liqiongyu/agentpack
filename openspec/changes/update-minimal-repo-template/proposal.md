# Change: update minimal config repo template

## Why

Day-1 adoption benefits from a copy/pasteable example repo that includes a small but complete set of modules (instructions + prompt + skill) and a “one-screen” command sequence. This reduces time-to-first-success for both humans and agents.

## What Changes

- Extend `docs/examples/minimal_repo/` to include:
  - an example Codex skill module (with `SKILL.md`)
  - an updated `agentpack.yaml` that references the added module
- Update docs to include a “one-screen” quickstart that:
  - references `docs/examples/minimal_repo/`
  - recommends installing operator `/ap-*` commands via `agentpack bootstrap`

## Non-Goals

- Do not change CLI behavior or flags.
- Do not change the bootstrap operator asset content.

## Impact

- Affected specs: `openspec/specs/agentpack-cli/spec.md`
- Affected docs: `docs/WORKFLOWS.md` (and/or `docs/CLI.md`)
- Affected example files: `docs/examples/minimal_repo/...`
- Tests: add a small CLI test to ensure the example repo remains usable for `plan`
