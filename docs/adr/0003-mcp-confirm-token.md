# ADR 0003: MCP confirm_token for mutating operations

- Status: accepted
- Date: 2026-01-19

## Context

Agentpack supports automation surfaces beyond the CLI, including MCP tools. Mutating operations (writing files, deploying, adopting, patching) must be:
- safe by default
- auditable
- explicitly confirmable

In interactive CLI usage, `--yes` can serve as explicit confirmation. In MCP, calls can be programmatic and repeated; a simple boolean flag is easy to misuse and hard to audit.

## Decision

For MCP, require a two-phase workflow for mutating actions:
1) **plan**: compute and return the proposed changes, and generate a `confirm_token` that binds to the plan.
2) **apply**: perform the mutation only if the caller provides the matching `confirm_token`.

The `confirm_token` is designed to:
- make the “what I reviewed” → “what I applied” relationship explicit
- reduce accidental repeated writes
- enable safer automation and better logs/audit trails

## Consequences

- Pros:
  - safer defaults for automation
  - clearer auditability of applied changes
  - enables “preview then apply” workflows consistently across CLI/MCP
- Cons:
  - slightly more complex client flow (plan + apply)
  - tokens must be handled carefully to avoid stale or mismatched applies

## References

- `docs/SPEC.md`
- `docs/dev/roadmap.md`
