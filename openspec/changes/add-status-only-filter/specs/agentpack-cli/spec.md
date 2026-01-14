# agentpack-cli (delta)

## ADDED Requirements

### Requirement: status supports --only drift filters
The system SHALL support `agentpack status --only <missing|modified|extra>` to filter drift items by kind. The `--only` option SHALL accept repeated values and comma-separated lists.

When invoked with `--json` and `--only` is set, the system SHALL include an additive `data.summary_total` capturing the unfiltered totals.

#### Scenario: status --only missing filters drift and includes totals
- **GIVEN** drift includes at least one `missing` item and at least one non-`missing` item
- **WHEN** the user runs `agentpack status --only missing --json`
- **THEN** every `data.drift[]` item has `kind = "missing"`
- **AND** `data.summary.missing > 0`
- **AND** `data.summary.modified = 0`
- **AND** `data.summary.extra = 0`
- **AND** `data.summary_total.missing > 0`
- **AND** `data.summary_total.modified > 0` OR `data.summary_total.extra > 0`
