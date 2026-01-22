## ADDED Requirements

### Requirement: MCP deploy planning helper is modularized
The system SHALL keep the MCP deploy planning helper (`deploy_plan_envelope_in_process`) implemented in a dedicated module file so `src/mcp/tools.rs` stays focused on routing and schemas while preserving MCP tool behavior and JSON envelopes.

#### Scenario: deploy and deploy_apply reuse the same deploy planning implementation
- **GIVEN** MCP tools `deploy` and `deploy_apply` that both require the same plan computation
- **WHEN** the implementation is refactored for maintainability
- **THEN** both tools still reuse the same deploy planning implementation
- **AND** the returned Agentpack JSON envelope for `command="deploy"` remains unchanged
