## 1. Implementation
- [x] Add `--git` flag to `agentpack init`.
- [x] Implement idempotent `git init` + minimal `.gitignore` update.

## 2. Tests
- [x] Update `help --json` golden output for the new flag.
- [x] Add/adjust regression tests if needed.

## 3. Validation
- [x] Run `cargo fmt`, `cargo clippy`, `cargo test`.
- [x] Run `openspec validate update-init-git-flag --strict --no-interactive`.
