# Change: Refactor CLI into modules

## Why
`src/cli.rs` has grown large, which increases the cost of adding commands and keeping the human/JSON output contracts consistent.

## What Changes
- Split the CLI implementation into a `src/cli/` module tree:
  - `cli/mod.rs`: entrypoint and shared wiring
  - `cli/commands/*.rs`: subcommand handlers
  - `cli/json.rs`: JSON envelope + error mapping helpers
  - `cli/util.rs`: shared helpers (confirm, printing, path helpers)
- Preserve CLI behavior and the `--json` contract (envelope, codes, and field meanings).

## Impact
- Affected specs: none (no behavior change intended)
- Affected code: `src/cli.rs` → `src/cli/` modules
- Affected tests: existing CLI tests/goldens should continue to pass
