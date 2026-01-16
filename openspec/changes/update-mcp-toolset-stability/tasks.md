## 1. Contract (M4-E1-T1 / #355)
- [ ] Define the stabilized MCP tool list and input schemas
- [ ] Run `openspec validate update-mcp-toolset-stability --strict --no-interactive`

## 2. Implementation
- [ ] Update MCP server tool list to include the stabilized set
- [ ] Add implementations for `preview`, `deploy`, `evolve_propose`, `evolve_restore`, and `explain`
- [ ] Keep tool results as Agentpack JSON envelopes (structuredContent + text)

## 3. Tests
- [ ] Update MCP stdio tests to assert the stabilized tool list
- [ ] Run `cargo test --all --locked`

## 4. Archive
- [ ] After shipping: `openspec archive update-mcp-toolset-stability --yes`
- [ ] Run `openspec validate --all --strict --no-interactive`
