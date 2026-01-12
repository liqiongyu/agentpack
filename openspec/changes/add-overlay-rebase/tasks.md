## 1. Spec
- [x] Add OpenSpec delta for `agentpack-cli` describing `overlay rebase` behavior and conflict reporting.
- [x] Run `openspec validate add-overlay-rebase --strict` and fix any issues.

## 2. Implementation
- [x] Extend overlay baseline metadata to include upstream identity (git commit/url/subdir or local repo revision) in a backwards-compatible way.
- [x] Implement overlay rebase core logic (3-way merge + baseline update) with a deterministic conflict signal.
- [x] Add `agentpack overlay rebase` CLI plumbing (+ `help --json` and mutating registry updates).
- [x] Add tests for: unmodified-file rebase, clean merge, and conflict reporting.

## 3. Docs
- [x] Update `docs/SPEC.md` overlays section to document `overlay rebase`.
- [x] Register new stable error codes in `docs/ERROR_CODES.md`.
- [x] Update `CHANGELOG.md` (Unreleased).

## 4. Validation
- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy --all-targets --all-features -- -D warnings`
- [x] `cargo test --all --locked`
