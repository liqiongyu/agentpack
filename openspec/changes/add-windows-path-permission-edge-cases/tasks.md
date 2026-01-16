## 1. Contract (M1-E5-T2 / #320)
- [x] Define requirements for Windows path/permission conformance + stable error codes
- [x] Run `openspec validate add-windows-path-permission-edge-cases --strict --no-interactive`

## 2. Implementation
- [ ] Add stable JSON error codes for common filesystem write failures (permission denied / invalid path / path too long)
- [ ] Update `docs/ERROR_CODES.md` for the new codes (and keep doc sync test passing)
- [ ] Add Windows-focused conformance tests for invalid chars, long paths, read-only, and permission denied

## 3. Tests
- [ ] `just check` passes locally

## 4. Archive
- [ ] After shipping: `openspec archive add-windows-path-permission-edge-cases --yes`
- [ ] Run `openspec validate --all --strict --no-interactive`
