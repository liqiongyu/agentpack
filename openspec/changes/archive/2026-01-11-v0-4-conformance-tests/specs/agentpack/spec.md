# agentpack (delta)

## ADDED Requirements

### Requirement: Target conformance tests cover critical safety semantics
The repository SHALL include conformance tests that validate critical cross-target safety semantics, including:
- delete protection (only manifest-managed paths can be deleted)
- per-root manifest write/read
- drift classification (missing/modified/extra)
- rollback restoring previous outputs

#### Scenario: conformance tests exist for built-in targets
- **GIVEN** built-in targets `codex` and `claude_code`
- **WHEN** the test suite is run
- **THEN** conformance tests execute these semantics for both targets
