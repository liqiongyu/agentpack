# Change: evolve propose coverage (missing drift + aggregated outputs) (v0.5)

## Why
`evolve propose` is intentionally conservative, but previously it could look like "no drift" even when drift existed that it couldn't safely propose (e.g., missing files or multi-module aggregated outputs like combined instructions).

## What Changes
- `evolve propose` detects drift even for non-proposeable cases and reports them as `skipped` items with reasons.
- `--json` output includes a `summary` and `skipped` list; human output prints skipped items.
- Documentation clarifies the conservative behavior and recommends `--dry-run` first.

## Impact
- Affected specs: `agentpack`
- Affected code: `src/cli.rs` (`evolve_propose`)
- Affected docs/templates/tests: `docs/SPEC.md`, `templates/claude/commands/ap-evolve.md`, new CLI tests
