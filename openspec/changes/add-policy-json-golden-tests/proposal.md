# Change: Add golden snapshots for policy lint/lock JSON output

## Why
Automation relies on `--json` output as an API contract. `agentpack policy lint` / `agentpack policy lock`
already exist, but we do not have golden snapshots that protect the stability of their JSON `data` payloads
for downstream tooling.

## What Changes
- Add golden snapshot tests for:
  - `agentpack policy lock --json --yes` (success `data`)
  - `agentpack policy lint --json` (success `data`, after lock)
- Normalize temp paths so snapshots are deterministic across OS.

## Scope
- Tests and snapshots only (no behavior changes).

## Acceptance
- Golden snapshots are deterministic (normalized temp paths, stable ordering).
- No changes to the `--json` envelope contract (`schema_version=1`).
