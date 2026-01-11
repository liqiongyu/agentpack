# Change: Overlay edit by scope (global/machine/project)

## Why
v0.2 supports `overlay edit` for global and (via `--project`) project overlays, but machine overlays lack an equivalent one-command UX. This creates daily friction (manual directory creation) and makes AI-first workflows harder to script.

## What Changes
- Add `agentpack overlay edit <module_id> --scope global|machine|project` (default: global).
- Keep `--project` for backward compatibility, but deprecate it (maps to `--scope project`).
- JSON output includes scope and the resolved overlay path (plus machine_id/project_id where relevant).

## Acceptance
- Machine overlays can be created/edited via `overlay edit --scope machine`.
- Existing `overlay edit --project` continues to work.
