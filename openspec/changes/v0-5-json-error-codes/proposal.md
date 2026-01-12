# Change: Structured JSON error codes for common failures (v0.5)

## Why
In `--json` mode, automation needs stable, machine-readable error codes beyond `E_CONFIRM_REQUIRED` and the generic fallback `E_UNEXPECTED`.

## What Changes
- Introduce additional stable error codes for common user-facing failures:
  - config missing/invalid/unsupported version
  - lockfile missing/invalid
  - unsupported `--target`
- Ensure these failures return the appropriate code in `--json` mode.

## Impact
- Affected specs: `agentpack`
- Affected code: config/lockfile loading, CLI error mapping
- Affected docs/tests: `docs/SPEC.md`, CLI tests
