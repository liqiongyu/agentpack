## 1. Implementation

- [x] 1.1 Convert `src/targets.rs` into `src/targets/mod.rs`.
- [x] 1.2 Move target rendering from `src/engine.rs` into `src/targets/*` modules.
- [x] 1.3 Move target-only helpers into `src/targets/util.rs` (or equivalent).
- [x] 1.4 Update `src/target_adapters.rs` to call target modules.
- [x] 1.5 Preserve behavior via existing conformance + journey tests.

## 2. Spec deltas

- [x] 2.1 Add a delta requirement describing targets modularization.

## 3. Validation

- [x] 3.1 `openspec validate refactor-targets-submodules --strict --no-interactive`
- [x] 3.2 `cargo fmt --all -- --check`
- [x] 3.3 `cargo clippy --all-targets --all-features -- -D warnings`
- [x] 3.4 `cargo test --all --locked`
