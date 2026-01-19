## 1. Implementation

- [x] Add `assert_cmd` and `predicates` under `[dev-dependencies]`.
- [x] Ensure `Cargo.lock` is updated and `cargo test --all --locked` passes.

## 2. Spec deltas

- [x] Add a requirement noting journey tests use `assert_cmd`/`predicates`.

## 3. Validation

- [x] `openspec validate add-assert-cmd --strict`
- [x] `cargo test --all --locked`
