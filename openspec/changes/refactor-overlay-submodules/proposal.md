# Change: Refactor overlay into submodules

## Why

`src/overlay.rs` has grown into a large, mixed-responsibility module, making it harder to reason about overlays and increasing the risk of regressions when extending overlay behavior (dir/patch/rebase).

Splitting the overlay implementation into focused submodules improves maintainability without changing user-facing behavior.

## What Changes

- Convert `src/overlay.rs` into a module directory `src/overlay/`.
- Split implementation into submodules:
  - `layout` (metadata + baseline layout helpers)
  - `dir` (directory overlay helpers)
  - `patch` (patch overlay helpers)
  - `rebase` (rebase implementation + shared merge helpers)
- Preserve the existing public API surface via re-exports from `src/overlay/mod.rs`.

## Impact

- Affected specs: none (no user-facing behavior change)
- Affected code: `src/overlay.rs` and its call sites
- Affected runtime behavior: none expected
