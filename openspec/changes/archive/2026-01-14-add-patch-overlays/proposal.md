# Change: Patch-based overlays (spec + format)

## Why
Directory overlays work well for “replace a whole file”, but for small edits they introduce churn and hard-to-review diffs (copying entire upstream files into overlays).

Patch overlays aim to reduce overlay noise by storing only the textual diff against upstream, while preserving:
- backward compatibility with existing directory overlays
- explicit, deterministic overlay precedence
- safe rebase semantics (3-way merge against a known baseline)

## What Changes
This change defines the **format and contract** for patch overlays and implements patch application during desired-state generation:
- Introduce an overlay kind indicator: `overlay_kind: "dir" | "patch"` (default = `dir` for existing overlays).
- Define patch storage layout under `.agentpack/patches/` within an overlay directory.
- Define applicability constraints (text-only, UTF-8) and non-goals (no binary patching).
- Define behavior when both dir and patch artifacts are present (kind conflict).
- Apply patch overlays during desired-state generation (plan/diff/deploy), returning a stable error code on failure.

## Non-goals
- Implement patch overlay rebase (handled in follow-up items).
- Add new CLI surface area (e.g. `overlay edit --kind patch` handled in follow-up).

## Impact
- Affected docs/specs: `docs/SPEC.md`, `openspec/specs/agentpack/spec.md`, `openspec/specs/agentpack-cli/spec.md`.
- Backward compatibility: existing overlays without `overlay_kind` remain directory overlays.
