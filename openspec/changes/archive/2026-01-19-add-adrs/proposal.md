# Change: Add ADRs for key architectural decisions

## Why

Some core design decisions (e.g., JSON contract stability, patch overlays, MCP confirm tokens) are critical to long-term maintenance and contributor onboarding. Keeping the rationale only in workplans or scattered docs makes it hard to find, easy to regress, and encourages duplicating “why” across documents.

## What Changes

- Introduce an ADR directory under `docs/adr/`.
- Add at least three initial ADRs covering:
  - JSON contract stability (`--json` schema_version=1, additive-only).
  - Patch overlays design (unified diffs under `.agentpack/patches/`, failure modes).
  - MCP `confirm_token` design (two-phase plan/apply, binding and auditability).
- Update `docs/dev/roadmap.md` to link to ADRs for “why” rather than embedding long rationale inline.

## Impact

- Affected specs: `agentpack-cli`
- Affected code: none (docs/spec only)
- Affected docs:
  - `docs/adr/*`
  - `docs/dev/roadmap.md`
