## 1. Implementation
- [x] Add `--sparse` and `--materialize` flags to `agentpack overlay edit` (mutually exclusive).
- [x] Implement sparse overlay skeleton creation (baseline + module_id, no copy).
- [x] Implement materialize behavior (copy missing upstream files only; never overwrite overlay edits).

## 2. Tests
- [x] Add tests for sparse overlay skeleton (no copied files, metadata exists).
- [x] Add tests for materialize behavior (copies missing files, preserves existing edits).

## 3. Docs
- [x] Update `docs/SPEC.md` overlay section to document sparse/materialize.

## 4. Validation
- [x] Run `cargo fmt --all -- --check`.
- [x] Run `cargo clippy --all-targets --all-features -- -D warnings`.
- [x] Run `cargo test --all --locked`.
- [x] Run `openspec validate add-sparse-overlay-mode --strict --no-interactive`.
