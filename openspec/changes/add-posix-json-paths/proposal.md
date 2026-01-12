# Change: Add POSIX-style path fields to `--json` output

## Why
`--json` output is treated as an API. Today, many path-like fields are serialized using OS-native separators (e.g. `\` on Windows), which forces agents/scripts to special-case path parsing across platforms.

## What Changes
- Keep existing (native) path fields unchanged for backwards compatibility.
- Add `*_posix` companion fields for key path-like values in JSON payloads (forward slashes), so automation can parse paths consistently across OS.
- Document the convention in `docs/JSON_API.md` and `docs/SPEC.md`.

## Impact
- Affected specs: `agentpack-cli` (`--json` contract for path fields)
- Affected code: CLI command JSON payloads + shared output structs (e.g. plan change items)
- Affected tests: add coverage for at least one representative JSON payload containing `*_posix` fields
