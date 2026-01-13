# ERROR_CODES（稳定错误码注册表）

> Current as of **v0.5.0** (2026-01-13). `SPEC.md` 是语义级权威；本文件是 `--json` 自动化分支用的“注册表”。

本文件定义 `--json` 模式下对外稳定的错误码（`errors[0].code`）。

约定：
- `ok=false` 时进程退出码非 0。
- `errors[0].code` 用于自动化分支；`errors[0].message` 主要用于人类阅读（可能随版本优化）。
- `warnings` 不用于强逻辑分支（字符串不保证稳定）。

## 稳定错误码

### E_CONFIRM_REQUIRED
含义：在 `--json` 模式下，命令会产生写入（文件系统或 git），但缺少 `--yes`。
典型场景：`deploy --apply --json`、`update --json`、`overlay edit --json` 等。
是否可重试：是。
建议动作：确认确实要写入后，补上 `--yes` 重试；或去掉 `--json` 走人类确认流程。
details：通常包含 `{"command": "..."}`。

### E_ADOPT_CONFIRM_REQUIRED
含义：`deploy --apply` 将覆盖一个“非托管但已存在”的文件（adopt_update），但未显式提供 `--adopt`。
是否可重试：是。
建议动作：
- 先 `preview --diff` 确认影响范围；
- 若确实要接管并覆盖，重新执行并加 `--adopt`。
details：包含 `{flag, adopt_updates, sample_paths}`。

### E_CONFIG_MISSING
含义：缺少 `repo/agentpack.yaml`。
是否可重试：是。
建议动作：运行 `agentpack init` 创建 skeleton，或指定正确 `--repo`。
details：通常包含 `{path, hint}`。

### E_CONFIG_INVALID
含义：`agentpack.yaml` 语法或语义不合法。
是否可重试：取决于修复配置。
建议动作：根据 `details`/报错信息修复 YAML（例如缺少 default profile、module id 重复、source 不合法、target 未配置等）。

### E_CONFIG_UNSUPPORTED_VERSION
含义：`agentpack.yaml` 的 `version` 不被支持。
是否可重试：取决于修复配置或升级工具版本。
建议动作：将 `version` 调整为支持值（当前为 `1`），或升级 agentpack。
details：通常包含 `{version, supported}`。

### E_LOCKFILE_MISSING
含义：缺少 `repo/agentpack.lock.json`（但当前命令需要它，例如 `fetch`）。
是否可重试：是。
建议动作：运行 `agentpack lock` 或 `agentpack update`。

### E_LOCKFILE_INVALID
含义：`agentpack.lock.json` JSON 不合法或无法解析。
是否可重试：取决于修复/重建 lockfile。
建议动作：修复 JSON 或删除后重新 `agentpack update` 生成。

### E_TARGET_UNSUPPORTED
含义：
- `--target` 指定了不支持的值；或
- manifest 里配置了未知 target。
是否可重试：是。
建议动作：
- `--target` 只能是 `all|codex|claude_code`
- manifest targets 只能包含内置 target（目前 `codex` 与 `claude_code`）

### E_DESIRED_STATE_CONFLICT
含义：多个模块对同一 `(target,path)` 产出不同内容，Agentpack 拒绝静默覆盖。
是否可重试：取决于配置/overlay 调整。
建议动作：调整 modules/overlays，使同一路径只由一个模块产出，或让冲突路径内容一致。
details：包含冲突双方的 sha256 与 module_ids。

### E_OVERLAY_NOT_FOUND
含义：请求的 overlay 目录不存在。
是否可重试：是。
建议动作：先执行 `agentpack overlay edit <module_id>` 创建 overlay。

### E_OVERLAY_BASELINE_MISSING
含义：overlay 元数据缺失（`<overlay_dir>/.agentpack/baseline.json` 不存在），无法 rebase。
是否可重试：是。
建议动作：重新运行 `agentpack overlay edit <module_id>` 以补齐元数据。

### E_OVERLAY_BASELINE_UNSUPPORTED
含义：overlay baseline 缺少可定位的 merge base，无法安全 rebase。
是否可重试：取决于修复 baseline。
建议动作：通常建议重新创建 overlay（生成新 baseline），或确保 upstream 可追溯（git）后再重建。

### E_OVERLAY_REBASE_CONFLICT
含义：`overlay rebase` 检测到无法自动合并的冲突。
是否可重试：是（解决冲突后重试）。
建议动作：打开 overlay 目录中带冲突标记的文件，手动解决后再次运行 `agentpack overlay rebase` 或直接提交 overlay 变更。
details：包含 `{conflicts, summary, overlay_dir, scope, ...}`。

## 非稳定/兜底错误码

### E_UNEXPECTED
含义：未被分类为稳定 UserError 的异常失败。
是否可重试：不确定。
建议动作：
- 把 `errors[0].message` 与上下文输出（stdout/stderr）保存下来；
- 尝试用更小的复现步骤重试；
- 如果你在做自动化：通常应当“转人工”或 fail-fast，而不是基于 message 文本做分支。
