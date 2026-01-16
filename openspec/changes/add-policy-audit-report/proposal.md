## Why

Governance features need a CI-friendly way to generate an audit report of what is being consumed and pinned:
- module sources and pinned versions (from `repo/agentpack.lock.json`), and
- (optionally) the pinned policy pack version (from `repo/agentpack.org.lock.json`).

This supports supply-chain review and makes changes easier to inspect in PRs.

## What changes

- Add a new read-only command: `agentpack policy audit`.
- In `--json` mode, the command outputs a structured audit report including:
  - module ids, types, sources, pinned versions, and content hashes from `repo/agentpack.lock.json`
  - an optional lockfile change summary when git history is available (best-effort; no network)
  - optional governance policy pack lock info when `repo/agentpack.org.lock.json` exists

## Impact

- Adds a new CLI command (additive).
- No changes to existing `--json` schemas for existing commands (except `help --json` command catalog, which is additive).
