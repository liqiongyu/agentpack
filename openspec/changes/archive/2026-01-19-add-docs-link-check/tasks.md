## 1. Implementation

- [x] Add a markdown link checker test (offline, deterministic) that scans relevant `docs/**/*.md` files and validates internal links resolve.
- [x] Exclude code fences from link parsing to avoid false positives.
- [x] Ensure failures are actionable (source file + broken target).

## 2. Spec deltas

- [x] Add a requirement for docs link checking to `openspec/changes/add-docs-link-check/specs/agentpack-cli/spec.md`.

## 3. Validation

- [x] `openspec validate add-docs-link-check --strict`
- [x] `cargo test --all --locked`
