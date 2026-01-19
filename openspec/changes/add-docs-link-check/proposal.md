# Change: Add docs markdown link checking in CI

## Why

As documentation is reorganized and extended, it’s easy for internal links to drift or break (especially with tombstones, renames, and language toggles). Broken links reduce discoverability and user trust.

## What Changes

- Add a deterministic, offline markdown link checker that validates internal/relative links in `docs/` resolve to existing files.
- Run it in CI (via `cargo test`) so broken links are caught before merge.

## Impact

- Affected specs: `agentpack-cli`
- Affected code: tests only (no runtime behavior)
- Affected docs: none (except fixing any newly-detected broken links)
