# Change: Refactor targets into submodules

## Why

Target rendering implementations currently live inside `src/engine.rs`, which increases coupling between the core engine and target-specific behaviors. This makes `engine.rs` harder to navigate and makes adding a new target require touching core engine code.

Splitting target rendering into focused modules under `src/targets/` improves maintainability while preserving user-facing behavior and the stable CLI/JSON contracts.

## What Changes

- Convert `src/targets.rs` into a module directory `src/targets/`.
- Move per-target rendering implementations out of `src/engine.rs` into `src/targets/<target>.rs`.
- Move target-only helper functions out of `src/engine.rs` into `src/targets/util.rs` (or equivalent).
- Keep the existing `TargetRoot` APIs stable via re-exports from `src/targets/mod.rs`.
- Update `src/target_adapters.rs` to call into `crate::targets::*` rendering functions instead of `Engine::render_*`.

## Impact

- Affected specs: none (no user-facing behavior change)
- Affected code: `src/engine.rs`, `src/target_adapters.rs`, `src/targets.rs`
- Affected runtime behavior: none expected
