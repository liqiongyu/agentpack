---
status: active
owner: liqiongyu
last_updated: 2026-01-19
superseded_by: ""
scope: docs
---

# Agentpack 综合后续改进计划（验收后）
> 目标：把“已经可用”的 Agentpack，推进到“对用户友好、对真实环境稳健、对维护者可持续”的状态。
> 本文是 **Codex 可直接执行** 的计划文档：包含 Spec（约束/接口/验收标准）、Epics（工作流分解）、Backlog（细粒度任务列表）。

---

## 0. 文档元信息
- **status**: active
- **last_updated**: 2026-01-19
- **baseline**: 你上传并通过静态验收的 `agentpack-v7_updated`（已具备 init/import/patch overlays/MCP confirm token 等能力）
- **适用读者**: Maintainers、Codex（作为开发代理）、贡献者
- **不适用**: 终端用户入门（终端用户请以未来 `docs/index.md` 为唯一入口）

---

## 1. 北极星目标与边界

### 1.1 北极星目标（North Star）
1) **个人重度用户优先**：多工具（Codex / Claude Code / VS Code / Cursor 等）、多 scope（repo/user）、多机器同步的日常闭环顺滑、少踩坑。
2) **安全与可控默认**：变更类操作默认保守、可预览、可回滚、可解释；自动化（`--json`/MCP）必须可审计、可拒绝。
3) **文档与代码同步**：用户能“看得懂、找得到、跟得上”；维护者能“改得动、不漂移”。
4) **维护成本可控**：targets 增加、生态变更、功能扩展不会让核心变成巨石；测试能尽早发现真实操作问题。

### 1.2 范围（In scope）
- **验收后改进计划**：修复验收中发现的“文档缺口/版本一致性/边界说明”等问题
- **用户文档完善计划**：重构文档信息架构（Diátaxis 风格），补齐入口、任务导向、参考手册生成与 doc-sync
- **测试完善计划**：补 E2E / Journey tests、真实环境模拟（多 scope、adopt、overlay rebase、multi-machine sync、MCP confirm）
- **规划文档清理计划**：合并/归档/删除中间执行态文档，建立 ADR 体系，避免文档丛林
- **维护性重构计划**：在不破坏外部契约的前提下，分阶段拆分 overlay/targets/mcp/handlers

### 1.3 非目标（Non-goals）
- 不服务 “git + dotfiles 管理器 + 脚本” 作为主要用户路线（不为了兼容而复杂化核心）
- 不做 GUI 大而全管理台（TUI 仅作为可选增强）
- 不引入中心化服务/云端控制面（保持本地优先）
- 不在本计划内新增大量新 target（先稳住质量与体验）

---

## 2. 全局原则（Codex 必须遵守）

### 2.1 外部契约优先（Do not break userspace）
- `--json` **schema_version=1**：只允许新增字段，不允许删除/重命名/改变语义。
- `ERROR_CODES`：稳定错误码不随便改；新增错误码需同时更新文档与测试。
- ADR（why）：`../adr/0001-json-contract-stability.md`

### 2.2 变更类命令必须可拒绝
- CLI：`--json` 模式下，任何写入动作必须要求 `--yes`（或等价的显式确认）。
- MCP：写入类工具必须两阶段（plan -> apply），且 apply 必须携带 **confirm_token**（或等价绑定）。
- ADR（why）：`../adr/0003-mcp-confirm-token.md`

### 2.3 Docs-as-code：代码改动必须带 docs/测试
- 任何新增 CLI flags/子命令：必须更新 reference，且加 doc-sync test 防漂移。
- 任何变更计划/apply 语义：必须新增或更新 E2E journey test 覆盖真实场景。

### 2.4 每个 Backlog item = 1 个小 PR
- 每个任务尽量控制在 200-500 行有效变更（不含测试/文档），避免大重写。
- PR 必须包含：动机、实现、测试命令、影响面、回滚策略。

---

## 3. Spec（本计划的可验收规格）

> 这里的 Spec 是“后续改进计划”的规格（文档/测试/重构/归档），不重复项目既有 `docs/SPEC.md` 的全量产品契约。

### 3.1 用户文档 Spec（Diátaxis + 单入口）
目标：用户第一天只靠 2-3 个页面就能成功；查参数只看 reference；概念解释与开发计划不污染用户入口。

#### 3.1.1 信息架构（IA）
必须建立如下目录结构（允许逐步迁移，但终局一致）：
- `docs/index.md`（唯一用户入口，包含“选路决策树”）
- `docs/tutorials/`（照做可跑通）
- `docs/howto/`（任务手册）
- `docs/reference/`（可查、完整、可靠；尽量自动生成）
- `docs/explanation/`（概念解释、设计理由）
- `docs/dev/`（贡献者/维护者/规划；默认不在用户导航）
- `docs/archive/`（历史快照/已过期计划文档；默认不在用户导航）

#### 3.1.2 文档发现性（Discoverability）
- 所有重要新能力必须能在 `docs/index.md` 的“选路”中被找到（例如 patch overlays、import、MCP confirm）。
- 所有 CLI flags 必须出现在 `docs/reference/cli.md`（自动生成或半自动）。

#### 3.1.3 文档漂移防护（Doc-sync）
- 必须有 CI 检查：当 CLI/flags 变更时，reference 必须同步（生成或 doc-sync test）。
- patch overlay 相关至少有 1 条 doc-sync test：确保 reference 与 overlays explanation 提到 `--kind patch`。
- ADR（why）：`../adr/0002-patch-overlays-design.md`

### 3.2 测试 Spec（分层 + 真实旅程 E2E）
目标：不只验证函数正确，更要验证“真实操作不会把用户环境搞坏”。

#### 3.2.1 测试分层
- Unit：小、快、纯逻辑
- Integration/Contract：验证 CLI/JSON 合约、targets conformance
- **Journey E2E**（本计划新增重点）：模拟用户真实操作路径（init/import/deploy/adopt/rollback/overlay rebase/multi-machine sync/MCP）

#### 3.2.2 E2E 的“真实性/可重复性”约束
- 所有 E2E 必须在临时目录中运行（temp HOME、temp config repo、temp project repo）。
- 默认禁止联网；不依赖系统全局状态。
- 输出可在不同 OS 上稳定断言（必要时只断言结构化 JSON 或稳定片段）。

#### 3.2.3 必备 E2E Journeys（至少 8 条）
J1 From-scratch first deploy
J2 Import existing assets（user + project 混合）
J3 Adopt_update 覆盖确认流程（拒绝->adopt->成功）
J4 Overlay sparse->materialize->rebase->deploy
J5 Patch overlay 生成->rebase->apply（冲突工件）
J6 Multi-machine sync（bare remote + rebase）
J7 Cross-target consistency（同一模块输出多个 targets）
J8 MCP 两阶段确认（plan->token->apply->rollback）

### 3.3 规划文档清理 Spec（合并/归档/删除）
目标：减少中间执行态文档的噪音；只保留“一个真源头”；历史可追溯但不干扰用户。

#### 3.3.1 单一真源头规则
- 每类规划文档只能有 **1 个 active**：
  - `docs/dev/roadmap.md`（当前唯一活的路线图/里程碑）
  - `docs/dev/codex.md`（当前唯一活的 Codex 执行指南）

#### 3.3.2 生命周期元数据（必须）
所有 planning/dev 文档必须在文件头包含：
```yaml
---
status: active | superseded | archived
owner: <name>
last_updated: YYYY-MM-DD
superseded_by: <path>
scope: docs | tests | refactor | governance | targets | release
---
```

#### 3.3.3 归档策略
- 旧的 `CODEX_EXEC_PLAN.md` / `CODEX_WORKPLAN.md` / `Agentpack_*CodexReady*.md` 统一迁移到 `docs/archive/`，并标记 `superseded_by`。
- 如果外部有引用：保留“墓碑页”（短文档，仅指向新位置），避免断链。

#### 3.3.4 ADR（Architecture Decision Records）
- 建立 `docs/adr/`，每个重大决策一页（title/status/context/decision/consequences）。
- ADR 是“长期解释”，替代把设计理由塞进 workplan。

### 3.4 可维护性重构 Spec（分阶段，不破坏外部行为）
目标：降低核心文件复杂度，减少新增 target/overlay/mcp 工具时的耦合与回归风险。

约束：
- 不改变 CLI/JSON/ERROR_CODES 外部契约
- 每次重构必须由：golden tests + conformance + 至少 1 条 journey E2E 覆盖

分阶段方向：
1) overlay 拆分
2) targets 从 engine 迁出
3) mcp 拆分
4) 统一 command handlers（CLI/MCP/TUI 共用）

---

## 4. Epics（工作流分解）

> Epics 以里程碑（Milestone）组织：每个 Milestone 下包含多个 Epics；每个 Epic 下包含细粒度 Backlog items。

### M0：发版/验收后的“阻断项”修复（P0）
- E0.1 文档缺口修复（patch overlays / import / MCP confirm）
- E0.2 版本一致性（changelog/tag/release/quickstart 对齐）
- E0.3 最小 doc-sync 检查落地（防止漂移回归）

### M1：用户文档体验重构 + 规划文档清理（P0-P1）
- E1.1 文档信息架构迁移（Diátaxis + 单入口）
- E1.2 Reference 自动生成（CLI/Config/Targets）
- E1.3 文档治理：合并/归档/删除 + ADR 引入

### M2：E2E/Journey 测试体系落地（P0-P1）
- E2.1 E2E harness（可复用 test env）
- E2.2 8 条必备 Journeys
- E2.3 CI 集成（可选 nextest、feature matrix）

### M3：可维护性重构（P1）
- E3.1 overlay 模块化拆分
- E3.2 targets 迁移与 registry
- E3.3 mcp 模块化拆分
- E3.4 handlers 抽象（CLI/MCP/TUI 共享）

### M4：治理轨（可选，强隔离）（P2）
- E4.1 “core 永不读 org config”的回归测试与文档
- E4.2 policy packs 的 trust chain（hash pin / allowlist）
- E4.3 CI/IDE 的治理集成指南（不影响个人默认路径）

---

## 5. Backlog（细粒度任务清单，Codex 可逐条执行）

> 规则：每个任务 = 1 PR。
> 每个任务必须写：实现点、文件改动范围、测试命令、文档更新点、验收标准。

### M0（P0）——阻断项修复

#### M0-DOC-001（P0）补齐 patch overlays 用户文档
- 目的：修复“功能已存在但文档找不到”的验收缺口
- 变更范围：
  - 更新 `docs/reference/cli.md`（或生成器）包含 `overlay edit --kind patch`
  - 更新 `docs/explanation/overlays.md`（新增 Patch overlays 小节：何时用/限制/冲突处理）
  - 更新 `docs/howto/overlays-create-sparse-materialize-rebase.md`（示例）
- 测试：
  - 新增 doc-sync test（见 M0-DOC-003）
- 验收：
  - 用户能通过 docs/index 的选路找到 patch overlays
  - CLI reference 明确列出 `--kind patch`
  - patch overlay 冲突处理至少有 1 个例子

#### M0-REL-002（P0）版本一致性：release/tag/changelog/quickstart 对齐
- 目的：避免用户按文档安装失败
- 变更范围：
  - 对齐 `CHANGELOG.md`、`docs/QUICKSTART.md`、`README.md`、GitHub release notes（如存在）
- 测试：
  - 加一个轻量文档检查（断言 quickstart 不引用不存在的 tag）
- 验收：
  - `docs/QUICKSTART.md` 的安装命令与真实可用版本一致

#### M0-DOC-003（P0）doc-sync：强制关键特性在文档中出现
- 目的：防止未来再次出现“功能存在但文档缺失”
- 实现：
  - 新增 `tests/doc_sync_patch_overlay.rs`：
    - 断言 reference/cli.md 或 overlays doc 包含关键片段（如 `--kind patch`、`patch overlays`）
- 验收：
  - 改动 CLI flags 但不更新文档会在 CI 失败

---

### M1（P0-P1）——用户文档体验 + 规划文档清理

#### M1-DOC-010（P0）建立 `docs/index.md` 作为唯一用户入口（选路决策树）
- 内容要求：
  - “从 0 开始” tutorial
  - “已有资产纳管” tutorial（import）
  - “日常闭环” how-to
  - “漂移治理” how-to（status->evolve propose）
  - “自动化” how-to（json/mcp）
  - “查参数” reference（cli/config/targets）
- 验收：
  - 用户不看其他文档也能定位到下一步

#### M1-DOC-011（P0）按 Diátaxis 创建目录并迁移现有文档（最小迁移）
- 变更：
  - 新建目录：`tutorials/ howto/ reference/ explanation/ dev/ archive/`
  - 迁移：QUICKSTART→tutorials、WORKFLOWS→howto、CLI→reference、ARCHITECTURE→explanation
- 验收：
  - docs/index.md 只链接到新的路径
  - 旧路径若被外部引用，保留墓碑页（指向新路径）

#### M1-DOC-012（P0）合并 Codex 执行文档：只保留一个 active `docs/dev/codex.md`
- 输入：`CODEX_EXEC_PLAN.md`、`CODEX_WORKPLAN.md`（以及类似中间文档）
- 输出：
  - `docs/dev/codex.md`：长期不变的执行规则、PR 模式、测试/文档要求
  - 旧文件迁入 `docs/archive/plans/` 并加 YAML 头 `status: superseded`
- 验收：
  - repo 顶层不再出现多个 “Codex plan” 并列文件
  - docs/index.md 不再链接这些中间文档

#### M1-DOC-013（P0）合并 Roadmap：只保留一个 active `docs/dev/roadmap.md`
- 输入：`Agentpack_Spec_Epics_Backlog_CodexReady.md`、`Agentpack_Roadmap_*_v0.6.md` 等
- 输出：
  - `docs/dev/roadmap.md`（active）
  - 旧版本迁入 `docs/archive/roadmap/YYYY-MM_<ver>.md`（archived）
- 验收：
  - 开发者只需要更新一个 roadmap

#### M1-DOC-014（P1）引入 ADR：`docs/adr/0001-*.md`
- 最少新增 3 条 ADR：
  - JSON contract stability
  - Patch overlays design
  - MCP confirm_token design
- 验收：
  - roadmap 中的“为什么”链接到 ADR，而不是塞进长 workplan

#### M1-DOC-015（P1）Reference 自动生成：新增 `agentpack docs gen`（或 `help --markdown`）
- 目标：减少手写 CLI reference 导致的漂移
- 输出：
  - `docs/reference/cli.md` 由工具生成（或半自动）
- 验收：
  - 在 CI 中校验生成结果无差异（`git diff --exit-code`）

#### M1-DOC-016（P1）文档断链检查（CI）
- 方案：
  - 引入 markdown link checker（或自写简单脚本）
- 验收：
  - docs/index.md 不存在 404/不存在路径

---

### M2（P0-P1）——E2E / Journey tests

#### M2-TST-001（P0）E2E Harness：`tests/journeys/common`（TestEnv 构建器）
- 能力：
  - temp HOME、temp project repo、temp config repo
  - 生成最小 manifests/modules
  - 提供命令运行助手（建议用 assert_cmd）
- 验收：
  - 后续每条 journey 只关注步骤，不重复搭环境代码

#### M2-TST-002（P0）引入 assert_cmd + predicates（dev-dependencies）
- 验收：
  - journey tests 使用统一断言方式，失败信息可读

#### M2-TST-010（P0）Journey J1：From-scratch first deploy
- 步骤：
  - init → update → preview --diff → deploy --apply → status → rollback
- 验收：
  - 所有命令退出码与关键输出符合预期

#### M2-TST-011（P0）Journey J2：Import existing assets（user+project）
- 验收：
  - import dry-run 不写盘
  - apply + yes 才写盘
  - 导入后 preview/deploy 成功

#### M2-TST-012（P0）Journey J3：Adopt_update 拒绝→adopt→成功
- 验收：
  - 无 adopt 时失败且错误码正确
  - adopt 后成功且不会再重复要求 adopt

#### M2-TST-013（P1）Journey J4：Overlay sparse→materialize→rebase→deploy
- 验收：
  - upstream 更新后 rebase 行为与冲突工件符合预期

#### M2-TST-014（P1）Journey J5：Patch overlay 生成→rebase→apply
- 验收：
  - patch 生成后能 apply
  - 冲突时生成可定位工件并返回稳定错误码

#### M2-TST-015（P1）Journey J6：Multi-machine sync（bare remote）
- 验收：
  - 两个 clone 间更新与 rebase 可复现

#### M2-TST-016（P1）Journey J7：Cross-target consistency
- 验收：
  - 同一模块输出多个 target 时 manifest/rollback 都正确

#### M2-TST-017（P1）Journey J8：MCP confirm（plan→token→apply→rollback）
- 验收：
  - 没 token 不能 apply
  - 错 token 拒绝
  - 正确 token 成功写盘

#### M2-TST-020（P1）CI：可选 nextest（提升稳定性/速度）
- 验收：
  - 在 CI 提供 `cargo nextest run` 路径（可选、可渐进）

---

### M3（P1）——可维护性重构（不破坏外部行为）

#### M3-REF-001（P1）overlay 拆分为子模块（layout/dir/patch/rebase）
- 验收：
  - 行为不变（golden/contract/E2E 至少跑一条）
  - 代码可读性显著提升（每个文件 < ~400 行，尽量）

#### M3-REF-002（P1）targets 从 engine 迁出到 `src/targets/*`
- 验收：
  - 新增 target 不需要改 engine 核心逻辑
  - conformance harness 仍可按 feature matrix 跑

#### M3-REF-003（P1）mcp 拆分为 server/tools/confirm
- 验收：
  - confirm_token 逻辑集中且可单测
  - 新增 MCP tool 不需要触碰 server 生命周期逻辑

#### M3-REF-004（P1）统一 command handlers（CLI/MCP/TUI 共享）
- 验收：
  - CLI/MCP 的业务逻辑单源
  - 变更类命令的 guardrails 在 handler 层统一生效

---

### M4（P2，可选）——治理轨（强隔离，不影响个人默认路径）

#### M4-GOV-001（P2）回归测试：core 永不读取 org config
- 验收：
  - 给 `agentpack.org.yaml` 写入特殊值也不会影响 deploy/status/preview 等 core 命令

#### M4-GOV-002（P2）policy lint/lock 的 JSON contract 与错误码补齐 golden tests
- 验收：
  - policy 输出可被自动化稳定消费

#### M4-GOV-003（P2）policy lock：增加 hash pin / allowlist（最小 trust chain）
- 验收：
  - policy pack 来源变更可检测、可审计

---

## 6. 全局验收与交付标准（Definition of Done）

对任何 Backlog item：
1) 代码实现完成
2) 至少 1 个测试覆盖（unit/integration/E2E 视任务性质）
3) 必要文档更新完成（用户文档或 dev 文档）
4) 不破坏 `--json` 契约与稳定错误码
5) CI 通过（至少：unit + integration/contract；E2E 可渐进）

---

## 7. 附录：参考资料（供维护者理解方法论）
> 为了避免在用户文档里堆方法论链接，这些参考资料只放在本计划附录。
> （如需粘贴到 README，请放到代码块或单独 References 页。）

```
Diátaxis（Tutorial/How-to/Reference/Explanation）: https://diataxis.fr/start-here/
Rust 官方：tests/ 目录与集成测试组织：https://doc.rust-lang.org/book/ch11-03-test-organization.html
assert_cmd（CLI 集成测试）：https://docs.rs/assert_cmd
cargo-nextest（更快更稳定的 test runner）：https://nexte.st/
Docs as Code（Write the Docs）：https://www.writethedocs.org/guide/docs-as-code.html
ADR（Cognitect / Michael Nygard）：https://www.cognitect.com/blog/2011/11/15/documenting-architecture-decisions
ADR 社区站点：https://adr.github.io/
```
