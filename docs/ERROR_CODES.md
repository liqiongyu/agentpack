# ERROR_CODES.md

> Current as of **v0.4** (2026-01-12). `docs/SPEC.md` remains the authoritative contract.

本文件定义 `--json` 模式下对外稳定的错误码注册表（`errors[0].code`）。

约定：
- `ok=false` 时进程退出码非 0。
- `errors[0].code` 用于自动化分支；`errors[0].message` 主要用于人类阅读（可能随版本优化）。
- `warnings` 不用于强逻辑分支（字符串不保证稳定）。

## 稳定错误码

### `E_CONFIRM_REQUIRED`
**含义**：在 `--json` 模式下，命令会产生写入（文件系统或 git），但缺少 `--yes`。
**典型场景**：`deploy --apply --json` / `init --json` / `sync --json` 等。
**是否可重试**：是。
**建议动作**：确认这是预期写入后，补上 `--yes`；或移除 `--json` 走人类交互流程。

### `E_ADOPT_CONFIRM_REQUIRED`
**含义**：`deploy --apply` 将覆盖一个“非托管但已存在”的文件（adopt_update），但未显式提供 `--adopt`。
**是否可重试**：是。
**建议动作**：
- 先 `plan/preview` 确认变更影响；
- 若确实要接管该文件（覆盖写入），重新执行并加 `--adopt`。

### `E_CONFIG_MISSING`
**含义**：缺少 `repo/agentpack.yaml`。
**是否可重试**：是。
**建议动作**：运行 `agentpack init` 创建 repo skeleton，或指定正确的 `--repo`。

### `E_CONFIG_INVALID`
**含义**：`agentpack.yaml` 语法或语义不合法。
**是否可重试**：取决于修复配置。
**建议动作**：根据 `details`/报错信息修复 YAML（例如缺少 default profile / module id 重复 / source 配置不合法）。

### `E_CONFIG_UNSUPPORTED_VERSION`
**含义**：`agentpack.yaml` 的 `version` 不被支持。
**是否可重试**：取决于修复配置或升级/降级工具版本。
**建议动作**：将 `version` 调整为支持的版本（当前为 `1`），或升级 agentpack。

### `E_LOCKFILE_MISSING`
**含义**：缺少 `repo/agentpack.lock.json`，但当前命令需要它（例如 `fetch`）。
**是否可重试**：是。
**建议动作**：运行 `agentpack update`（或 `agentpack lock`）生成 lockfile。

### `E_LOCKFILE_INVALID`
**含义**：`agentpack.lock.json` JSON 不合法或无法解析。
**是否可重试**：取决于修复/重建 lockfile。
**建议动作**：修复 JSON 或删除后重新 `agentpack update` 生成。

### `E_TARGET_UNSUPPORTED`
**含义**：`--target` 指定了不支持的 target。
**是否可重试**：是。
**建议动作**：改用允许值（`all`/`codex`/`claude_code`），或修复 manifest targets 配置。

### `E_DESIRED_STATE_CONFLICT`
**含义**：多个模块对同一 `(target,path)` 产出不同内容，Agentpack 拒绝静默覆盖。
**是否可重试**：取决于配置/overlay 调整。
**建议动作**：调整 modules/overlays 使同一路径只由一个模块产出，或让冲突路径内容一致。
