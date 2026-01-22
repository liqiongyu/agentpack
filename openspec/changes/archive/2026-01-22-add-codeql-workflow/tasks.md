## 1. Spec & planning
- [x] Add OpenSpec delta requirements for the new CodeQL workflow
- [x] Run `openspec validate add-codeql-workflow --strict --no-interactive`

## 2. Implementation
- [x] Add `.github/workflows/codeql.yml` (CodeQL for `rust` + `actions`)
- [x] Update `docs/SECURITY_CHECKS.md` to document CodeQL scanning
- [x] Update `docs/GITHUB_SETUP.md` guidance for code scanning setup

## 3. Verification
- [x] `cargo fmt --all`
- [x] `cargo test --locked --test docs_markdown_links`
- [x] `just check`
