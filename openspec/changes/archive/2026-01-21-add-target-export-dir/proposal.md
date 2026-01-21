# Change: Add experimental `export_dir` target (feature-gated)

## Why
Agentpack targets are currently tool-specific. Adding a generic “export to directory” target makes it easy to:
- inspect compiled assets without installing a specific tool,
- integrate with downstream systems that want a filesystem tree,
- serve as a small, reviewable example for adding new targets (options, mapping rules, conformance).

This target is **experimental** and shipped behind a feature gate so it does not change default builds.

## What Changes
- Add a new target id: `export_dir` (files mode), behind Cargo feature `target-export-dir` (default off).
- Add target mapping:
  - `instructions` → `AGENTS.md` under the export root (aggregated with module markers when multiple modules contribute)
  - `skill` → `skills/<skill_name>/...`
  - `prompt` → `prompts/<file>.md`
  - `command` → `commands/<file>.md`
- Require `targets.export_dir.options.root` to define the export root directory.
- Add conformance coverage and CI matrix entry for the new feature.
- Add mapping docs (based on `docs/TARGET_MAPPING_TEMPLATE.md`) and mark the target as `experimental` in targets reference docs.

## Impact
- Affected specs: `agentpack` (new target adapter + conformance)
- Affected code: targets registry + config validation + docs + conformance tests
- Compatibility: no behavior change in default builds (feature-gated target)
