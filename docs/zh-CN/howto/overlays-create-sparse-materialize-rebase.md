# Overlays：sparse → materialize → rebase

> Language: 简体中文 | [English](../../howto/overlays-create-sparse-materialize-rebase.md)

本 how-to 描述最常见的 overlays 工作流：

1) 创建 **sparse** overlay（只保留最小改动）
2) 需要浏览 upstream 时再 **materialize**（按需拷贝，不覆盖你的改动）
3) upstream 更新后执行 **rebase**，并尽量保持 overlay 稀疏

概念模型与磁盘布局见：`docs/zh-CN/explanation/overlays.md`（或对应英文版 `docs/explanation/overlays.md`）。

## 0) 前置条件

- 你已经有要定制的 module id（例如：`skill:my-skill`）。
- 你可以正常跑通一套闭环：
  - `agentpack update`
  - `agentpack preview --diff`
  - `agentpack deploy --apply`

## 1) 创建 sparse overlay（推荐）

先只创建元数据（不拷贝 upstream 文件），然后只把你真正要改的文件放进 overlay 目录。

- `agentpack overlay edit <module_id> --sparse`

提示：
- 如需明确优先级，可加 `--scope global|machine|project`。
- 自动化（`--json`）模式下，`overlay edit` 属于写盘命令，需要 `--yes`。

## 2) 小改动优先用 patch overlays（可选）

如果只是改 upstream 文件的少量行，patch overlays 可以避免把整棵文件树复制到 overlay 里。

1) 创建 patch overlay：
- `agentpack overlay edit <module_id> --kind patch --scope global`

2) 在 `.agentpack/patches/` 下放一个 unified diff patch（例如：`.agentpack/patches/SKILL.md.patch`）。

如果 patch 在 `plan`/`deploy` 阶段无法应用，会以稳定错误码 `E_OVERLAY_PATCH_APPLY_FAILED` 失败（详见 `docs/reference/error-codes.md`）。

## 3) materialize：按需拷贝 upstream 用于浏览（可选）

- `agentpack overlay edit <module_id> --materialize`

该操作会以 missing-only 方式把 upstream 文件补齐到 overlay（不会覆盖你已有的 overlay 改动）。

## 4) upstream 更新后 rebase

在 upstream 更新（通常先执行 `agentpack update`）后：

- `agentpack overlay rebase <module_id> --sparsify`

推荐顺序：
1) `agentpack update`
2) `agentpack overlay rebase <module_id> --sparsify`
3) `agentpack preview --diff`
4) `agentpack deploy --apply`

先 dry-run（不写盘）：
- `agentpack overlay rebase <module_id> --sparsify --dry-run`

## 5) 冲突处理

当 `overlay rebase` 返回 `E_OVERLAY_REBASE_CONFLICT`：

1) 打开 overlay 目录下的冲突标记文件，手动解决。
2) 重新执行：
   - `agentpack overlay rebase <module_id>`

对 patch overlays，还会额外写入一份可定位的冲突工件：

- `.agentpack/conflicts/<relpath>`（例如：`.agentpack/conflicts/SKILL.md`）
