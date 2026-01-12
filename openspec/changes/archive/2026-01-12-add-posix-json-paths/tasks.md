## 1. Implementation
- [x] Add a shared helper to format paths as POSIX-style strings for JSON output (forward slashes).
- [x] For key `--json` payloads that include paths, add `*_posix` companion fields (keep existing fields unchanged).

## 2. Docs
- [x] Update `docs/JSON_API.md` with the `*_posix` convention.
- [x] Update `docs/SPEC.md` to document the path convention for `--json` outputs.

## 3. Tests
- [x] Add/extend tests to assert `*_posix` fields are present and use `/` separators.

## 4. Validation
- [x] Run `cargo fmt --all -- --check`.
- [x] Run `cargo clippy --all-targets --all-features -- -D warnings`.
- [x] Run `cargo test --all --locked`.
- [x] Run `openspec validate add-posix-json-paths --strict --no-interactive`.
