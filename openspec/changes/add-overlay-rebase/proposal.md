# Change: add-overlay-rebase

## Why
Sparse overlays reduce repo bloat, but they do not solve the day-2 problem: when upstream module content changes, existing overlays can drift and require manual merging.

In particular, non-sparse overlays created by copying the upstream tree can unintentionally **pin** old upstream files (because overlay copies override upstream), making upstream upgrades painful.

## What Changes
- Add `agentpack overlay rebase <module_id>` to update an existing overlay against the current upstream using a 3-way merge, based on the overlay baseline metadata.
- Extend overlay baseline metadata (`<overlay_dir>/.agentpack/baseline.json`) with upstream identity data so rebasing can locate the original baseline content.
- Add stable `--json` error codes for common, actionable rebase failures (missing overlay/baseline, merge conflicts).

## Impact
- Affected specs: `agentpack-cli`
- Affected code: `src/overlay.rs`, `src/cli/args.rs`, `src/cli/commands/overlay.rs`, `src/cli/commands/help.rs`, `src/cli/util.rs`
- Tests: unit/integration tests for clean rebase and conflict reporting
- Docs: `docs/SPEC.md`, `docs/ERROR_CODES.md`, `CHANGELOG.md`

## Compatibility
This change is additive:
- New CLI subcommand and new `--json` fields are backwards compatible.
- Overlay baseline metadata remains backwards compatible (existing `baseline.json` without upstream identity continues to support drift warnings; rebasing may require additional data).
