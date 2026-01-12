# Change: Add `evolve restore` for missing drift

## Why
`agentpack evolve propose` intentionally focuses on creating reviewable overlay changes. When drift is `missing` (a desired output file is absent on disk), the most common operator intent is to restore the desired state (i.e., write the expected content back), not to update source.

Today, that requires running a full `deploy --apply`, which may also update other managed files. A targeted “restore missing only” helper reduces risk and makes recovery workflows more ergonomic for both humans and automation.

## What Changes
- Add a new subcommand: `agentpack evolve restore`.
- Behavior: create-only restore of missing desired outputs (no updates, no deletes).
- JSON guardrails: in `--json` mode, requires `--yes` (stable `E_CONFIRM_REQUIRED`).

## Impact
- Affected specs: `agentpack-cli`
- Affected code: `src/cli/args.rs`, `src/cli/commands/evolve.rs`
- Affected docs/tests: `docs/SPEC.md`, new CLI tests
