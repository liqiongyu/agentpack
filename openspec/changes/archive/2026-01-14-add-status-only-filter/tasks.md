## 1. Spec
- [x] Update `docs/SPEC.md` to describe `status --only` and `summary_total` (additive).
- [x] Update `docs/CLI.md` and `docs/zh-CN/CLI.md` to document the `--only` flag.
- [x] Update `docs/JSON_API.md` to document `summary_total` behavior.

## 2. Implementation
- [x] Add `status --only` flag parsing (repeatable and comma-separated).
- [x] Filter drift list and computed summary when `--only` is set.
- [x] Add `summary_total` to `status --json` only when filtering is used.
- [x] Update `schema --json` to document the additive `status.summary_total?` field.

## 3. Tests
- [x] Update `help --json` golden snapshot to include `status --only`.
- [x] Update `schema --json` golden snapshot to include `status.summary_total?`.
- [x] Add integration test covering `status --only` filtering + `summary_total`.

## 4. Validation
- [x] `openspec validate add-status-only-filter --strict --no-interactive`
- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy --all-targets --all-features -- -D warnings`
- [x] `cargo test --all --locked`
