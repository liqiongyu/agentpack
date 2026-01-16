# Change: expand CLI JSON golden snapshots for core commands

## Why

`--json` output is a stable automation contract. We already have partial golden coverage, but gaps mean regressions can slip in (field renames, ordering drift, path normalization issues, missing `--yes` guardrails) without CI catching them.

We need broader, deterministic JSON golden snapshots for the core command suite so any contract changes are explicit and reviewed.

## What Changes

- Expand CLI JSON golden snapshot coverage for core commands (success paths), including at minimum:
  - `init`, `update`
  - `plan`, `diff`, `preview`
  - `overlay path`
  - `evolve` (at least one representative path)
- Ensure snapshots are cross-platform stable (path normalization, deterministic ordering, stable placeholders for ephemeral ids).

## Non-Goals

- Do not change CLI behavior or JSON schema in this change.
- Do not introduce new error codes.

## Impact

- Affected specs: `openspec/specs/agentpack-cli/spec.md`
- Affected tests: `tests/cli_json_golden*.rs`, `tests/golden/*.json`
- Affected docs: none (unless a mismatch or drift is discovered)
