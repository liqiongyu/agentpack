## 1. Specs
- [x] Add requirements for module_fs_key (cross-platform safe) and legacy fallback behavior.

## 2. Implementation
- [x] Add `module_fs_key` helper and use it for overlays and store paths.
- [x] Keep legacy path fallback for existing overlays/checkouts.
- [x] Update docs/SPEC.md overlay path descriptions.

## 3. Tests
- [x] Add tests asserting overlay paths are Windows-safe for ids like `instructions:base`.
- [x] Add tests for store path stability / no collisions (module_fs_key includes hash).

## 4. Validation
- [x] Run `cargo fmt`, `cargo clippy`, `cargo test`.
- [x] Run `openspec validate v0-5-module-fs-key --strict --no-interactive`.
