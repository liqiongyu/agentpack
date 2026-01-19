## 1. Implementation

- [ ] Add `tests/journeys/common` with a `TestEnv` helper for journey tests.
- [ ] Add a smoke test that uses the harness to run `agentpack init` in a temp environment.

## 2. Spec deltas

- [ ] None (tests-only change).

## 3. Validation

- [ ] `openspec validate add-journey-harness --strict`
- [ ] `cargo test --all --locked`
