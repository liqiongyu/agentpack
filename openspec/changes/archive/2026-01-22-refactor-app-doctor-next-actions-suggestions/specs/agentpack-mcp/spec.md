## ADDED Requirements

### Requirement: doctor next_actions suggestions remain consistent across CLI and MCP
The system SHALL centralize the doctor `next_actions` suggestion logic so that the MCP `doctor` tool stays consistent with CLI `doctor --json` over time.

#### Scenario: doctor next_actions suggestions remain consistent
- **GIVEN** a doctor report that indicates `.gitignore` needs fixing
- **WHEN** an MCP client calls the `doctor` tool
- **AND** a user runs `agentpack doctor --json`
- **THEN** both responses include equivalent suggested `next_actions` (modulo surface-appropriate `--json` / `--yes` flags)
