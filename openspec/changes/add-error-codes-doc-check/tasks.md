## 1. Contract (M1-E4-T2 / #318)
- [x] Define the ERROR_CODES registry consistency requirement
- [x] Run `openspec validate add-error-codes-doc-check --strict --no-interactive`

## 2. Implementation
- [ ] Add a CI check to ensure `docs/ERROR_CODES.md` matches emitted `errors[0].code` values

## 3. Tests
- [ ] Ensure the check runs in `cargo test --all --locked` and has a clear failure message

## 4. Archive
- [ ] After shipping: `openspec archive add-error-codes-doc-check --yes`
- [ ] Run `openspec validate --all --strict --no-interactive`
