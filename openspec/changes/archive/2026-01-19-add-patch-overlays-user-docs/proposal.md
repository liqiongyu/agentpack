# Change: Add patch overlays user documentation

## Why
Patch overlays (`overlay_kind=patch` / `agentpack overlay edit --kind patch`) are implemented and covered by tests, but user-facing docs do not mention how to create or use them. This creates a “feature exists but users can’t find it” gap.

## What Changes
- Document patch overlays in the user docs:
  - `docs/OVERLAYS.md` and `docs/zh-CN/OVERLAYS.md`
  - `docs/CLI.md` and `docs/zh-CN/CLI.md` (include `--kind dir|patch`)
- Include at least one conflict/failure example and point to the stable error codes:
  - `E_OVERLAY_PATCH_APPLY_FAILED`
  - `E_OVERLAY_REBASE_CONFLICT` + conflict artifacts under `.agentpack/conflicts/`

## Impact
- Affected specs: `agentpack-cli`
- Affected docs: overlays + CLI reference
- Compatibility: no CLI/JSON behavior changes
