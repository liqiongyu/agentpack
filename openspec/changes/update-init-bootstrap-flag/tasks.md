## 1. Implementation
- [x] Add `--bootstrap` flag to `agentpack init`.
- [x] Install operator assets after init when `--bootstrap` is set.

## 2. Tests
- [x] Update `help --json` golden output for the new flag.
- [x] Add regression test that verifies operator assets are installed into a temp HOME.

## 3. Validation
- [x] Run `cargo fmt`, `cargo clippy`, `cargo test`.
- [x] Run `openspec validate update-init-bootstrap-flag --strict --no-interactive`.
