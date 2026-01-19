# ADR 0001: JSON contract stability (`--json` schema_version=1)

- Status: accepted
- Date: 2026-01-19

## Context

Agentpack is used both interactively and in automation (CI, scripts, MCP clients). For automation, `--json` output is part of the public contract: consumers parse it, persist it, and depend on stable error codes and stable meanings.

Breaking the JSON schema (renaming fields, changing semantics, removing fields) forces downstream consumers to update in lockstep and makes upgrades risky.

## Decision

- Treat `--json` as a stable API contract with `schema_version=1`.
- Changes to the JSON envelope SHALL be additive-only:
  - adding new top-level fields is allowed
  - adding new error codes is allowed (with docs + tests)
  - removing/renaming fields or changing semantics is not allowed within `schema_version=1`
- Stable error codes remain stable; new error codes require documentation and test coverage.

## Consequences

- Pros:
  - automation stays reliable across upgrades
  - CI and tooling can safely pin to `schema_version=1`
  - regressions are caught early when docs + golden tests are updated together
- Cons:
  - we must be disciplined about introducing new fields rather than “fixing” existing ones
  - occasional redundancy or deprecated fields may accumulate until a future `schema_version=2`

## References

- `docs/SPEC.md`
- `docs/reference/json-api.md`
- `docs/reference/error-codes.md`
