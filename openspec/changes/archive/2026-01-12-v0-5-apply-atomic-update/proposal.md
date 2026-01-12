# Change: Apply atomic update semantics (v0.5)

## Why
`apply` should use atomic replacement semantics where possible. The current implementation pre-deletes files before writing, creating an unnecessary window where outputs can be missing.

## What Changes
- Remove pre-delete in `apply` for create/update paths.
- Centralize best-effort replace semantics in `write_atomic` (including a Windows fallback when destination exists).

## Impact
- Affected specs: `agentpack`
- Affected code: `src/apply.rs`
