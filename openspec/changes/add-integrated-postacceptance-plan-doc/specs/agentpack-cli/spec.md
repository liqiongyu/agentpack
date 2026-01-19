## ADDED Requirements

### Requirement: Planning doc snapshots are tracked with metadata

The repository SHALL track major planning snapshots in git with a YAML frontmatter header including `status`, `owner`, `last_updated`, `superseded_by`, and `scope`.

#### Scenario: A new planning snapshot is added
- **WHEN** a new planning snapshot is checked into `docs/`
- **THEN** it includes the required YAML frontmatter fields
