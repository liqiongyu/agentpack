# Change: update import mapping rules

## Why

`agentpack import` is intended to be the “day-1 adoption” path. For that to work, import output must be predictable and explainable across tools and scopes.

The current `import` implementation is an MVP: it imports assets safely, but its mapping rules (module ids, tags, and scope separation) are not yet fully specified or documented, and there are avoidable collision cases (e.g. same skill name in user + project).

## What Changes

- Define deterministic mapping rules for `import` (module ids, tags, targets, and destination paths).
- Ensure user-scope and project-scope candidates do not silently collide in module ids (prefer collision-free ids by construction).
- Document the mapping with 3 common examples (repo-only, user-only, mixed) so users and agents can predict results.

## Non-Goals

- Do not add overwrite/adopt behavior for existing config repo paths (handled separately).
- Do not generate overlays as part of import (future work).

## Impact

- Affected specs: `openspec/specs/agentpack-cli/spec.md`
- Affected code: `src/cli/commands/import.rs`
- Affected docs: `docs/WORKFLOWS.md` (examples), possibly `docs/SPEC.md` / `docs/JSON_API.md`
- Tests: extend `tests/cli_import.rs` to assert mapping determinism
