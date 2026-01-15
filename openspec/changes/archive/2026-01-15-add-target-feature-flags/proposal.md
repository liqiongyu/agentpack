# Change: Target adapter compile-time feature flags

## Why
Agentpack’s TargetAdapters evolve quickly as upstream tools change. Being able to compile a smaller binary with only the targets you need helps:
- reduce build/dependency surface area,
- speed up iteration and CI,
- isolate target-specific churn,
- and make it easier to add new targets without entangling unrelated adapters.

## What Changes
- Define per-target Cargo features:
  - `target-codex`
  - `target-claude-code`
  - `target-cursor`
  - `target-vscode`
- Define a default feature set that includes all built-in targets (so `cargo build` preserves current behavior).
- Require `agentpack help --json` to expose the compiled target set (additive field) so automation can discover what the running binary supports.
- Define behavior when selecting a non-compiled target: fail with existing stable error code `E_TARGET_UNSUPPORTED` (in `--json` mode).

## Impact
- Affected OpenSpec capability: `openspec/specs/agentpack-cli/spec.md`
- Compatibility: additive only for `schema_version=1`
- Follow-up implementation work is tracked under: #218 (feature-gated adapters) and #219 (CI matrix)
