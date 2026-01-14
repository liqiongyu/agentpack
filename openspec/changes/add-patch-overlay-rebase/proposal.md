# Change: Patch overlay rebase

## Why
Patch overlays reduce churn for small edits, but they still need a safe way to follow upstream changes over time.

Without a rebase workflow, patches will eventually stop applying cleanly as upstream files evolve, causing `plan/diff/deploy` to fail.

## What Changes
This change extends `agentpack overlay rebase` to support `overlay_kind=patch` overlays:
- Rebase updates patch files under `.agentpack/patches/` against the latest upstream content.
- Rebase uses the existing baseline metadata (`.agentpack/baseline.json`) as the merge base.
- Conflicts produce a stable error code (`E_OVERLAY_REBASE_CONFLICT`) and write conflict artifacts for manual resolution.

## Impact
- Affected CLI contract: `agentpack overlay rebase`
- Affected docs: `docs/SPEC.md` (overlay rebase semantics)
- Backward compatibility: directory overlays remain unchanged; patch overlays are opt-in via `overlay_kind=patch`.
