# Change: Consolidate roadmap docs into a single active dev roadmap

## Why
The repository currently contains multiple overlapping roadmap/spec snapshots. Consolidating to a single active roadmap reduces drift and makes it clear where maintainers should update milestones and priorities.

## What Changes
- Add/maintain a single active roadmap at `docs/dev/roadmap.md` (with required YAML frontmatter).
- Move older roadmap/spec snapshots into `docs/archive/roadmap/` and mark them as archived, with pointers to the active roadmap.

## Impact
- Affected specs: `agentpack-cli` (documentation governance requirements)
- Affected docs: `docs/dev/roadmap.md`, `docs/archive/roadmap/*`, and references in maintainer docs
- Compatibility: no CLI/JSON behavior changes
