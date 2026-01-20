# Overlays: sparse → materialize → rebase

> Language: English | [Chinese (Simplified)](../zh-CN/howto/overlays-create-sparse-materialize-rebase.md)

This how-to walks through the most common overlays workflow:

1) create a **sparse** overlay (minimal diffs)
2) optionally **materialize** upstream files for browsing
3) **rebase** after upstream updates, keeping the overlay sparse

For the conceptual model and on-disk layout, see `docs/explanation/overlays.md`.

## 0) Prerequisites

- You have a module id you want to customize (example: `skill:my-skill`).
- You can run a normal loop successfully:
  - `agentpack update`
  - `agentpack preview --diff`
  - `agentpack deploy --apply`

## 1) Create a sparse overlay (recommended)

Create an overlay that starts empty (metadata only), then add only the files you actually want to change.

- `agentpack overlay edit <module_id> --sparse`

Notes:
- Use `--scope global|machine|project` if you want to control precedence explicitly.
- In automation (`--json`), `overlay edit` is mutating and requires `--yes`.

## 2) Patch overlays for small, reviewable edits (optional)

If you only need to tweak a few lines in an upstream file, patch overlays avoid copying whole files into the overlay tree.

1) Create a patch overlay:
- `agentpack overlay edit <module_id> --kind patch --scope global`

2) Add a unified diff patch under `.agentpack/patches/` (example: `.agentpack/patches/SKILL.md.patch`).

If the patch cannot be applied during `plan`/`deploy`, the command fails with stable error code `E_OVERLAY_PATCH_APPLY_FAILED` (see `docs/reference/error-codes.md`).

## 3) Materialize upstream files for browsing (optional)

If you want to browse upstream implementation without committing the whole tree into diffs:

- `agentpack overlay edit <module_id> --materialize`

This copies upstream files into the overlay in a missing-only manner (it does not overwrite your overlay edits).

## 4) Rebase after upstream updates

After upstream changes (typically after `agentpack update`), rebase your overlay onto the new upstream:

- `agentpack overlay rebase <module_id> --sparsify`

Recommended workflow:
1) `agentpack update`
2) `agentpack overlay rebase <module_id> --sparsify`
3) `agentpack preview --diff`
4) `agentpack deploy --apply`

Dry-run first (no writes):
- `agentpack overlay rebase <module_id> --sparsify --dry-run`

## 5) Conflict handling

If `overlay rebase` reports `E_OVERLAY_REBASE_CONFLICT`:

1) Open the conflict-marked files under the overlay directory and resolve manually.
2) Re-run:
   - `agentpack overlay rebase <module_id>`

For patch overlays, a copy of the conflicted merged file is also written under:

- `.agentpack/conflicts/<relpath>` (example: `.agentpack/conflicts/SKILL.md`)
