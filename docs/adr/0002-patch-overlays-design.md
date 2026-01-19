# ADR 0002: Patch overlays design

- Status: accepted
- Date: 2026-01-19

## Context

Overlays let users customize upstream modules without forking. Traditional “dir overlays” materialize full files, but that makes updates expensive: rebases produce large diffs, and small upstream changes can require re-copying files.

Users often want a minimal, reviewable customization that:
- keeps the upstream content as the source of truth
- stores only the delta in version control
- can be rebased when upstream changes

## Decision

Introduce “patch overlays” as a first-class overlay kind:
- `overlay_kind=patch` overlays store unified diff patches under `.agentpack/patches/`.
- Patch overlays are intended for UTF-8 text files.
- During desired-state generation, patches are applied to the upstream content to produce the final materialized files.
- Failures are treated as actionable, deterministic errors:
  - patch apply failures surface as `E_OVERLAY_PATCH_APPLY_FAILED`
  - rebase conflicts surface as `E_OVERLAY_REBASE_CONFLICT` and write conflict artifacts under `.agentpack/conflicts/`

## Consequences

- Pros:
  - small diffs and clean review surface
  - upstream updates are easier to rebase onto
  - changes remain local and auditable
- Cons:
  - patching is less suitable for binary files or non-UTF-8 content
  - patch failures require conflict resolution (but with deterministic artifacts)

## References

- `docs/explanation/overlays.md`
- `docs/reference/error-codes.md`
