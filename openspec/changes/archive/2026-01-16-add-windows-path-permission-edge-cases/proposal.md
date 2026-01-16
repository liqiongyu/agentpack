# Change: add Windows path/permission conformance edge cases

## Why

Agentpack’s safety guarantees depend on predictable filesystem behavior across platforms. Windows has distinct path and permission constraints (invalid characters, path length limits, read-only semantics) that can produce confusing failures if left as unclassified `E_UNEXPECTED` errors.

We need conformance coverage for these boundaries and stable JSON error codes so automation can handle failures safely and humans get actionable guidance.

## What Changes

- Add conformance tests (Windows-focused) covering:
  - invalid path characters
  - overly long paths
  - read-only destination files
  - permission denied writes
- Classify common filesystem write failures into stable `--json` error codes with actionable details (instead of `E_UNEXPECTED`).
- Update the error code registry (`docs/ERROR_CODES.md`) accordingly.

## Non-Goals

- Do not change the desired-state semantics or target mappings.
- Do not introduce new targets.

## Impact

- Affected specs: `openspec/specs/agentpack-cli/spec.md`, `openspec/specs/agentpack/spec.md`
- Affected docs: `docs/ERROR_CODES.md` (new stable codes)
- Affected code: filesystem/apply error classification for `deploy`/`rollback`/other writers
- Affected tests: new conformance cases (Windows-gated where appropriate)
