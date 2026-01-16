## 1. Spec
- [ ] Update `openspec/changes/add-mcp-confirm-token/specs/agentpack-mcp/spec.md`
- [ ] Run `openspec validate add-mcp-confirm-token --strict --no-interactive`

## 2. Implementation
- [ ] Add `confirm_token` issuance on `deploy` tool results (additive envelope field)
- [ ] Require `confirm_token` for `deploy_apply` when `yes=true`
- [ ] Bind token to a plan hash and enforce expiry
- [ ] Emit stable error codes for missing/expired/mismatched token

## 3. Tests
- [ ] Update `tests/mcp_server_stdio_mutating.rs` to cover the token flow
- [ ] Add coverage for token mismatch and expiry (at least one)

## 4. Docs
- [ ] Document new error codes in `docs/ERROR_CODES.md`
- [ ] Update MCP section in `docs/SPEC.md` (confirm_token flow)

## 5. Verification
- [ ] Run `cargo fmt --all -- --check`
- [ ] Run `cargo clippy --all-targets --all-features -- -D warnings`
- [ ] Run `cargo test --all --locked`
