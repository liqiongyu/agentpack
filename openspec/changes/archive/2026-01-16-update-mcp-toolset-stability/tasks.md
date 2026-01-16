## 1. Contract (M4-E1-T1 / #355)
- [x] Define the stabilized MCP tool list and input schemas
- [x] Run `openspec validate update-mcp-toolset-stability --strict --no-interactive`

## 2. Implementation
- [x] Update MCP server tool list to include the stabilized set
- [x] Add implementations for `preview`, `deploy`, `evolve_propose`, `evolve_restore`, and `explain`
- [x] Keep tool results as Agentpack JSON envelopes (structuredContent + text)

## 3. Tests
- [x] Update MCP stdio tests to assert the stabilized tool list
- [x] Run `cargo test --all --locked`

## 4. Archive
- [x] After shipping: `openspec archive update-mcp-toolset-stability --yes`
- [x] Run `openspec validate --all --strict --no-interactive`
