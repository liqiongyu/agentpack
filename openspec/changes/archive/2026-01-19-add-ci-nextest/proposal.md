# Change: Add optional nextest in CI

## Why

`cargo nextest` is a faster and often more stable test runner than the built-in `cargo test` harness. Providing an opt-in/gradual `nextest` job in CI helps us improve feedback time and reduce flaky test pain without changing runtime behavior.

## What Changes

- Add a `just nextest` task for local/CI use.
- Add an **optional** GitHub Actions job that runs `cargo nextest run` on `ubuntu-latest` for Rust changes.

## Impact

- Affected specs: none (tooling-only)
- Affected code: `.github/workflows/ci.yml`, `justfile`
- Affected runtime behavior: none
