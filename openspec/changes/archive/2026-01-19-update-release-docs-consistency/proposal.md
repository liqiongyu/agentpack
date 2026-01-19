# Change: Update release docs consistency

## Why
Users frequently copy installation commands from `README.md` and `docs/QUICKSTART.md`. If the referenced tag/version drifts from the crate version, installation fails or installs an unexpected version.

## What Changes
- Align `README.md` install-from-source example tag with the current crate version.
- Add an automated check to keep `README.md` and `docs/QUICKSTART.md` install tags consistent with the crate version.

## Impact
- Affected specs: `agentpack-cli`
- Affected code/docs: `README.md`, `docs/QUICKSTART.md`, `tests/`
- Compatibility: no CLI/JSON behavior changes
