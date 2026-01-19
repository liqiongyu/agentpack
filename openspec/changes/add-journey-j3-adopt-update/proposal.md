# Change: Add journey test J3 (adopt-update flow)

## Why

When `agentpack deploy --apply` would overwrite an existing unmanaged file, the plan includes `adopt_update` and the CLI MUST refuse unless `--adopt` is provided (with a stable error code for `--json` automation).

We need an end-to-end journey test to protect:
- correct refusal behavior and stable `errors[0].code` (`E_ADOPT_CONFIRM_REQUIRED`), and
- that once a file is adopted (made managed), subsequent updates are treated as `managed_update` and do not require `--adopt` again.

## What Changes

- Add an integration test for Journey J3 using `tests/journeys/common::TestEnv`:
  - seed an unmanaged existing file at a known target path
  - run `agentpack deploy --apply --json --yes` (expect `E_ADOPT_CONFIRM_REQUIRED`)
  - rerun with `--adopt` (expect success)
  - mutate the source module to force a follow-up update, and verify it is `managed_update` (no `--adopt` required)

## Impact

- Affected specs: none (tests-only)
- Affected code: `tests/` only
- Affected runtime behavior: none
