## ADDED Requirements

### Requirement: Patch overlays docs stay consistent with the CLI

The repository SHALL include an automated check that ensures patch overlays are documented, including the `overlay edit --kind patch` flag and stable error codes used for patch failures and rebase conflicts.

#### Scenario: CI catches missing patch overlay docs
- **WHEN** CI runs the doc-sync check
- **THEN** it passes when required patch overlay documentation is present
- **AND** it fails with an actionable message when documentation drifts
