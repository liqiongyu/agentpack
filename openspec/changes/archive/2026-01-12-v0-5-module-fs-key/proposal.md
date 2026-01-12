# Change: module_fs_key for overlays and store paths (v0.5)

## Why
`module_id` is a logical identifier and must not be used directly as a filesystem path component. In practice it breaks on Windows (e.g. `instructions:base` contains `:`) and can introduce path traversal / collision risks.

## What Changes
- Introduce a stable `module_fs_key` derived from `module_id` (sanitized + short hash).
- Use `module_fs_key` for overlay directories (global/machine/project) and for store git checkout directories.
- Keep CLI/manifest/user-facing identifiers as `module_id` (no UX change).
- Backwards compatibility: if legacy paths exist (pre-v0.5), prefer them; otherwise create/use the new canonical paths.

## Impact
- Affected specs: `agentpack`, `agentpack-cli`
- Affected code: overlay path resolution, overlay edit/evolve propose writes, git store layout
