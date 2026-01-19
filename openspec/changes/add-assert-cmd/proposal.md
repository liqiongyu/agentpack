# Change: Add `assert_cmd` + `predicates` for journey tests

## Why

Upcoming journey/E2E tests should use a consistent, readable CLI assertion style with good failure output.

## What Changes

- Add `assert_cmd` and `predicates` as dev-dependencies.

## Impact

- Affected specs: `agentpack-cli`
- Affected code: tests only (dependency surface)
- Affected runtime behavior: none
