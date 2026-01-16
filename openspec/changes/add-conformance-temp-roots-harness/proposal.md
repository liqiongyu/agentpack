# Change: add temp-roots conformance harness (no real home writes)

## Why

Target conformance tests are the safety net that lets Agentpack track target ecosystem changes without regressing core safety semantics.

They must be safe to run on contributor machines and in CI:
- no reliance on real user home or machine state
- no accidental writes outside the test temp directory
- parallel-safe execution

## What Changes

- Add/standardize a conformance test harness that:
  - runs each conformance test entirely under temp directories
  - sets environment isolation (`AGENTPACK_HOME`, `HOME`, deterministic `AGENTPACK_MACHINE_ID`, etc.)
  - uses deterministic workspace setup (no network)
- Refactor existing target conformance smoke tests to use the harness.

## Non-Goals

- Do not change the conformance semantics being tested (delete safety, drift classification, rollback, etc.).
- Do not change CLI behavior or JSON schema.

## Impact

- Affected specs: `openspec/specs/agentpack/spec.md`
- Affected tests: `tests/conformance_targets.rs` (and any shared harness modules)
