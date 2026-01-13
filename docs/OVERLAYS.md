# Overlays

> Language: English | [Chinese (Simplified)](zh-CN/OVERLAYS.md)

Overlays let you customize upstream modules without forking, while keeping updates as mergeable, reviewable, and rollbackable as possible.

## 1) Layers and precedence

The final materialized content for a module is composed from four layers (low → high):
1) upstream (local_path or git checkout)
2) global overlay
3) machine overlay
4) project overlay

For the same path, higher-precedence files override lower-precedence ones.

## 2) On-disk layout

Inside the config repo:
- global: `repo/overlays/<module_fs_key>/...`
- machine: `repo/overlays/machines/<machine_id>/<module_fs_key>/...`
- project: `repo/projects/<project_id>/overlays/<module_fs_key>/...`

Notes:
- `module_fs_key` is a filesystem-safe directory name derived from `module_id` (sanitized + short hash, with a max prefix length to avoid overly long paths).
- The CLI and manifest still use `module_id`; `module_fs_key` is for disk addressing only.
- For compatibility, agentpack may try legacy overlay directory names if present.

## 3) Overlay metadata (`.agentpack/`)

Each overlay directory contains:
- `.agentpack/baseline.json`: upstream fingerprint captured at overlay creation time (used for drift warnings and 3-way merge).
- `.agentpack/module_id`: the original module id (useful for auditing/diagnostics).

Rule:
- `.agentpack/` is reserved metadata and is never deployed to target roots.

## 4) Create/edit: `overlay edit`

Command:
- `agentpack overlay edit <module_id> [--scope global|machine|project] [--sparse|--materialize]`

Behavior:
- Default (no `--sparse/--materialize`):
  - If the overlay does not exist, it copies the full upstream module tree into the overlay, then opens `$EDITOR` (if set).
- `--sparse`:
  - Create a sparse overlay: create metadata only, do not copy upstream files.
  - Recommended: keep only the files you actually changed (smaller diffs; easier merges later).
- `--materialize`:
  - Copy upstream files into the overlay in a missing-only manner (does not overwrite existing overlay edits).
  - Useful when you want to browse upstream implementation without committing the whole tree into overlay diffs.

Note:
- `overlay edit` is mutating; in `--json` mode you must pass `--yes`.

## 5) Rebase after upstream updates: `overlay rebase` (3-way merge)

Command:
- `agentpack overlay rebase <module_id> [--scope ...] [--sparsify]`

Purpose:
- After the upstream module changes, re-apply your overlay edits onto the new upstream and auto-resolve simple cases when possible.

Key behaviors:
- Uses `.agentpack/baseline.json` as the merge base and performs 3-way merge for files in the overlay.
- For files copied into the overlay but not actually edited (`ours == base`), it updates them to the latest upstream to avoid unintentionally pinning old versions.
- `--sparsify`: deletes overlay files that become identical to upstream after rebase, keeping overlays sparse.
- Supports `--dry-run`: report what would happen without writing.

Conflicts:
- On conflicts, the command fails with `E_OVERLAY_REBASE_CONFLICT`; `details` includes the conflict file list.
- Resolve conflicts manually in the overlay directory, then re-run `overlay rebase` (or commit the overlay changes directly).

## 6) `overlay path`

Command:
- `agentpack overlay path <module_id> [--scope ...]`

Purpose:
- Print the overlay directory (human) or provide it in JSON (`data.overlay_dir`) so scripts/agents can open it directly.

## 7) Practical tips

- Prefer `--sparse` to keep overlays small and easy to merge.
- Use `--materialize` only when you need to browse upstream files.
- After upstream updates: run `agentpack update`, then `agentpack overlay rebase ...`, then `preview --diff` to inspect changes.
