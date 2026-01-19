## 1. Implementation

- [x] 1.1 Convert `src/overlay.rs` into `src/overlay/mod.rs`.
- [x] 1.2 Split overlay code into `layout/dir/patch/rebase` submodules.
- [x] 1.3 Preserve public API surface (re-exports; no call-site changes expected).

## 2. Spec deltas

- [x] 2.1 Add a delta requirement describing the overlay modularization (archive with `--skip-specs` since this is refactor-only).

## 3. Validation

- [x] 3.1 `openspec validate refactor-overlay-submodules --strict --no-interactive`
- [x] 3.2 `cargo fmt --all -- --check`
- [x] 3.3 `cargo clippy --all-targets --all-features -- -D warnings`
- [x] 3.4 `cargo test --all --locked`
