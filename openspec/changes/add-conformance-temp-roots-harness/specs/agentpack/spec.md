# agentpack (delta)

## ADDED Requirements

### Requirement: Conformance tests run in temporary roots without writing to real home

The repository SHALL ensure target conformance tests:
- run entirely within temporary directories,
- do not rely on real user home or machine state, and
- can execute safely in parallel.

#### Scenario: conformance tests do not write outside temp roots
- **WHEN** the conformance test suite is executed
- **THEN** it does not read or write outside test-managed temporary roots
