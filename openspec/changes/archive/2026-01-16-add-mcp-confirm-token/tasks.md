## 1. Spec
- [x] Update `openspec/changes/add-mcp-confirm-token/specs/agentpack-mcp/spec.md`
- [x] Run `openspec validate add-mcp-confirm-token --strict --no-interactive`

## 2. Implementation
- [x] Add `confirm_token` issuance on `deploy` tool results (additive envelope field)
- [x] Require `confirm_token` for `deploy_apply` when `yes=true`
- [x] Bind token to a plan hash and enforce expiry
- [x] Emit stable error codes for missing/expired/mismatched token

## 3. Tests
- [x] Update `tests/mcp_server_stdio_mutating.rs` to cover the token flow
- [x] Add coverage for token mismatch and expiry (at least one)

## 4. Docs
- [x] Document new error codes in `docs/ERROR_CODES.md`
- [x] Update MCP section in `docs/SPEC.md` (confirm_token flow)

## 5. Verification
- [x] Run `cargo fmt --all -- --check`
- [x] Run `cargo clippy --all-targets --all-features -- -D warnings`
- [x] Run `cargo test --all --locked`
