# Change: Add 5-minute demo script + tutorial

## Why
Agentpack is easiest to understand when users can run a safe, non-destructive command and immediately see a plan/diff. A “5-minute demo” script that uses temporary HOME/AGENTPACK_HOME reduces onboarding friction and makes the value proposition tangible.

## What Changes
- Add `scripts/demo_5min.sh` that runs against a copied example repo in a temp workspace, with temp `HOME` and `AGENTPACK_HOME` (no writes to the user’s real environment).
- Add a tutorial page (EN + zh-CN) that explains the demo, the safety properties, and the next steps (how to apply with explicit `--yes`).
- Add an integration test that executes the script and asserts success.

## Impact
- Affected specs: `agentpack-cli` (docs/onboarding surface)
- Affected code/docs: new script + docs + tests
- Compatibility: no CLI/JSON behavior changes
