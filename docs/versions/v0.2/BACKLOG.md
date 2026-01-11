
---

```markdown
# BACKLOG.md (v0.2)

> 以 Epic -> Story 的方式组织；每条 Story 需要验收标准（AC）。

## P0（必须做）

### EPIC A：Deploy 安全与幂等（manifest）
A1. 写入 `.agentpack.manifest.json`
- AC:
  - deploy --apply 后目标目录存在 manifest
  - manifest 列出所有 managedFiles，包含 hash/moduleId
A2. 删除保护：deploy 只能删除 manifest 中的文件
- AC:
  - 目标目录存在非托管文件时，deploy 不删除它
  - status 能提示 extra files，但不报错
A3. status/drift 基于 manifest 做校验
- AC:
  - 手动修改托管文件，status 能识别 changed
  - 手动删除托管文件，status 能识别 missing

### EPIC B：多机器同步（remote/sync + machine overlay）
B1. remote set / sync
- AC:
  - 能配置 git remote
  - sync 能执行 pull/rebase/push（至少封装 git 命令并输出提示）
B2. machineId 与 machine overlay
- AC:
  - doctor 输出 machineId
  - plan 能接受 --machine 并应用 machine overlays
  - overlay 生效可解释（plan provenance 里能看到）

### EPIC C：AI-first JSON 输出与 doctor
C1. plan/diff/status/deploy/lock/fetch 支持 --json
- AC:
  - 输出稳定 schemaVersion
  - 内容包含 warnings/errors
C2. doctor 命令
- AC:
  - doctor 输出 target 路径检查结果
  - 对不可写/不存在目录给出清晰建议

## P1（强烈建议做）

### EPIC D：进化最小闭环（record/score/propose）
D1. record：写入事件日志
- AC:
  - 支持从 stdin 读 JSON event 并落盘
  - event 可包含 moduleId、tool、success、feedback
D2. score：计算模块健康度
- AC:
  - 输出每个 module 的失败率、最近修改时间、回滚次数（如可得）
D3. evolve propose：产出 patch 提案（不自动 apply）
- AC:
  - 生成可 review 的 patch 文件（或 git 分支）
  - diff 命令能展示 patch 影响范围

### EPIC E：可解释性增强
E1. explain 子命令：explain plan/diff/status
- AC:
  - 能输出“为什么这个文件来自 overlay X 而不是 Y”的解释

## P2（可后置）

### EPIC F：更多 targets adapter
F1. Cursor adapter（如果 Cursor 有明确的 prompts/commands 目录约定）
F2. VSCode agent 生态适配（视具体工具而定）

### EPIC G：TUI
G1. 交互式查看 plan/diff/status（只读）
G2. deploy/rollback 的交互确认（对人更友好）

### EPIC H：MCP（后置）
H1. agentpack 自己暴露 MCP server（让其他 agent 通过 MCP 调用）
H2. MCP 资源管理增强（registry + enable/disable + version pin）
