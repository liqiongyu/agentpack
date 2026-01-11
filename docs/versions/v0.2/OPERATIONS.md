
---

```markdown
# OPERATIONS.md (v0.2)

## 推荐的“一个仓库，多台机器”协作方式

### 1) 你应该把什么提交到远端？
建议提交：
- agentpack.yaml（模块声明与 profile）
- modules/（你的全局资产）
- overlays/（可共享的 overlay）
- templates/（生成用模板）
不建议提交：
- state/（lockfile、snapshots、logs 默认不提交，除非你明确要团队共享）
- cache/（机器相关）

### 2) 机器级覆盖怎么做？
两种策略选一种：

A. machine overlay 不入库（更干净）
- machine overlay 写到 `$AGENTPACK_HOME/state/machines/<machineId>.yaml`
- 不参与 git，同步靠你自己的 dotfiles 管理

B. machine overlay 入库（更可控）
- overlays/machines/<machineId>/...
- 提交到 repo，但明确只给自己用

### 3) 项目级覆盖怎么做？
推荐：
- 项目根目录放 `agentpack.project.yaml`
- 里面只写 projectOverlays，不动全局 repo
- 这样项目仓库可以自包含（适合团队协作）

### 4) 发生冲突怎么办？
- 使用 `agentpack sync --rebase`
- 冲突时 agentpack 输出冲突文件列表与建议：
  - 如果冲突发生在 modules：建议人工合并
  - 如果冲突发生在 machine overlay：建议改为不入库策略
