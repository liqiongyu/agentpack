# Overlays（覆盖层）

> Language: 简体中文 | [English](../OVERLAYS.md)

Overlays 的目的：让你在不 fork 上游模块的情况下做本地定制，并且能在上游更新时尽量“可合并、可回滚、可 review”。

## 1) 覆盖层级与优先级

同一个模块的最终内容由 4 层组成（低 → 高）：
1) upstream（local_path 或 git checkout）
2) global overlay
3) machine overlay
4) project overlay

同路径文件的合成策略：高优先级文件覆盖低优先级文件。

## 2) 磁盘布局

Config repo 内：
- global: `repo/overlays/<module_fs_key>/...`
- machine: `repo/overlays/machines/<machine_id>/<module_fs_key>/...`
- project: `repo/projects/<project_id>/overlays/<module_fs_key>/...`

说明：
- `module_fs_key` 是从 `module_id` 派生的文件系统安全目录名（会 sanitize，并附带短 hash；前缀长度有上限，避免超长路径）。
- CLI 与 manifest 仍然用 `module_id`；`module_fs_key` 仅用于磁盘寻址。
- 为兼容历史目录命名，agentpack 会在读取 overlay 时尝试 legacy 目录（存在则使用）。

## 3) overlay 元数据（.agentpack）

每个 overlay 目录内都会有：
- `.agentpack/baseline.json`：记录创建 overlay 时的 upstream 指纹，用于 drift 警告与 3-way merge。
- `.agentpack/module_id`：记录原始 module_id（便于审计与诊断）。

规则：
- `.agentpack/` 为保留目录，不参与部署（不会写到 target roots）。

## 4) 创建与编辑：overlay edit

命令：
- `agentpack overlay edit <module_id> [--scope global|machine|project] [--kind dir|patch] [--sparse|--materialize]`

行为：
- 默认（不加 `--sparse/--materialize`）：
  - 若 overlay 不存在，会复制 upstream 模块的完整文件树到 overlay，然后打开 `$EDITOR`（如果设置了）。
- `--sparse`：
  - 创建“稀疏 overlay”：只创建 `.agentpack` 元数据目录，不复制上游文件。
  - 推荐做法：在 overlay 内只放你修改过的文件（差异最小，未来合并更轻松）。
- `--materialize`：
  - 以 missing-only 的方式把 upstream 文件补齐到 overlay（不覆盖已有 overlay edits）。
  - 适合“我想浏览/参考上游实现，但不想把整棵树都纳入 overlay diff”。

提示：
- `overlay edit` 是写入类命令；`--json` 模式下需要 `--yes`。

Overlay kinds：
- `--kind dir`（默认）：目录型 overlay，通过在 overlay 目录里放置目标文件来覆盖 upstream。
- `--kind patch`：补丁型 overlay，在 `.agentpack/patches/` 下存放 unified diff patch，并在生成 desired state 时将 patch 应用到 upstream 文件上。

Patch overlays（kind = `patch`）：
- 适合“只改一两行”的小修改：不需要把整份 upstream 文件复制进 overlay tree，diff 更小、更易 review。
- 限制：
  - 仅支持 UTF-8 文本文件（不支持二进制 patch）。
  - 每个 patch 文件 MUST 是单文件的 unified diff。
- 布局：
  - `.agentpack/patches/<relpath>.patch`（例如：`.agentpack/patches/SKILL.md.patch`）
- 创建：
  - `agentpack overlay edit <module_id> --kind patch` 只创建元数据与 `.agentpack/patches/`，不会复制 upstream 文件。
- 示例：给单个文件打补丁
  1) 创建 patch overlay：
     - `agentpack overlay edit <module_id> --kind patch --scope global`
  2) 添加 patch 文件（例：`.agentpack/patches/SKILL.md.patch`）：
     ```diff
     --- a/SKILL.md
     +++ b/SKILL.md
     @@ -1,4 +1,4 @@
      ---
      name: my-skill
     -description: base
     +description: base（patched）
      ---
     ```
- 失败处理：
  - 如果 patch 在 `plan`/`deploy` 期间无法应用，命令会失败并返回稳定错误码 `E_OVERLAY_PATCH_APPLY_FAILED`。

## 5) 上游更新后的合并：overlay rebase（3-way merge）

命令：
- `agentpack overlay rebase <module_id> [--scope ...] [--sparsify]`

用途：
- 上游模块更新后，把 overlay 的改动在新的 upstream 上“重新应用”，尽量自动解决简单冲突。

行为要点：
- 读取 `.agentpack/baseline.json` 作为 merge base，对 overlay 中的文件做 3-way merge。
- 对“复制进 overlay 但你其实没改”的文件（ours == base），会更新到最新 upstream，避免无意 pin 老版本。
- `--sparsify`：删除 rebase 后与 upstream 完全一致的 overlay 文件，让 overlay 尽量保持稀疏。
- 支持 `--dry-run`：只输出会发生什么，不写入。

冲突：
- 如果产生冲突，命令会失败并返回 `E_OVERLAY_REBASE_CONFLICT`，details 里包含冲突文件列表。
- 解决方式：打开冲突文件手工处理后，再跑一次 `overlay rebase`（或直接手工提交 overlay）。
 - 对 patch overlays，冲突时还会在 `.agentpack/conflicts/<relpath>` 写入一份可定位的冲突工件（例如：`.agentpack/conflicts/SKILL.md`）。

## 6) overlay path

命令：
- `agentpack overlay path <module_id> [--scope ...]`

用途：
- 打印 overlay 目录路径（human）或在 JSON 输出里提供 `data.overlay_dir`（便于脚本/agent 直接打开）。

## 7) 常见建议

- 优先使用 `--sparse`：减少 overlay 体积与未来合并成本。
- 只在需要浏览时再 `--materialize`。
- 上游更新后：先 `agentpack update`，再 `agentpack overlay rebase ...`，最后 `preview --diff` 看变化。
