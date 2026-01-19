# Change: Consolidate Codex execution docs into a single active guide

## Why
The repository currently has multiple “Codex plan/workplan” documents, which makes it unclear which one is the canonical execution guide and increases drift over time.

## What Changes
- Add a single active execution guide at `docs/dev/codex.md` (with required YAML frontmatter).
- Move legacy Codex planning docs into `docs/archive/plans/` and mark them as superseded (YAML frontmatter + pointer to the new guide).

## Impact
- Affected specs: `agentpack-cli` (documentation governance requirements)
- Affected docs: `docs/dev/codex.md`, `docs/archive/plans/*`, and references in contributor docs
- Compatibility: no CLI/JSON behavior changes
