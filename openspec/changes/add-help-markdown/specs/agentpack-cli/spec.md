## ADDED Requirements

### Requirement: CLI reference can be generated from the CLI definition

The CLI SHALL provide a deterministic Markdown representation of its command/flag surface so that `docs/reference/cli.md` can be generated from source-of-truth CLI definitions.

#### Scenario: `agentpack help --markdown` generates the CLI reference
- **WHEN** a maintainer runs `agentpack help --markdown`
- **THEN** it outputs a stable Markdown CLI reference suitable for committing as `docs/reference/cli.md`

#### Scenario: CI detects drift in the CLI reference
- **WHEN** the CLI surface changes but `docs/reference/cli.md` is not regenerated
- **THEN** CI fails with an actionable message describing how to regenerate the file
