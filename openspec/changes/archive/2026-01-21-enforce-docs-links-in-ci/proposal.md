# Change: Enforce markdown link checks in CI

## Why
Broken internal links in docs/README directly hurt onboarding. Today, docs-only changes may skip Rust checks in CI, so link regressions can slip through.

## What Changes
- Expand `tests/docs_markdown_links.rs` to validate links in `docs/` **and** top-level `README.md` / `README.zh-CN.md`.
- Add a CI job to run the link-check test so broken links fail PR checks even for docs-only changes.

## Impact
- Affected specs: `agentpack-cli` (docs-as-code / onboarding surface)
- Affected code/docs: tests + CI workflow
- Compatibility: no CLI/JSON behavior changes
