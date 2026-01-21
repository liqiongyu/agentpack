# Change: Add reproducible CLI demo assets (VHS)

## Why
A short, reproducible CLI demo makes Agentpack’s workflow and safety story obvious at a glance. Recording via a scripted tool (VHS) avoids manual screen recording drift and makes it easy to refresh the demo as the CLI evolves.

## What Changes
- Add `docs/assets/demo.tape` and `docs/assets/demo.gif` (generated via VHS).
- Add `docs/assets/README.md` explaining how to regenerate the demo.
- Add a `just demo-gif` recipe.
- Add a lightweight test that asserts demo assets exist and are referenced.

## Impact
- Affected specs: `agentpack-cli` (docs/marketing surface)
- Affected code/docs: docs assets, justfile, tests
- Compatibility: docs/tooling only; no CLI/JSON behavior changes
