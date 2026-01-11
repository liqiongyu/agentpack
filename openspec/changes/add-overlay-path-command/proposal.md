# Change: Add `agentpack overlay path`

## Why
AI agents and scripts often need to locate the overlay directory without duplicating path mapping logic (global/machine/project).

## What Changes
- Add `agentpack overlay path <module_id> --scope <global|machine|project>` (default: global).
- The command is read-only:
  - human: prints the absolute overlay directory path
  - json: `data.overlay_dir`

## Acceptance
- Path mapping matches `overlay edit --scope`.
- `--json` output is parseable and includes `data.overlay_dir`.
