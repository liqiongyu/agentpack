## 1. Contract (M1-E1-T3 / #308)
- [x] Define dry-run conflict reporting requirements (destination path conflicts + module id collisions)
- [x] Confirm stable error code for apply conflicts (`E_IMPORT_CONFLICT`), and document details fields
- [x] Run `openspec validate update-import-conflict-reporting --strict --no-interactive`

## 2. Implementation
- [x] Detect destination path conflicts during plan building (no writes)
- [x] Add additive JSON fields to report conflicts in dry-run output
- [x] Keep apply behavior safe-by-default (fail on conflicts; stable error code + details)
- [x] Update human output to summarize conflicts + suggested next actions

## 3. Tests
- [x] Add/extend integration tests for dry-run conflict reporting and error codes

## 4. Docs
- [x] Update `docs/JSON_API.md` with additive fields for import conflict reporting
- [x] Update `docs/CLI.md` with conflict notes / examples
- [x] Update `docs/ERROR_CODES.md` with conflict details fields

## 5. Archive
- [x] After shipping: `openspec archive update-import-conflict-reporting --yes`
- [x] Run `openspec validate --all --strict --no-interactive`
