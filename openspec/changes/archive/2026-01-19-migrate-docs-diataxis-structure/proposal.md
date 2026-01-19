# Change: Migrate docs to a Diátaxis directory structure

## Why
Agentpack docs currently mix tutorials, how-to guides, reference material, and design notes at the top level, which makes navigation harder and increases the chance of doc drift.

## What Changes
- Create a Diátaxis-style directory structure under `docs/` and `docs/zh-CN/`:
  - `tutorials/`, `howto/`, `reference/`, `explanation/`, plus `dev/` and `archive/`
- Migrate the canonical user docs referenced by `docs/index.md` into the new structure.
- Keep tombstone pages at the old top-level paths to avoid breaking external links.
- Update doc-sync tests to follow the new canonical paths.

## Impact
- Affected specs: `agentpack-cli` (documentation layout requirements)
- Affected docs: `docs/index.md`, `docs/zh-CN/index.md`, moved docs under `docs/**`
- Affected tests: doc-sync tests under `tests/`
- Compatibility: no CLI/JSON behavior changes
