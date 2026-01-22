## ADDED Requirements

### Requirement: doctor next_actions suggestions remain consistent across CLI and MCP
The system SHALL centralize the doctor `next_actions` suggestion logic so that CLI `doctor --json` and the MCP `doctor` tool keep equivalent suggestion behavior over time.

#### Scenario: doctor next_actions suggestions remain consistent
- **GIVEN** a doctor report that includes a root suggestion like `create directory: mkdir -p ...`
- **WHEN** the user runs `agentpack doctor --json`
- **AND** an MCP client calls the `doctor` tool
- **THEN** both responses include equivalent suggested `next_actions` (modulo surface-appropriate `--json` / `--yes` flags)
