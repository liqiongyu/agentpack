# Change: Add sparse overlay mode for `overlay edit`

## Why
`agentpack overlay edit` currently creates an overlay by copying the entire upstream module tree into the overlay directory. This is convenient for browsing, but it makes config repos grow linearly with upstream size and increases sync/merge cost.

Sparse overlays keep the overlay directory small by storing only modified files, while still composing correctly at deploy time (upstream + overlays).

## What Changes
- Add `--sparse` to `agentpack overlay edit` to create an overlay skeleton with metadata (baseline + module_id) but without copying upstream files.
- Add `--materialize` to explicitly populate upstream files into an overlay directory without overwriting existing overlay edits (escape hatch for editing convenience).

## Impact
- Affected specs: `agentpack-cli`
- Affected code: `src/cli/args.rs`, `src/cli/commands/overlay.rs`, `src/overlay.rs`, `src/fs.rs`
- Affected docs/tests: `docs/SPEC.md`, core overlay tests
