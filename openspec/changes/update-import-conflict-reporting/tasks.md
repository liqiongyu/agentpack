## 1. Contract (M1-E1-T3 / #308)
- [ ] Define dry-run conflict reporting requirements (destination path conflicts + module id collisions)
- [ ] Confirm stable error code for apply conflicts (`E_IMPORT_CONFLICT`), and document details fields
- [ ] Run `openspec validate update-import-conflict-reporting --strict --no-interactive`

## 2. Implementation
- [ ] Detect destination path conflicts during plan building (no writes)
- [ ] Add additive JSON fields to report conflicts in dry-run output
- [ ] Keep apply behavior safe-by-default (fail on conflicts; stable error code + details)
- [ ] Update human output to summarize conflicts + suggested next actions

## 3. Tests
- [ ] Add/extend integration tests for dry-run conflict reporting and error codes

## 4. Docs
- [ ] Update `docs/JSON_API.md` with additive fields for import conflict reporting
- [ ] Update `docs/CLI.md` with conflict notes / examples

## 5. Archive
- [ ] After shipping: `openspec archive update-import-conflict-reporting --yes`
- [ ] Run `openspec validate --all --strict --no-interactive`
