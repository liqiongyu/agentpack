## 1. Implementation
- [x] Update lockfile generation so `resolved_source.local_path.path` is repo-relative (not absolute).
- [x] Preserve existing lockfile parsing behavior (existing lockfiles still load).

## 2. Tests
- [x] Add a test that verifies a local module’s `resolved_source.local_path.path` does not contain an absolute path.

## 3. Docs
- [x] Update `docs/SPEC.md` lockfile section to document repo-relative local paths.

## 4. Validation
- [x] Run `cargo fmt --all -- --check`.
- [x] Run `cargo clippy --all-targets --all-features -- -D warnings`.
- [x] Run `cargo test --all --locked`.
- [x] Run `openspec validate update-lockfile-local-path --strict --no-interactive`.
