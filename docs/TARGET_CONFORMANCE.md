# TARGET_CONFORMANCE.md

Conformance tests are the quality bar for targets.

## Required semantics
1. Delete protection: plan/apply only delete manifest-managed paths.
2. Manifest: apply writes per-root `.agentpack.manifest.json`.
3. Drift: status distinguishes `missing`/`modified`/`extra` (extras are not auto-deleted).
4. Rollback: restores create/update/delete effects.
5. JSON contract: envelope fields and key error codes remain stable.

## Recommended harness approach
- Use temp directories as fake target roots.
- Run the real pipeline (`deploy --apply`, `status`, `rollback`) against those roots.
- Keep tests hermetic: avoid writing to real `~/.codex` or `~/.claude`.
