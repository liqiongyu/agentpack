## ADDED Requirements

### Requirement: Journey tests use standard CLI assertion tooling

The repository SHALL use standard CLI testing utilities (e.g., `assert_cmd` and `predicates`) for journey/E2E tests to keep assertions consistent and failure output actionable.

#### Scenario: Journey tests can spawn the agentpack binary consistently
- **WHEN** a journey test needs to run `agentpack` with a temp `AGENTPACK_HOME`
- **THEN** it can use `assert_cmd` to execute the binary and `predicates` to assert on output
