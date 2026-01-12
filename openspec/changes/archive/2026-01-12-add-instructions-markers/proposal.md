# Change: Add instructions section markers (enable evolve propose for aggregated AGENTS.md)

## Why
`agentpack` aggregates multiple `instructions` modules into a single deployed `AGENTS.md` (Codex target). Today, `agentpack evolve propose` intentionally refuses to generate proposals for multi-module outputs, which blocks the most common real-world workflow: editing a combined instructions file and wanting to map that drift back to the correct `instructions:<id>` module for review.

## What Changes
- Codex instructions aggregation writes stable per-module section markers in the combined `AGENTS.md` output.
- `agentpack evolve propose` detects these markers and, when possible, maps drift in an aggregated `AGENTS.md` back to the specific `instructions` module section(s), producing proposeable candidates instead of `multi_module_output` skips.

## Compatibility
- This changes the deployed `AGENTS.md` content (comments are added). Existing deployments will appear drifted until re-deployed.
- Behavior is additive for automation: JSON schema does not change; only `evolve propose` becomes more capable.

## Impact
- Affected specs: `agentpack`
- Affected code: `src/engine.rs` (instructions aggregation), `src/cli/commands/evolve.rs` (proposal mapping)
- Affected docs/tests: `docs/SPEC.md`, new CLI test for marker-based proposals
