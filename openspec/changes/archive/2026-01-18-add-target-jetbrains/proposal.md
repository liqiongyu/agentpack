# Change: Add JetBrains target adapter (Junie guidelines)

## Why
JetBrains users (and agents running inside JetBrains IDEs) benefit from a “first-class” target that writes the default Junie guideline file location. This reduces setup friction vs. manually configuring Junie to read `AGENTS.md`, while staying consistent with Agentpack’s safety model (manifests, drift detection, rollback).

## What Changes
- Add a new target id: `jetbrains` (files mode).
- Render `instructions` modules into `<project_root>/.junie/guidelines.md`.
  - Multi-module output uses per-module section markers to preserve attribution.
- Add a Cargo feature flag `target-jetbrains`.
  - Include the feature in the default feature set so official binaries support JetBrains out-of-the-box (while still allowing minimal builds via `--no-default-features`).
- Add conformance coverage for the new target, including per-root manifests, delete protection, drift classification, and rollback.
- Update target documentation (mapping rules, examples, migration notes).

## Non-goals
- Managing JetBrains user-level settings.
- Managing `.aiignore` (repo root collisions with other targets must be solved separately).
- Managing JetBrains prompts/skills/commands formats (if any); this change focuses on Junie guidelines.

## Impact
- Affected OpenSpec capabilities:
  - `openspec/specs/agentpack/spec.md` (new target behavior and conformance coverage)
  - `openspec/specs/agentpack-cli/spec.md` (feature-gated target list)
- User-visible docs:
  - `docs/TARGETS.md` and `docs/zh-CN/TARGETS.md`
  - `docs/SPEC.md` (supported targets list / target section)
- Tracking issue: #392
