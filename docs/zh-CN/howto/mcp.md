# MCP（Codex + VS Code 集成）

> Language: 简体中文 | [English](../../howto/mcp.md)

Agentpack 提供基于 stdio 的 MCP 服务器：

```bash
agentpack mcp serve
```

这让支持 MCP 的客户端（包括 Codex）可以通过结构化工具调用 Agentpack，而不是依赖 shell 文本解析。

## 暴露了哪些工具？

工具集合：
- 只读：`plan`、`diff`、`preview`、`status`、`doctor`、`deploy`、`explain`
- 写入（需显式批准）：`deploy_apply`、`rollback`、`evolve_propose`、`evolve_restore`

工具结果复用 Agentpack 稳定的 `--json` envelope 作为权威 payload（同时会以序列化 JSON 文本返回）。

写入类工具要求显式批准：
- `deploy_apply`、`rollback`、`evolve_propose`、`evolve_restore` 需要 `yes=true`，否则返回 `E_CONFIRM_REQUIRED`。

两阶段部署确认：
- 先调用 `deploy` 获取 `data.confirm_token`（以及 `data.confirm_plan_hash`、`data.confirm_token_expires_at` 等）。
- 再调用 `deploy_apply`，并携带 `yes=true` 与 `confirm_token`。
- 若 token 缺失/过期/不匹配，`deploy_apply` 返回 `E_CONFIRM_TOKEN_REQUIRED` / `E_CONFIRM_TOKEN_EXPIRED` / `E_CONFIRM_TOKEN_MISMATCH`。

## Codex 配置

Codex MCP 服务器在 `~/.codex/config.toml` 的 `[mcp_servers.<name>]` 下配置。

添加一个 `agentpack` 服务器配置：

```toml
[mcp_servers.agentpack]
command = "agentpack"
args = ["mcp", "serve"]

# 强烈建议：设置工作目录为项目根目录。
# Agentpack 依赖 CWD 来识别 project overlays。
cwd = "/path/to/your/project"

# 可选：如果使用非默认 Agentpack home。
# env = { AGENTPACK_HOME = "/path/to/.agentpack" }

# 可选：限制 Codex 可调用的工具。
# enabled_tools = [
#   "plan", "diff", "preview", "status", "doctor", "deploy", "explain",
#   "deploy_apply", "rollback", "evolve_propose", "evolve_restore"
# ]

enabled = true
```

注意：
- `agentpack mcp serve` 不支持 `--json`（stdout 被 MCP 协议占用）。
- 如果 `agentpack` 不在 Codex PATH 中，请将 `command` 设为 agentpack 的绝对路径。

Codex 侧 MCP 配置细节：
- https://developers.openai.com/codex/mcp/

## VS Code 配置

VS Code MCP 服务器可全局配置（用户 `mcp.json`）或按工作区配置（`.vscode/mcp.json`）。

下面是工作区级 `.vscode/mcp.json` 示例：

```json
{
  "servers": {
    "agentpack": {
      "type": "stdio",
      "command": "agentpack",
      "args": ["mcp", "serve"],
      "env": {
        "AGENTPACK_HOME": "${env:AGENTPACK_HOME}"
      }
    }
  }
}
```

安全提示：
- 不要在 `mcp.json` 中硬编码敏感信息。VS Code 支持 `inputs`（交互式密钥）和 `envFile`。
- 参考：https://code.visualstudio.com/docs/copilot/customization/mcp-servers

## 常见问题

### 工作目录不对（project overlays 未生效）

Agentpack 通过 MCP 服务器进程的工作目录判定 project context（也就是 project overlays 是否生效）。

修复方式：
- 在 `~/.codex/config.toml` 中设置 `cwd` 为项目根目录，或
- 从项目根目录启动 Codex，且不要覆盖 `cwd`。
- 在 VS Code 中更推荐使用项目根目录下的 `.vscode/mcp.json`（并打开该目录/工作区）。

### Agentpack home / config repo 错误

Agentpack 通过 `AGENTPACK_HOME` 找到 config repo 与 snapshots：
- config repo 默认：`$AGENTPACK_HOME/repo`
- snapshots：`$AGENTPACK_HOME/state/snapshots`

修复方式：
- 在 MCP 配置中设置 `env = { AGENTPACK_HOME = "..." }`，或
- 对支持的工具传入 `repo` 参数。

### 未显式批准导致写入被拒绝

写入工具需要 `yes=true`：
- `deploy_apply` → 没有 `yes=true` 会返回 `E_CONFIRM_REQUIRED`（并且还需要匹配的 `confirm_token`）
- `rollback` → 没有 `yes=true` 会返回 `E_CONFIRM_REQUIRED`

即使批准了：
- 若会覆盖未被管理的文件，`deploy_apply` 仍会返回 `E_ADOPT_CONFIRM_REQUIRED`，除非你显式传 `adopt=true`。

### Debug（stdio 传输）

如果客户端提示 MCP 服务器“无响应”，可先验证服务能否正常启动：

```bash
agentpack mcp serve
```

该进程 stdout 只应输出 MCP 协议消息，不应输出其他内容。
