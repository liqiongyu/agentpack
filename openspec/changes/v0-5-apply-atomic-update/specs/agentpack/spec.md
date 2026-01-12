# agentpack (delta)

## MODIFIED Requirements

### Requirement: apply uses atomic writes for updates
The system SHALL avoid unnecessary pre-deletes when writing updated outputs. On platforms where atomic replacement is supported by the underlying filesystem APIs, an update SHALL replace the destination atomically.

#### Scenario: update does not require deleting the destination first
- **GIVEN** a deployed managed file exists
- **WHEN** a subsequent deploy updates that file
- **THEN** the system uses atomic replacement semantics where available
