# Change: Detect DesiredState path conflicts (v0.5)

## Why
Multiple modules can currently write the same `(target, path)` with different bytes. Because `DesiredState` is a map, later inserts overwrite earlier ones silently, which is unsafe and hard to debug (especially for automation).

## What Changes
- Detect conflicts when building `DesiredState`: if a `(target, path)` is already present with different content, fail fast.
- If the bytes are identical, merge module provenance (`module_ids`) instead of erroring.
- In `--json` mode, return a stable error code `E_DESIRED_STATE_CONFLICT`.

## Impact
- Affected specs: `agentpack`
- Affected code: engine/target rendering (desired state construction), CLI JSON errors
