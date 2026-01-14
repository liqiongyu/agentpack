# 排障（Troubleshooting）

> Language: 简体中文 | [English](../TROUBLESHOOTING.md)

这份文档按“症状 → 原因 → 解决”组织，并尽量用稳定错误码来定位问题。

如果你使用 `--json`，请优先看：
- `errors[0].code`
- `errors[0].details`（如果有）

错误码全集见：`ERROR_CODES.md`。

## 1) E_CONFIRM_REQUIRED

症状：
- `--json` 模式下执行写入类命令失败
- 错误码：`E_CONFIRM_REQUIRED`

原因：
- agentpack 把 `--json` 输出当作机器 API，为了避免脚本/LLM“误写入”，对写入类命令必须显式 `--yes`。

解决：
- 确认你确实想执行写入，然后重试并加 `--yes`：
  - `agentpack --json deploy --apply --yes`
  - `agentpack --json update --yes`
  - `agentpack --json bootstrap --yes`

## 2) E_ADOPT_CONFIRM_REQUIRED

症状：
- `deploy --apply` 被拒绝覆盖某些文件
- 错误码：`E_ADOPT_CONFIRM_REQUIRED`

原因：
- 这些更新是 `adopt_update`：目标路径已有文件，但它不在 agentpack 的托管清单里（非托管）。默认不会无感覆盖。

解决：
1) 先预览并确认影响范围：
- `agentpack preview --diff`

2) 如果确认要接管覆盖：
- `agentpack deploy --apply --adopt`

提示：
- `errors[0].details.sample_paths` 通常会给出部分路径样例。

## 3) E_DESIRED_STATE_CONFLICT

症状：
- `plan/preview/deploy` 报“conflicting desired outputs...`
- 错误码：`E_DESIRED_STATE_CONFLICT`

原因：
- 多个模块对同一 `(target, path)` 产出了不同内容。agentpack 拒绝静默覆盖。

解决：
- 调整 manifest：让该路径只由一个模块负责；或让冲突路径内容一致。
- 如果是 overlays 引起的，优先检查 overlay 中是否有同名文件覆盖了其他模块输出。

## 4) E_CONFIG_MISSING / E_CONFIG_INVALID / E_CONFIG_UNSUPPORTED_VERSION

症状：
- 找不到或解析不了 `agentpack.yaml`

解决：
- `E_CONFIG_MISSING`：运行 `agentpack init` 或用 `--repo` 指定正确 repo
- `E_CONFIG_INVALID`：按 `details.error` 修复 YAML；注意必须存在 `profiles.default`
- `E_CONFIG_UNSUPPORTED_VERSION`：将 manifest `version` 调整为支持值（当前 1）或升级工具

## 5) E_LOCKFILE_MISSING / E_LOCKFILE_INVALID

症状：
- fetch 或需要 git 模块时提示缺少/损坏 lockfile

解决：
- 生成：`agentpack lock` 或 `agentpack update`
- 如果 lockfile 损坏：删除 `agentpack.lock.json` 后重新 `agentpack update`

## 6) E_TARGET_UNSUPPORTED

症状：
- `--target` 传了不支持值，或 manifest 里 targets 配了未知 target

解决：
- `--target` 只能是 `all|codex|claude_code|cursor|vscode`
- manifest targets 只能包含：`codex|claude_code|cursor|vscode`

## 7) Overlay 相关

### E_OVERLAY_NOT_FOUND
- 先 `agentpack overlay edit <module_id>` 创建 overlay

### E_OVERLAY_BASELINE_MISSING
- overlay 元数据缺失，通常是手工创建目录导致
- 解决：重新运行 `agentpack overlay edit <module_id>` 让它补齐 `.agentpack/baseline.json`

### E_OVERLAY_BASELINE_UNSUPPORTED
- baseline 无法定位 merge base（例如历史版本 baseline 信息不足）
- 解决：重新创建 overlay（新 baseline），或把 upstream 变成可追溯（git）再创建 overlay

### E_OVERLAY_REBASE_CONFLICT
- 发生 3-way merge 冲突
- 解决：打开冲突文件手工处理后再 rebase，或直接手动 commit overlay

## 8) evolve propose 失败：dirty working tree

症状：
- `evolve propose` 提示 working tree dirty / refusing to propose...

原因：
- propose 会创建分支并写入 overlay 文件，要求 config repo 干净，避免把不相关改动混进去。

解决：
- 先 `git status`，把未提交改动 commit 或 stash
- 再重试 `agentpack evolve propose ...`

## 9) Windows 路径问题

症状：
- module id 里有 `:`，创建 overlay 目录失败

说明：
- agentpack 使用 `module_fs_key`（sanitize + hash）作为目录名，已避免 Windows 的非法字符。
- 如果你从历史版本迁移而来，旧目录名可能不兼容。建议：
  - 用 `agentpack overlay path <module_id>` 找到当前生效目录
  - 只保留一个目录命名（避免 legacy + canonical 并存引发困惑）
