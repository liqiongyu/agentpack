# Change: Add doc-sync check for patch overlays documentation

## Why
Patch overlays are easy to regress in docs (missing `--kind patch`, missing error codes, or missing conflict artifact hints). A small automated check prevents “feature exists but docs drift” from recurring.

## What Changes
- Add an integration test that asserts the patch overlay feature is documented in:
  - `docs/CLI.md` (flag discoverability)
  - `docs/OVERLAYS.md` (behavior + failure/conflict handling)
- The test fails with actionable messages when key strings are missing.

## Impact
- Affected specs: `agentpack-cli`
- Affected code: `tests/`
- Compatibility: no CLI/JSON behavior changes
