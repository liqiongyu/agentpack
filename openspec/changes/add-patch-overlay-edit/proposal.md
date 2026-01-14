# Change: Patch overlay edit

## Why
Patch overlays are optimized for “edit a few lines” workflows, but today users must create the patch-overlay layout manually.

Adding an explicit `overlay edit` flag makes patch overlays discoverable, consistent, and less error-prone.

## What Changes
- Add a flag to `agentpack overlay edit` to create a patch overlay skeleton (metadata + `.agentpack/patches/`).
- Define how this flag interacts with existing `--sparse` / `--materialize` behavior.

## Impact
- Affected CLI contract: `agentpack overlay edit`
- Affected docs: `docs/SPEC.md` (overlay edit semantics)
- Backward compatibility: default behavior remains directory overlays; patch overlays are opt-in.
