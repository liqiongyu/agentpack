# Evolve（自进化闭环）

Evolve 的定位不是“自动修改你的系统”，而是把变化变成 **可 review、可回滚、可同步** 的改动：
- 变化发生在目标目录（例如你手工改了 `~/.claude/commands/ap-plan.md`）
- agentpack 负责把这类 drift 捕获为 overlays（在 config repo 创建分支），让你像 code review 一样审查与合并

相关命令：
- `agentpack record` / `agentpack score`
- `agentpack explain plan|diff|status`
- `agentpack evolve propose` / `agentpack evolve restore`

## 1) record / score（观测）

### record

`agentpack record` 会从 stdin 读取一段 JSON，并追加写入 `state/logs/events.jsonl`。

用法示例：
- `echo '{"module_id":"command:ap-plan","success":true}' | agentpack record`

约定：
- 顶层可以有 `module_id`/`success`，也可以放在内部 event 结构里。
- score 会容忍坏行（例如 JSON 截断），跳过并输出 warnings。

### score

`agentpack score` 基于 events.jsonl 做简单统计（例如失败率），用于：
- 找到“经常失败/需要修”的模块
- 作为 evolve 的优先级信号（未来可扩展）

## 2) explain（解释）

`agentpack explain plan|diff|status` 会解释：
- 某个输出文件来自哪个 module_id
- 来自哪一层 source（upstream/global/machine/project overlay）

这对排查“为什么是这个版本生效”很关键。

## 3) evolve propose（把 drift 变成 overlays）

命令：
- `agentpack evolve propose [--module-id <id>] [--scope global|machine|project] [--branch <name>]`

推荐流程：
1) 先看候选（不写入）：
- `agentpack evolve propose --dry-run --json`

2) 再创建提案分支：
- `agentpack evolve propose --scope global`

行为与限制：
- 这是写入类命令：`--json` 下必须 `--yes`。
- 会要求 config repo 是 git 仓库，并且工作区必须干净（否则拒绝）。
- 会创建分支（默认 `evolve/propose-<timestamp>`），把 drifted 文件写入对应 overlay 路径，然后 `git add -A` 并尝试 commit。
  - commit 失败（例如缺 git identity）时不会丢改动：分支与变更会保留。

### 哪些 drift 会被 propose？

默认是保守策略：只对“能安全映射回 source 的改动”自动提案。

1) **单模块输出**（推荐）：
- 某个输出文件的 `module_ids.len() == 1`
- 文件存在且内容不同（modified）

2) **聚合输出（Codex 的 AGENTS.md）**：
- 当多个 instructions 模块合成一个 `AGENTS.md` 时，agentpack 会在每个模块段落外包 marker：

```md
<!-- agentpack:module=instructions:one -->
...content...
<!-- /agentpack -->
```

- 若 deployed 与 desired 都包含 marker，evolve propose 可以逐模块对比段落差异，并把变更写回对应 instructions 模块的 overlay。

以下情况会被跳过（会在 `skipped` 里给 reason）：
- `missing`：文件不存在（见 evolve restore）
- `multi_module_output`：无法安全定位到单个模块
- `read_error`：文件读失败

## 4) evolve restore（恢复 missing 文件，create-only）

命令：
- `agentpack evolve restore [--module-id <id>]`

用途：
- 当某些托管文件被删除（missing），你只想把它们“创建回来”，不想更新/删除其他东西。

特性：
- create-only：只创建缺失文件
- 不更新已有文件
- 不删除任何文件

推荐：
- 先 `--dry-run --json` 看将要恢复哪些路径，再决定执行。

## 5) 与 overlays 的关系

- evolve propose 写入的内容本质就是 overlays。
- 你最终应该把它们通过 review 合入 config repo，然后 `deploy --apply` 让系统回到“期望态”。

如果你想手工做同样的事：
- 可以直接 `agentpack overlay edit --sparse <module_id>` 然后自己把 drift copy 进去。
