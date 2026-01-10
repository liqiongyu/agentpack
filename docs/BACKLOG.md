# BACKLOG.md

## 优先级约定
P0：v0.1 必须完成（可用）
P1：v0.2 重要增强
P2：v1.0 扩展与生态

## Milestone v0.1（P0）

### Epic A：Core（manifest/lock/store）
- [P0] A1. 配置目录与 repo 初始化（XDG/OS 兼容）
- [P0] A2. YAML schema 校验（version, modules, targets, profiles）
- [P0] A3. Source resolver：local_path + git(url/ref/subdir)
- [P0] A4. lockfile 生成（稳定排序、sha256、file manifest）
- [P0] A5. store fetch/cache（可复现、hash 校验）

### Epic B：Overlay（最小可用）
- [P0] B1. project_id 识别（git origin -> hash；无 origin 用 path hash）
- [P0] B2. overlay 合成器（upstream + global + project）
- [P0] B3. overlay edit：生成 overlay skeleton + 打开编辑器
- [P0] B4. 冲突/漂移 warning（upstream 更新导致覆盖文件变动）

### Epic C：Deploy Pipeline（plan/diff/apply/rollback）
- [P0] C1. plan 输出（结构化变更列表）
- [P0] C2. diff 输出（文本 diff + JSON 摘要）
- [P0] C3. apply（备份 + 原子写入：temp->rename）
- [P0] C4. validate（写后读取+简单语法校验）
- [P0] C5. snapshot（deployments/<id>.json）
- [P0] C6. rollback（按 snapshot 恢复）

### Epic D：Codex Adapter（skills/prompts/AGENTS）
- [P0] D1. codex paths 解析（CODEX_HOME、repo_root、cwd）
- [P0] D2. skills deploy（repo scope + user scope 可配置）
- [P0] D3. prompts deploy（user scope ~/.codex/prompts）
- [P0] D4. AGENTS deploy（global ~/.codex/AGENTS.md + repo AGENTS.md）
- [P0] D5. status/drift 读取（hash 对比）

### Epic E：Claude Code Adapter（commands）
- [P0] E1. repo commands deploy（.claude/commands）
- [P0] E2. user commands deploy（~/.claude/commands）
- [P0] E3. frontmatter 最小校验（description/allowed-tools）
- [P0] E4. status/drift

### Epic F：AI-first Bootstrap（operator assets）
- [P0] F1. 内置 Codex skill：agentpack-operator（模板）
- [P0] F2. 内置 Claude commands：/ap-plan /ap-deploy /ap-status /ap-diff（模板）
- [P0] F3. bootstrap 命令实现（按 scope 写入）
- [P0] F4. --json schema 稳定化（所有核心命令）

### Epic G：质量与发布
- [P0] G1. 单元测试：resolver/lock/overlay 合成
- [P0] G2. Golden tests：adapter plan 输出快照
- [P0] G3. 跨平台 CI（mac/linux/windows）
- [P0] G4. 打包发布（单二进制 + shell completions）

## Milestone v0.2（P1）

### Epic H：TUI
- [P1] H1. 列表：profiles/modules/targets
- [P1] H2. 一键 plan/deploy/rollback
- [P1] H3. Drift 视图与修复建议

### Epic I：Evals gate（可选但强烈建议）
- [P1] I1. agentpack eval：运行 repo/agentpack/evals/*.sh
- [P1] I2. deploy --apply 前可配置必须 eval 通过
- [P1] I3. refine 工作流雏形：生成 patch -> eval -> apply（默认需要确认）

### Epic J：Registry 搜索（轻量）
- [P1] J1. GitHub search（skills/commands 仓库索引）
- [P1] J2. “解释模块” inspect：显示会写哪些文件、支持哪些 targets
- [P1] J3. 安全审计：显示 source/commit/license（尽可能）

### Epic K：Claude plugin 输出模式（高级）
- [P1] K1. 生成 .claude-plugin/plugin.json
- [P1] K2. marketplace.json 生成
- [P1] K3. cache 风险提示与 debug 命令

## Milestone v1.0（P2）
- Cursor/VS Code adapters
- MCP module type 全量支持（配置层）
- 更强 overlay（patch/3-way merge）
- 生态贡献：模块包规范、官方 registry 对接
