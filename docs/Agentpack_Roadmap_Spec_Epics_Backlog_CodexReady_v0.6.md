# Agentpack 后续迭代方向 + Codex 可执行 Spec/Epics/Backlog

> 当前基线：Agentpack **v0.6.0**（本仓库状态）。
> 文档生成日期：**2026-01-15**。
> 面向对象：维护者、人类贡献者、以及 **Codex/Claude/Cursor/VS Code** 等 coding agent（作为自动化执行者）。

本文件的目标是把「愿景/设计约束」和「可执行的 Epic/Backlog」收敛到一份**一致、无冲突、可直接拆成 PR** 的工程计划。
**默认优先级：重度个人用户（多工具/多机器）**；团队/组织治理是长期目标，但必须以**硬隔离**方式实现，确保不会拖累个人主线。

---

## 0. 使用方法（给 Codex 的执行规则）

1) **一条 backlog task = 一个 PR**（或一个很小的 commit group）。
2) PR 描述必须包含：动机、范围、非目标、验收标准、回归测试命令。
3) 每个 PR 至少运行：
   - `cargo fmt`
   - `cargo clippy --all-targets -- -D warnings`
   - `cargo test`
4) **任何用户可见行为变更**：必须同步更新至少一处文档（通常是 `docs/CLI.md` / `docs/SPEC.md` / `docs/JSON_API.md` / `docs/ERROR_CODES.md`）。
5) **任何 `--json` 输出变化**：
   - 只能做 *additive*（新增字段），除非同时 bump `schema_version` 并给出迁移说明；
   - 必须补充 golden tests（见 M1-E4）。
6) 任何 target 行为变化：
   - 必须通过 `docs/TARGET_CONFORMANCE.md` 定义的语义测试；
   - 必须更新 target mapping 文档（`docs/TARGETS.md` 或新增 target 小节）。

---

## 1. 产品北极星与范围

### 1.1 北极星（North Star）
把散落在不同 AI coding 工具中的“指令资产”（instructions / skills / commands / prompts / MCP wiring）变成一种可治理的本地基础设施：
- 声明式（desired state）
- 可预览/可 diff（plan）
- 安全可应用（apply）
- 可回滚（rollback）
- 可检测漂移并回收成 overlay（drift → evolve propose）
- 可被 agent 稳定自动化（`--json` + 稳定错误码）

### 1.2 核心用户（主线）
**重度个人用户**：
- 同时使用 Codex + Claude Code + （可能）Cursor/VS Code
- 多机器 / 多项目 / 多工作目录
- 想要：一致的 coding 规范与操作手册（而不是每个工具各写一份）
- 想要：升级/回滚/漂移修复可控
- 愿意接受“比复制文件略复杂”的工具，但不愿接受“像 K8s 一样重”

### 1.3 长期用户（战略，但必须隔离）
**团队/组织**：
- 希望把“AI coding 规范”纳入工程治理（policy-as-code、CI、供应链、审计）
- 但任何组织能力必须是 **opt-in**，不能影响个人默认路径（见 §2.3 硬隔离约束）

### 1.4 非目标（明确不做）
- 不做通用 dotfiles 管理器替代（chezmoi/yadm/stow 生态不是主要竞争对手）
- 不做 GUI 为主的产品形态（TUI 仅做轻量浏览器，且可选）
- 不做 agent runtime / multi-agent orchestration 平台（Agentpack 是资产控制面，不是 agent 框架）
- 不做“自动帮用户到处改配置”的黑盒魔法：一切写入必须可预览、可审计、可回滚、需要显式确认

---

## 2. 顶层设计约束（必须长期成立的“不变量”）

### 2.1 安全默认（Safe-by-default）
- 默认不删除非托管文件；默认不覆盖用户已有文件
- 对覆盖/接管（adopt）必须显式确认（CLI 与 MCP 都必须有强确认语义）
- 对写入类操作：默认 dry-run/preview；apply 必须显式 `--apply` + `--yes`

### 2.2 合约优先（Do not break userspace）
- `--json` 输出是**公共 API**：字段语义必须稳定；错误码必须稳定
- 任何破坏性变更必须：
  - bump `schema_version`
  - 更新 `docs/JSON_API.md`
  - 更新 golden tests
  - 提供迁移说明与兼容期策略

### 2.3 组织治理硬隔离（不能影响个人主线）
- 组织/治理配置必须与个人配置物理隔离（例如 `agentpack.org.yaml`），且核心命令**默认不得读取**它
- 组织能力默认只读（lint/check/report），除非用户显式启用“组织 apply”模式
- 工程隔离建议（满足其一即可）：
  - `agentpack org ...` 子命令树 + 独立配置文件 + 默认不加载
  - 或单独发行/feature gating 的 `agentpack-org` 变体

### 2.4 单一语义来源（Single source of truth）
- CLI 是语义源头
- MCP server 只是结构化封装层：必须复用同一套核心执行逻辑（不要复制一套 apply/diff）
- Operator assets（skills/commands/prompts）是“引导层”，执行仍应落到 CLI/MCP

---

## 3. 未来迭代方向总览（从 v0.6 往后）

这部分是“方向版”，下面 §4 会给出可执行 backlog。

### 3.1 先赢个人主线：上手 + 纳管 + 漂移闭环
下一阶段最关键的缺口是：**把用户已存在的资产一键纳管**。
重点不是加更多 target，而是让用户第一天就能把 `.claude/commands`、`~/.codex/prompts`、现有 AGENTS.md/skills 等导入为 modules+overlays，然后进入 `status → evolve` 的日常闭环。

### 3.2 把漂移治理做到“不可替代”
继续强化：
- `status`（可操作、可建议下一步、对 agent 友好）
- `evolve propose/restore`（覆盖更多 target 输出、结构化 skipped reasons、生成可 review 的 overlay/patch）
- overlays（从 dir override 进化到 patch overlays，减少冲突噪音）

### 3.3 生态变化用 TargetAdapter + Conformance 抵消
把 “target 变了就崩” 变成 “改 adapter + fixtures + conformance 就能快发版”。
优先建设：conformance harness、fixture 生成器、feature-gated targets、以及清晰的 mapping 文档模板。

### 3.4 集成策略：CLI 做强 + operator assets 做好 + MCP 做成薄而稳
- CLI：最小共同分母，必须强
- operator assets：让 Codex/Claude 真的“想得到并会用” Agentpack（尤其是 skills/commands）
- MCP：让执行变成结构化工具调用，并在跨 host（Codex CLI/IDE、VS Code、Cursor、Claude 等）保持一致；同时严格遵守 MCP 对“工具=任意代码执行、必须显式同意”的安全原则

---

## 4. Roadmap（Milestones → Epics → Tasks）

优先级标记：
- **P0**：直接影响个人主线 adoption 或合约稳定性
- **P1**：显著提升个人日用体验
- **P2**：生态扩展/长期能力

每个 Task 都必须能独立成 PR。

---

# Milestone M1（建议版本 v0.7）：上手与纳管（Import / Guided init / 行动导向输出）

## M1-E1（P0）`agentpack import`：把现有资产一键纳入管理

### M1-E1-T1：Import 设计与 CLI 入口
**目标**：新增 `agentpack import`（默认 dry-run），把现有 assets 扫描并转换为 modules + overlays 提案。
**范围**：
- 支持扫描至少：
  - repo 内：`AGENTS.md`、`.claude/commands/*`、项目内 skills（如存在）
  - user 内：Codex `~/.codex/prompts/*`、Codex `~/.codex/skills/*`、Claude `~/.claude/*`（只扫描，不默认写）
- 输出一个“导入计划”（plan items），包含：
  - 将生成哪些 module（id、tags、targets）
  - 将生成哪些 overlay（scope：global/machine/project）
  - 是否存在冲突/需要 adopt
**非目标**：第一次不做“自动推断复杂组织结构”；只做可解释的直接映射。
**验收**：
- `agentpack import --dry-run --json` 输出 schema_version=1 envelope，且含稳定字段：
  - `results.import.plan[]`（新增字段允许 additive）
  - `next_actions[]`（建议：`init`/`add`/`deploy`）
- `agentpack import --apply --yes` 在一个临时目录里能完成导入（不会写真实 home，除非用户显式指定 `--home-root <tmp>` 测试模式）。
**测试**：
- fixtures：构造模拟 home/repo 目录树；测试 import 计划稳定且 deterministic。
**文档**：
- `docs/CLI.md` 增加 import 章节
- `docs/WORKFLOWS.md` 增加“从已有环境迁移”流程

### M1-E1-T2：导入映射规则（最小可用且可解释）
**目标**：定义并实现一套最小映射规则，让用户能预测“导入后会变成什么”。
**规则建议**：
- 每类 asset 默认一个 module family：
  - instructions: `instructions:<name>`
  - skill: `skill:<name>`
  - command: `command:<name>`
  - prompt: `prompt:<name>`
- tags 默认：`imported`, `user`/`project`, `codex`/`claude_code`/`cursor`/`vscode`
- overlays 默认策略：
  - user-scope 资产导入为 `global` overlay（或单独的 `machine`，视你现有 overlay scope 语义）
  - repo-scope 资产导入为 `project` overlay
**验收**：
- 文档中给出 3 个最常见例子（repo-only、user-only、mixed），并展示导入后的目录结构。

### M1-E1-T3：冲突与 adopt 的安全策略（导入不许搞炸）
**目标**：import 过程中不允许默认覆盖现存内容；必须生成明确冲突报告。
**验收**：
- 发现冲突时：
  - dry-run：输出冲突列表 + 建议动作（比如 `--adopt` 或 “换 module id”）
  - apply：若用户未显式选择 adopt，必须返回稳定错误码（例如 E_ADOPT_REQUIRED）
- 任何写入必须满足 atomic write（遵循基线 spec）。

---

## M1-E2（P0）`init --guided`：零知识也能建出正确的最小 config repo

### M1-E2-T1：`agentpack init --guided` 交互流程
**目标**：新增 `--guided`，在 TTY 下提供最小问答，引导用户生成 `agentpack.yaml`。
**范围**：
- 选择目标：codex/claude_code/cursor/vscode（多选）
- 选择 scope：只管理 repo / 同时管理 user
- 可选：自动 bootstrap operator assets
**验收**：
- guided init 生成的 config 能直接跑通 `update → preview --diff → deploy --apply --yes`
- 非 TTY 下 `--guided` 必须失败并提示使用非交互 flags（保持脚本友好）。

### M1-E2-T2：模板与示例仓库（降低学习成本）
**目标**：提供一个“最小可用模板”（templates/ 或 docs/examples）。
**验收**：
- 模板包含至少：
  - 一个基础 instructions module
  - 一个 Codex skill module（带 SKILL.md）
  - 一组 Claude `/ap-*` commands（由 bootstrap 安装）
- 文档里给“一屏跑通”的命令序列。

---

## M1-E3（P1）行动导向输出：`doctor/status/evolve` 的 next_actions 与枚举化 reasons

### M1-E3-T1：`status` JSON 增加 `summary` 与 `next_actions`
**目标**：让 agent 不靠 NLP 也能知道下一步做什么。
**验收**：
- `status --json` 在 envelope 中新增：
  - `results.status.summary`（按 root/target 分组统计）
  - `results.status.next_actions[]`（枚举 action + suggested command string）
- human 输出也要按 root/target 分组，并在尾部给“推荐命令”。

### M1-E3-T2：`evolve propose` skipped reasons 枚举化
**目标**：把 “为什么跳过某个 drift 项” 从字符串变成稳定枚举，让 agent 可自动分支处理。
**验收**：
- `evolve propose --json` 输出 `skipped[]`，每项必须包含：
  - `reason_code`（enum）
  - `reason_message`（人类可读）
  - `next_actions[]`

### M1-E3-T3：`doctor` 变更类操作统一确认语义
**目标**：doctor 若会写入，必须和 deploy 一致：dry-run 默认、apply 必须 `--yes`。
**验收**：
- JSON 模式下写入类 doctor 没有 `--yes` 必须返回稳定错误码。

---

## M1-E4（P0）合约锁定：JSON golden tests + 错误码回归

### M1-E4-T1：CLI JSON golden tests（核心命令覆盖）
覆盖命令建议：
- `init`, `update`, `preview/plan/diff`, `deploy`, `status`, `doctor`, `rollback`, `overlay`, `evolve`
**验收**：
- golden fixtures 跨平台稳定（路径规范化、换行）
- 任何字段变更必须显式更新 golden（并在 PR 中解释）

### M1-E4-T2：错误码覆盖与文档一致性检查
**验收**：
- CI 中增加“ERROR_CODES.md 与代码枚举一致性”检查（可用简单脚本/单测）

---

## M1-E5（P0）Conformance harness 强化（生态变化的保险丝）

### M1-E5-T1：临时 roots conformance（不写真实 home）
**验收**：
- 每个 target conformance 运行时只在 temp dir，且可并行

### M1-E5-T2：Windows/路径/权限边界用例
**验收**：
- 至少覆盖：非法字符、长路径、只读文件、权限不足
- 错误码稳定且信息可解释

---

# Milestone M2（建议版本 v0.8）：漂移治理与 overlays 进化（Patch overlays / evolve 扩展 / 轻量 TUI）

## M2-E1（P1）Patch-based overlays（按 spec 实现）

### M2-E1-T1：overlay_kind=patch 的存储与校验
**验收**：
- `.agentpack/overlay.json` 正确写入 overlay_kind
- patch overlays 不允许与 dir override 混用（配置错误有稳定错误码）

### M2-E1-T2：patch 应用（desired-state 生成阶段）
**验收**：
- `.agentpack/patches/<relpath>.patch` 统一 diff 能正确应用到 upstream UTF-8 文本
- 失败返回 `E_OVERLAY_PATCH_APPLY_FAILED`（或 spec 规定的码）
- 生成冲突工件（如 spec 中 `.agentpack/conflicts/`）并可被 `overlay rebase` 解释

### M2-E1-T3：`overlay edit --kind patch` 工作流
**验收**：
- 创建 patch overlay 时生成 baseline（如 spec 中 `.agentpack/baseline.json`）并支持稀疏编辑
- editor 交互合理（至少保证不破坏现有 dir overlay 流程）

### M2-E1-T4：Patch overlay 的 conformance tests
**验收**：
- 至少 3 类测试：apply 成功、hunk 冲突、上游变化 rebase 后恢复

---

## M2-E2（P1）Evolve 扩展：覆盖更多 target 输出 + 更好的归因

### M2-E2-T1：evolve propose 覆盖更多输出类型
**范围**：不仅是 AGENTS.md，也包括：
- Claude commands
- Codex skills/prompts
- VS Code MCP wiring / prompts（若已作为 target）
**验收**：
- 产出 overlay 提案时能标注来源（哪个 module、哪个 root、哪个 target）
- `explain` 能解释 evolve 产物的来源链

### M2-E2-T2：evolve propose “branch/commit” 更可控
**验收**：
- 支持 `--branch`，默认命名 deterministic（含 module_id/scope/timestamp）
- dry-run 与 apply 输出一致（除“写入/分支创建”差异）

### M2-E2-T3：evolve restore 扩展到 create-only 的更多场景
**验收**：
- 支持恢复缺失的 commands/skills/prompts 等（仅 create，不覆盖）

---

## M2-E3（P2）轻量 TUI（只读浏览器，避免 UI 膨胀）

### M2-E3-T1：TUI MVP（plan/diff/status/snapshots）
**验收**：
- 默认只读；不允许 deploy/apply
- 与 CLI 使用同一套内部数据结构（避免两套语义）

### M2-E3-T2：TUI 与 `--json` 数据结构对齐
**验收**：
- TUI 读取内部结构与 JSON 输出共享同一模型（避免 drift）

---

# Milestone M3（建议版本 v0.9）：Targets 平台化（模块化适配 + 更快跟进生态变化）

## M3-E1（P1）TargetAdapter modularization（feature gating + 可贡献）

### M3-E1-T1：targets 拆分为 crates / cargo features
**验收**：
- core build 不带所有 target 依赖
- `--features target_codex` 等可选启用
- docs 更新：如何添加新 target、如何跑 conformance matrix

### M3-E1-T2：target mapping 模板（docs + examples）
**验收**：
- 新 target 必须提供：
  - mapping 规则（输入/输出/roots）
  - examples
  - migration notes
  - conformance tests

---

## M3-E2（P2）新增 targets（严格 gate）
候选：JetBrains / Zed / 其它 IDE
**约束**：
- 必须先写 mapping doc
- 必须补 conformance
- 必须能在个人用户主线中提供“真实价值”（不是为了覆盖面而覆盖面）

---

# Milestone M4（建议版本 v1.0）：MCP 稳定化与“结构化执行面”（跨 host 复用 + 更强安全协议）

## M4-E1（P1）agentpack-mcp：工具集合稳定 + 单一语义来源

### M4-E1-T1：MCP 工具集最小化与版本化
**验收**：
- tools list 固定：`doctor`, `status`, `preview`, `diff`, `plan`, `deploy`, `rollback`, `evolve_propose`, `evolve_restore`, `explain`
- 每个 tool 输入/输出 schema 有文档（可复用 CLI JSON）

### M4-E1-T2：两阶段确认（confirm_token）
**动机**：MCP 规范强调 tools 是任意代码执行，host 必须显式获得用户同意；两阶段确认可以降低 TOCTOU 与注入风险。
**验收**：
- `deploy` tool 先返回 plan + `confirm_token`
- `deploy_apply` 必须提供 token，且 token 与 plan 哈希绑定
- token 过期/不匹配返回稳定错误码

### M4-E1-T3：Host 侧集成文档（Codex/VS Code）
**验收**：
- 给出 Codex 配置例子（CLI 与 IDE），并说明“配置共享”。
- 给出 VS Code `.vscode/mcp.json` 例子，并强调不要硬编码 secrets（对齐 VS Code 官方建议）。

---

## M4-E2（P1）Operator assets：让 agent “会用且安全地用”
**动机**：Codex/Claude 的“技能/命令/提示”是最便宜的分发与引导层。Codex skills 用 `SKILL.md` 定义能力并可携带 scripts/resources/assets。
**验收**：
- bootstrap 产物始终与当前 CLI 契约一致（`--json`、错误码、确认语义）
- 文档明确：哪些命令允许模型程序化调用、哪些必须人类触发（结合 Claude 的 allowlist/permissions 风格）

---

# Governance Track（v1+ 长期）：团队/组织“AI coding 规范治理”（必须 opt-in，不影响个人）

这条线的任何交付都必须满足 §2.3 隔离约束。建议以独立 milestone/label 管理。

## G-E0（P0）隔离与边界测试
- 增加 tests：确保 core 命令不读取 org config
- 确保 org 子命令默认只读

## G-E1（P1）Policy-as-code（lint/check）
- `agentpack org lint`：检查 repo 是否符合组织 policy
- policy 可版本化，可在 CI 中运行
- 输出必须有 `--json` 合约与稳定错误码

## G-E2（P1）供应链与审计（pin/sign/allowlist）
- 强制 git sources pin 到 tag/commit
- policy enforce allowlist remotes
- 生成审计报告（版本、来源、变更摘要）

---

## 5. 通用 Definition of Done（所有 task 的统一验收清单）

- [ ] CLI 行为有文档（`docs/CLI.md` 至少更新一处）
- [ ] 如影响 contract：更新 `docs/JSON_API.md` / `docs/ERROR_CODES.md` / `docs/SPEC.md`
- [ ] 有测试：unit/integration/fixture 或 golden；覆盖主要路径与失败路径
- [ ] conformance（若影响 target 行为）通过
- [ ] `cargo fmt` / `clippy` / `test` 通过
- [ ] 用户可见错误信息可解释，且 JSON 模式有稳定 error code

---

# Appendix A：基线 Implementation Contract（docs/SPEC.md v0.6.0）

# Spec (implementation contract)

> Current as of **v0.6.0** (2026-01-15). This is the project’s **single authoritative spec**, aligned to the current implementation. Historical iterations live in git history; the repo no longer keeps `docs/versions/` snapshots.

## 0. Conventions

Command name: `agentpack`

Config repo: the agentpack config repo (a local clone), by default at `$AGENTPACK_HOME/repo`.

Default data directory: `~/.agentpack` (override via `AGENTPACK_HOME`), with:
- `repo/` (config repo, git; contains `agentpack.yaml` and `agentpack.lock.json`)
- `cache/` (git sources cache)
- `state/snapshots/` (deploy/rollback snapshots)
- `state/logs/` (record events)

Optional durability mode: set `AGENTPACK_FSYNC=1` to request `fsync` on atomic writes (slower, but more crash-consistent).

Supported as of v0.6.0:
- targets: `codex`, `claude_code`, `cursor`, `vscode`
- module types: `instructions`, `skill`, `prompt`, `command`
- source types: `local_path`, `git` (`url` + `ref` + `subdir`)

All commands default to human-readable output; pass `--json` for machine-readable JSON (envelope includes `schema_version`, `warnings`, and `errors`).

### 0.1 Stable error codes in `--json` mode (external contract)

When `--json` is enabled, common actionable failures must return stable error codes in `errors[0].code`:
- `E_CONFIRM_REQUIRED`: in `--json` mode, a mutating command is missing `--yes`.
- `E_ADOPT_CONFIRM_REQUIRED`: would overwrite an existing unmanaged file (`adopt_update`), but `--adopt` was not provided.
- `E_CONFIG_MISSING`: missing `repo/agentpack.yaml`.
- `E_CONFIG_INVALID`: `agentpack.yaml` is syntactically or semantically invalid (e.g. missing default profile, duplicate module id, invalid source config).
- `E_CONFIG_UNSUPPORTED_VERSION`: `agentpack.yaml` `version` is unsupported.
- `E_LOCKFILE_MISSING`: missing `repo/agentpack.lock.json` but the command requires it (e.g. `fetch`).
- `E_LOCKFILE_INVALID`: `agentpack.lock.json` is invalid JSON.
- `E_LOCKFILE_UNSUPPORTED_VERSION`: `agentpack.lock.json` `version` is unsupported.
- `E_TARGET_UNSUPPORTED`: an unsupported target (manifest targets or CLI `--target` selection).
- `E_DESIRED_STATE_CONFLICT`: multiple modules produced different content for the same `(target, path)` (refuse silent overwrite).
- `E_OVERLAY_NOT_FOUND`: overlay directory does not exist (overlay not created yet).
- `E_OVERLAY_BASELINE_MISSING`: overlay baseline metadata is missing (cannot rebase safely).
- `E_OVERLAY_BASELINE_UNSUPPORTED`: baseline has no locatable merge base (cannot rebase safely).
- `E_OVERLAY_REBASE_CONFLICT`: overlay rebase produced conflicts requiring manual resolution.
- `E_POLICY_VIOLATIONS`: `policy lint` found one or more governance policy violations.
- `E_POLICY_CONFIG_MISSING`: missing `repo/agentpack.org.yaml` when running governance policy commands.
- `E_POLICY_CONFIG_INVALID`: `repo/agentpack.org.yaml` is invalid.
- `E_POLICY_CONFIG_UNSUPPORTED_VERSION`: `repo/agentpack.org.yaml` `version` is unsupported.

See: `ERROR_CODES.md`.

Note: In `--json` mode, unclassified/unexpected failures use the non-stable fallback code `E_UNEXPECTED` (see: `JSON_API.md` and `ERROR_CODES.md`).

## 1. Core concepts and data model

### 1.1 Module

Logical fields:
- `id: string` (globally unique; recommended `type/name`)
- `type: oneof [instructions, skill, prompt, command]`
- `source: Source`
- `enabled: bool` (default `true`)
- `tags: [string]` (used by profiles)
- `targets: [string]` (restrict to specific targets; default all)
- `metadata`:
  - `name` / `description` (optional)

### 1.2 Source

- `local_path`:
  - `path: string` (repo-relative path or absolute path)
- `git`:
  - `url: string`
  - `ref: string` (tag/branch/commit; default `main`)
  - `subdir: string` (path within repo; optional)
  - `shallow: bool` (default `true`)

### 1.3 Profile

- `name: string`
- `include_tags: [string]`
- `include_modules: [module_id]`
- `exclude_modules: [module_id]`

### 1.4 Target

- `name: oneof [codex, claude_code, cursor, vscode]`
- `mode: oneof [files]` (v0.1)
- `scope: oneof [user, project, both]`
- `options: map` (target-specific)

### 1.5 Project identity (for project overlays)

`project_id` generation rules (priority order):
1) hash of the normalized git remote `origin` URL (recommended)
2) if no remote: hash of the repo root absolute path

`project_id` must be stable (same project across machines).

## 2. Config files

### 2.1 `repo/agentpack.yaml` (manifest)

Example:

```yaml
version: 1

profiles:
  default:
    include_tags: ["base"]
  work:
    include_tags: ["base", "work"]

targets:
  codex:
    mode: files
    scope: both
    options:
      codex_home: "~/.codex"           # can be overridden by CODEX_HOME
      write_repo_skills: true          # write to $REPO_ROOT/.codex/skills
      write_user_skills: true          # write to ~/.codex/skills
      write_user_prompts: true         # write to ~/.codex/prompts
      write_agents_global: true        # write to ~/.codex/AGENTS.md
      write_agents_repo_root: true     # write to <repo>/AGENTS.md
  claude_code:
    mode: files
    scope: both
    options:
      write_repo_commands: true        # write to <repo>/.claude/commands
      write_user_commands: true        # write to ~/.claude/commands
      write_repo_skills: false         # optional: write to <repo>/.claude/skills
      write_user_skills: false         # optional: write to ~/.claude/skills

modules:
  - id: instructions:base
    type: instructions
    tags: ["base"]
    source:
      local_path:
        path: "modules/instructions/base"

  - id: skill:git-review
    type: skill
    tags: ["work"]
    source:
      git:
        url: "https://github.com/your-org/agentpack-modules.git"
        ref: "v1.2.0"
        subdir: "skills/git-review"

  - id: prompt:draftpr
    type: prompt
    tags: ["work"]
    source:
      local_path:
        path: "modules/prompts/draftpr.md"

  - id: command:ap-plan
    type: command
    tags: ["base"]
    source:
      local_path:
        path: "modules/claude-commands/ap-plan.md"
```

Notes:
- `instructions` module sources point to a directory, which may contain:
  - `AGENTS.md` (template)
  - rule fragments (future extension)
- `skill` module sources point to the skill directory root (contains `SKILL.md`)
- `prompt` module sources point to a single `.md` file (Codex custom prompt)
- `command` module sources point to a single Claude slash command `.md` file

### 2.2 `repo/agentpack.lock.json` (lockfile)

Minimal fields:
- `version: 1`
- `generated_at: ISO8601`
- `modules: [ { id, type, resolved_source, resolved_version, sha256, file_manifest } ]`

Where:
- `resolved_source: { ... }`
- `resolved_version: string` (commit sha or semver tag)
- `file_manifest: [{path, sha256, bytes}]`

Requirements:
- The lockfile must be diff-friendly (stable JSON key order; stable array ordering).
- `fetch` can only use lockfile `resolved_version` values.
- For `local_path` modules: `resolved_source.local_path.path` must be stored as a repo-relative path (never absolute), and must use `/` separators to keep cross-machine diffs stable.

### 2.3 `repo/agentpack.org.yaml` (governance policy config; opt-in)

This file is **optional** and is only read by `agentpack policy ...` commands. Core commands (`plan/diff/deploy/...`) MUST NOT read it.

Minimal schema (v1):
- `version: 1`
- Optional `policy_pack`:
  - `source: string` (source spec; see below)
- Optional `distribution_policy`:
  - `required_targets: string[]` (must exist under `repo/agentpack.yaml -> targets:`)
  - `required_modules: string[]` (must exist in `repo/agentpack.yaml -> modules:` and be `enabled: true`)

Source spec syntax (same as `agentpack add`):
- `local:<repo-relative-path>`
- `git:<url>[#ref=<ref>&subdir=<path>&shallow=<true|false>]`

Example:

```yaml
version: 1

policy_pack:
  source: "git:https://github.com/your-org/agentpack-policy-pack.git#ref=v1.0.0&subdir=pack"

distribution_policy:
  required_targets: ["codex", "claude_code"]
  required_modules: ["instructions:base"]
```

### 2.4 `repo/agentpack.org.lock.json` (governance policy lockfile; opt-in)

This lockfile is generated by `agentpack policy lock` and pins a configured `policy_pack` for auditability and CI reproducibility.

Minimal fields:
- `version: 1`
- `policy_pack: { source, resolved_source, resolved_version, sha256, file_manifest }`

Where:
- `policy_pack.source` is the configured source (local path or git URL/ref/subdir/shallow).
- `policy_pack.resolved_source.git.commit` pins git sources to an immutable commit SHA.
- `sha256` and `file_manifest[]` are deterministic content hashes (diff-friendly; stable ordering).

### 2.5 `<target root>/.agentpack.manifest.json` (target manifest)

Goals:
- Safe delete (delete managed files only)
- Drift/status (`modified` / `missing` / `extra`)

Schema (v1 example):

```json
{
  "schema_version": 1,
  "generated_at": "2026-01-11T00:00:00Z",
  "tool": "codex",
  "snapshot_id": "optional",
  "managed_files": [
    {
      "path": "skills/agentpack-operator/SKILL.md",
      "sha256": "…",
      "module_ids": ["skill:agentpack-operator"]
    }
  ]
}
```

Requirements:
- `path` must be a relative path and must not contain `..`.
- The manifest records only files written by agentpack deployments; never treat user-native files as managed files.
- Readers MUST tolerate unsupported `schema_version` by emitting a warning and treating the manifest as missing (fall back behavior).

### 2.4 `state/logs/events.jsonl` (event log)

The event log written by `agentpack record` is JSON Lines (one JSON object per line).

Line shape (v1 example):

```json
{
  "schema_version": 1,
  "recorded_at": "2026-01-11T00:00:00Z",
  "machine_id": "my-macbook",
  "module_id": "command:ap-plan",
  "success": true,
  "event": { "module_id": "command:ap-plan", "success": true }
}
```

Conventions:
- `event` is arbitrary JSON; `score` only parses `module_id|moduleId` and `success|ok`.
- Top-level `module_id` and `success` are optional (compat with historical logs); `score` prefers them if present.
- `score` must tolerate bad lines (truncated / invalid JSON): skip with a warning rather than failing the entire command.
- Compatibility:
  - Adding new top-level fields is allowed (old readers ignore unknown fields).
  - If a line has an unsupported `schema_version`: skip with a warning (do not abort the whole command).
  - `score --json` includes skipped line counts and reason stats in `data.read_stats` to help diagnose log health.
- Optional top-level fields (additive, v1): `command_id`, `duration_ms`, `git_rev`, `snapshot_id`, `targets`.

## 3. Overlays

### 3.1 Overlay layers and precedence

Final composition order (low → high):
1) upstream module (local repo dir or cached checkout)
2) global overlay (`repo/overlays/<module_fs_key>/...`)
3) machine overlay (`repo/overlays/machines/<machine_id>/<module_fs_key>/...`)
4) project overlay (`repo/projects/<project_id>/overlays/<module_fs_key>/...`)

Where:
- `module_fs_key` is a cross-platform-safe directory name derived from `module_id` (sanitized, plus a short hash to avoid collisions).
- The CLI and manifests use the original `module_id`; `module_fs_key` is only for disk addressing.

### 3.2 Overlay representation (v0.2)

Overlay uses a “file override” model:
- overlay directory structure mirrors the upstream module
- same-path files override upstream

Patch overlays
- overlays may declare `overlay_kind: "dir" | "patch"` (default = `dir`)
  - `overlay_kind` is stored at `<overlay_dir>/.agentpack/overlay.json`
  - format: `{ "overlay_kind": "dir" | "patch" }`
- `overlay_kind=patch` stores unified diff patch files under `.agentpack/patches/` and applies them to upstream UTF-8 text files during desired-state generation
  - patch overlays only support UTF-8 text files
  - each `.patch` MUST represent a single-file unified diff, and its header path MUST match the patch filename-derived `<relpath>`
- a single overlay directory MUST NOT mix directory override files and patch artifacts (treat as configuration error)
- on patch apply failure, commands return stable error code `E_OVERLAY_PATCH_APPLY_FAILED`

Patch layout:
- `<overlay_dir>/.agentpack/patches/<relpath>.patch`
  - `<relpath>` is the POSIX-style path within the upstream module root (no absolute paths; no `..`)

### 3.3 Overlay editing commands (see CLI)

`agentpack overlay edit <module_id> [--scope global|machine|project] [--kind dir|patch] [--sparse|--materialize]`:
- if the overlay does not exist: by default it copies the entire upstream module tree into the overlay directory (scope path mapping below)
- opens the editor (`$EDITOR`)
- after saving: changes take effect via deploy

Implemented options:
- `--kind patch`: create a patch overlay skeleton (metadata + `.agentpack/patches/`) without copying upstream files, and set `<overlay_dir>/.agentpack/overlay.json` to `overlay_kind=patch`.
- `--sparse`: create a sparse overlay (write metadata only; do not copy upstream files; users add only changed files).
- `--materialize`: “fill in” missing upstream files into the overlay directory (copy missing files only; never overwrite existing overlay edits).

`agentpack overlay rebase <module_id> [--scope global|machine|project] [--sparsify]`:
- reads `<overlay_dir>/.agentpack/baseline.json` as merge base
- performs 3-way merge for files modified in the overlay (merge upstream updates into overlay edits)
- for `overlay_kind=patch`, rebase operates on `.agentpack/patches/<relpath>.patch` instead of overlay override files
  - it computes the edited content by applying the patch to the baseline version of `<relpath>`
  - it merges edited content against the latest upstream version using a 3-way merge
  - on success, it rewrites the patch file to apply cleanly to the latest upstream version
  - on conflicts, it writes conflict-marked full file content under `<overlay_dir>/.agentpack/conflicts/<relpath>` and returns `E_OVERLAY_REBASE_CONFLICT`
  - if the patch becomes a no-op after rebase, it deletes the patch file (empty patches are not supported) and prunes now-empty parent directories under `.agentpack/patches/`
- for files that were copied into overlay but not modified (`ours == base`): update them to latest upstream (avoid unintentionally pinning old versions)
- on success: refresh baseline (so drift warnings are computed from the latest upstream)
- on conflicts: overlay files contain conflict markers; in `--json` mode return stable error code `E_OVERLAY_REBASE_CONFLICT` (details include the conflict file list)

Optional:
- `--sparsify`: delete overlay files that are identical to upstream after rebase (keep overlays minimal).

Scope → path mapping:
- global: `repo/overlays/<module_fs_key>/...`
- machine: `repo/overlays/machines/<machine_id>/<module_fs_key>/...`
- project: `repo/projects/<project_id>/overlays/<module_fs_key>/...`

Compatibility:
- `--project` is still accepted but deprecated (equivalent to `--scope project`).

Additional (v0.3+):
- `agentpack overlay path <module_id> [--scope global|machine|project]`
  - human: prints absolute overlay dir path
  - json: returns `data.overlay_dir`

### 3.4 Overlay metadata (`.agentpack/`)

- Overlay skeleton writes `<overlay_dir>/.agentpack/baseline.json` for overlay drift warnings (not deployed).
- Overlay skeleton writes `<overlay_dir>/.agentpack/overlay.json` for `overlay_kind` (not deployed).
- Patch overlays store patch files under `<overlay_dir>/.agentpack/patches/` (not deployed).
- Patch overlay rebase conflicts may be written under `<overlay_dir>/.agentpack/conflicts/` (not deployed).
- `.agentpack/` is a reserved metadata directory: it is never deployed to target roots and must not appear in module outputs.

## 4. CLI commands (v0.6.0)

Global flags:
- `--repo <path>`: config repo location
- `--profile <name>`: default `default`
- `--target <name|all>`: default `all`
- `--machine <id>`: machine overlay id (default: auto-detected machineId)
- `--json`: JSON output
- `--yes`: skip confirmation prompts
- `--dry-run`: force no writes (even for `deploy --apply`); default false

Safety guardrails:
- In `--json` mode, commands that write to disk and/or mutate git require `--yes` (avoid accidental writes in scripts/LLMs).
- If `--yes` is missing: exit code is non-zero, stdout is still valid JSON (`ok=false`), and a stable error code `E_CONFIRM_REQUIRED` is returned in `errors[0].code`.

### 4.1 `init`

`agentpack init [--git] [--bootstrap]`
- creates `$AGENTPACK_HOME/repo` (use `--git` to also run `git init` and write/update a minimal `.gitignore`)
- writes a minimal `agentpack.yaml` skeleton
- creates a `modules/` directory

Optional:
- `--git`: ensure `.gitignore` contains `.agentpack.manifest.json` (idempotent).
- `--bootstrap`: install operator assets into the config repo after init (equivalent to `agentpack bootstrap --scope project`).

### 4.2 `add` / `remove`

- `agentpack add <type> <source> [--id <id>] [--tags a,b] [--targets codex,claude_code,cursor,vscode]`
- `agentpack remove <module_id>`

Source expressions:
- `local:modules/xxx`
- `git:https://...#ref=...&subdir=...`

### 4.3 `lock`

`agentpack lock`
- resolves all module sources
- generates/updates the lockfile

### 4.4 `fetch` (install)

`agentpack fetch`
- materializes lockfile modules into the cache (git sources checkout)
- validates sha256

v0.3+ behavior hardening (fewer footguns):
- when the lockfile exists but a `<moduleId, commit>` checkout cache is missing, `plan/diff/deploy/overlay edit` will auto-fetch the missing checkout (a safe network operation), rather than forcing users to run `fetch` manually first.

### 4.4.1 `update` (composite)

`agentpack update [--lock] [--fetch] [--no-lock] [--no-fetch]`
- default strategy:
  - if lockfile does not exist: run `lock` + `fetch`
  - if lockfile exists: run `fetch` only by default
- purpose: reduce friction in the common lock/fetch workflow, especially for AI/script orchestration.

Notes:
- In `--json` mode, `update` is treated as mutating and requires `--yes` (otherwise `E_CONFIRM_REQUIRED`).
- `--json` output aggregates steps: `data.steps=[{name, ok, detail}, ...]`.

### 4.4.2 `preview` (composite)

`agentpack preview [--diff]`
- always runs `plan`
- when `--diff` is set: also computes and prints diff (human: unified diff; json: diff summary)

Notes:
- `preview` is read-only and does not require `--yes`.

### 4.5 `plan` / `diff`

`agentpack plan`
- shows which targets/files would be written and what operation would be performed (`create` / `update` / `delete`)
- if multiple modules produce the same `(target, path)`:
  - same content: merge `module_ids` (for provenance/explain)
  - different content: error and return `E_DESIRED_STATE_CONFLICT` (block apply by default)

`agentpack diff`
- prints per-file text diffs; in JSON mode prints diff summary + file hash changes
- for `update` operations: JSON includes `update_kind` (`managed_update` / `adopt_update`)

### 4.6 `deploy`

`agentpack deploy [--apply] [--adopt]`

Default behavior:
- runs `plan`
- shows diff
- when `--apply` is set:
  - performs apply (with backup) and writes a state snapshot
  - writes `.agentpack.manifest.json` under each target root
- delete protection: only deletes managed files recorded in the manifest (never deletes unmanaged user files)
- overwrite protection: refuses to overwrite existing unmanaged files (`adopt_update`) unless `--adopt` is provided
- without `--apply`: show plan only (equivalent to `plan` + `diff`)

Notes:
- `--json` + `--apply` requires `--yes` (otherwise `E_CONFIRM_REQUIRED`).
- If the plan contains any `adopt_update`, apply requires `--adopt`; in `--json` mode, missing `--adopt` returns `E_ADOPT_CONFIRM_REQUIRED`.
- Even if the plan is empty, if the target root is missing a manifest, agentpack writes a manifest (so drift/safe-delete works going forward).

### 4.7 `status`

`agentpack status [--only <missing|modified|extra>[,...]]`
- if the target root contains `.agentpack.manifest.json`: compute drift (`modified` / `missing` / `extra`) based on the manifest
- if there is no manifest (or the manifest has an unsupported `schema_version`): fall back to comparing desired outputs vs filesystem, and emit a warning
- if installed operator assets (bootstrap) are missing or outdated: emit a warning and suggest running `agentpack bootstrap`
- `--only`: filters the drift list to the selected kinds (repeatable or comma-separated)
- in `--json` mode, `data.summary_total` MAY be included when filtering is used (additive)
- in `--json` mode, `data.next_actions` MAY be included (additive) to suggest common follow-up commands

### 4.8 `rollback`

`agentpack rollback --to <snapshot_id>`
- restores backups
- records a rollback event

### 4.9 `bootstrap` (AI-first operator assets)

`agentpack bootstrap [--target all|codex|claude_code|cursor|vscode] [--scope user|project|both]`
- installs operator assets:
  - Codex: writes one skill (`agentpack-operator`)
  - Claude: writes a set of slash commands (`ap-doctor`, `ap-update`, `ap-preview`, `ap-plan`, `ap-diff`, `ap-deploy`, `ap-status`, `ap-explain`, `ap-evolve`)
  - Claude (optional): writes one Skill (`agentpack-operator`) when enabled via `targets.claude_code.options.write_*_skills`
- asset contents come from embedded templates shipped with agentpack (updated with versions)
- each operator file includes a version marker: `agentpack_version: x.y.z` (frontmatter or comment)

Requirement:
- Skill files (`SKILL.md`) MUST start with YAML frontmatter and include non-empty `name` and `description` fields (validated during module materialization).
- If a Claude command uses bash execution, it must declare `allowed-tools` (minimal set).

Notes:
- In `--json` mode, `bootstrap` requires `--yes` (it writes to target roots; otherwise `E_CONFIRM_REQUIRED`).

### 4.10 `doctor`

`agentpack doctor [--fix]`
- prints machineId (used for machine overlays)
- checks target roots exist and are writable, with actionable suggestions (mkdir/permissions/config)
- git hygiene (v0.3+):
  - if a target root is inside a git repo and `.agentpack.manifest.json` is not ignored: emit a warning (avoid accidental commits)
  - `--fix`: idempotently appends `.agentpack.manifest.json` to that repo’s `.gitignore`
    - in `--json` mode, if it writes, it requires `--yes` (otherwise `E_CONFIRM_REQUIRED`)
- in `--json` mode, `data.next_actions` MAY be included (additive) to suggest common follow-up commands

### 4.11 `remote` / `sync`

- `agentpack remote set <url> [--name origin]`
- `agentpack sync [--rebase] [--remote origin]`

Behavior:
- wraps a recommended multi-machine sync flow with git commands (`pull --rebase` + `push`)
- does not resolve conflicts automatically; on conflict it fails and asks the user to handle it

### 4.12 `record` / `score`

- `agentpack record` (reads JSON from stdin and appends to `state/logs/events.jsonl`)
- `agentpack score` (computes failure rates from `events.jsonl`)

Event conventions (v0.2):
- `record` treats stdin JSON as `event` (no strict schema).
- `score` identifies:
  - module id: `module_id` or `moduleId`
  - success: `success` or `ok` (default to true if missing)

### 4.13 `explain`

`agentpack explain plan|diff|status`
- prints “provenance explanation” for changes/drift: moduleId + overlay layer (`project` / `machine` / `global` / `upstream`)

### 4.14 `evolve propose`

`agentpack evolve propose [--module-id <id>] [--scope global|machine|project]`
- captures drifted deployed file contents and generates overlay changes (creates a proposal branch in the config repo; does not auto-deploy)

Notes:
- In `--json` mode it requires `--yes` (otherwise `E_CONFIRM_REQUIRED`).
- Requires a clean working tree in the config repo; it creates a branch and attempts to commit.
  - If git identity is missing and commit fails, agentpack prints guidance and keeps the branch and changes.
- Current behavior is conservative: only generate proposals for drift that can be safely attributed to a single module.
  - By default it only processes outputs with `module_ids.len() == 1`.
  - For aggregated Codex `AGENTS.md` (composed from multiple `instructions` modules): if the file contains segment markers, agentpack tries to map drift back to the corresponding module segment and propose changes.
    - If markers are missing/unparseable, it skips with a `multi_module_output` reason.
  - It only processes drift where the deployed file exists but content differs; it skips `missing` drift (recommend `deploy` to restore).
  - Recommended flow: run `agentpack evolve propose --dry-run --json` to inspect `candidates` / `skipped` / warnings, then decide whether to pass `--yes` to create the proposal branch.

Aggregated instructions marker format (implemented; example):

```md
<!-- agentpack:module=instructions:one -->
# one
<!-- /agentpack -->
```

### 4.15 `evolve restore`

`agentpack evolve restore [--module-id <id>]`
- restores `missing` desired outputs to disk in a “create-only” way (creates missing files only; does not update existing files; does not delete anything)

Notes:
- In `--json` mode, if it writes, it requires `--yes` (otherwise `E_CONFIRM_REQUIRED`).
- Supports `--dry-run`: prints the file list only; does not write.

### 4.16 `help` / `schema` (utility commands)

`agentpack help`
- prints CLI help/usage
- `agentpack help --json` emits machine-consumable command metadata (see: `JSON_API.md`), including at minimum:
  - `data.commands[]` (command catalog)
  - `data.mutating_commands[]` (command IDs that require `--yes` in `--json` mode)
  - `data.global_args[]` (global flags)
  - `data.targets[]` (compiled-in target adapters)

`agentpack schema`
- prints a brief JSON schema summary (human mode)
- `agentpack schema --json` documents:
  - `data.envelope` (the `schema_version=1` envelope fields/types)
  - `data.commands` (minimum expected `data` fields for key read commands)

### 4.17 `tui` (optional)

`agentpack tui [--adopt]`

Availability:
- Feature-gated: only available when the agentpack binary is built with the `tui` feature.

Behavior:
- Interactive terminal UI for browsing `plan` / `diff` / `status`.
- Requires a TTY (intended for human interactive use).

Apply:
- Pressing `a` in the UI triggers apply for the current `--repo` / `--machine` / `--profile` / `--target`.
- Apply MUST require an explicit in-UI confirmation prompt; agentpack MUST NOT write to disk unless the user confirms.
- `--adopt` has the same semantics as `deploy --adopt` (allow overwriting existing unmanaged files / adopt updates).
- Respects `--dry-run` (no writes).

JSON mode:
- `tui` does not support `--json`; when `--json` is passed, it fails with `E_CONFIG_INVALID`.

### 4.18 `mcp serve` (MCP server, stdio)

`agentpack mcp serve`

Behavior:
- Runs an MCP server over stdio (newline-delimited JSON-RPC).
- Stdout is reserved for MCP protocol messages; logs and diagnostics MUST go to stderr.

Tools (minimum set):
- read-only: `plan`, `diff`, `status`, `doctor`
- mutating (explicit approval): `deploy_apply`, `rollback`

Tool results:
- Tool results reuse Agentpack’s `--json` envelope as the canonical payload, returned as structured content and as serialized JSON text.

JSON mode:
- `mcp serve` does not support `--json`.

### 4.19 `policy lint` (governance, read-only)

`agentpack policy lint`

Behavior:
- Read-only governance command (opt-in) for CI-friendly “asset hygiene” checks.
- Lints a repository root selected via `--repo <path>` (default: `$AGENTPACK_HOME/repo`).
- Initial checks (additive over time):
  - Skill frontmatter completeness: every `SKILL.md` MUST include YAML frontmatter with non-empty `name` and `description`.
  - Claude command allowed-tools: command markdown that uses the bash tool MUST declare `allowed-tools` that includes `Bash(...)`.
  - Dangerous defaults: command markdown that uses the bash tool MUST invoke mutating agentpack commands with `--json` and `--yes`.
  - Policy pack pinning (when configured): if `repo/agentpack.org.yaml` configures `policy_pack`, then `repo/agentpack.org.lock.json` MUST exist and MUST match the configured source (no network access).
  - Org distribution policy (when configured): if `repo/agentpack.org.yaml` configures `distribution_policy`, then `policy lint` MUST validate the required targets/modules in `repo/agentpack.yaml`.

Exit codes:
- Succeeds (exit 0) when no violations are found.
- Exits non-zero when at least one policy violation is found.

JSON mode:
- On success: `command="policy.lint"`, `ok=true`, and `data` contains `{root, root_posix, issues, summary}` (issues will be empty).
- On violations: `ok=false`, `errors[0].code="E_POLICY_VIOLATIONS"`, and `errors[0].details` contains `{root, root_posix, issues, summary}` (note: `data` is `{}` on failure).

### 4.20 `policy lock` (governance, mutating)

`agentpack policy lock`

Behavior:
- Reads `repo/agentpack.org.yaml` and resolves the configured `policy_pack.source`.
- Writes/updates `repo/agentpack.org.lock.json` to pin the policy pack (diff-friendly, deterministic ordering).

JSON mode:
- `policy lock --json` requires `--yes` (otherwise `E_CONFIRM_REQUIRED`).
- On success: `command="policy.lock"`, `ok=true`, and `data` includes `lockfile_path`, `resolved_version`, and `sha256`.

## 5. Target adapter details

Build-time target selection:
- Target adapters can be compiled selectively via Cargo features:
  - `target-codex`
  - `target-claude-code`
  - `target-cursor`
  - `target-vscode`
- Default builds include all built-in targets.
- `agentpack help --json` includes `data.targets[]` listing targets compiled into the running binary.
- Selecting a non-compiled target is treated as unsupported (`E_TARGET_UNSUPPORTED`).

### 5.1 `codex` target

Paths (follow Codex docs):
- `codex_home`: `~/.codex` (override via `CODEX_HOME`)
- user skills: `$CODEX_HOME/skills`
- repo skills: per Codex skill precedence:
  - `$CWD/.codex/skills`
  - `$CWD/../.codex/skills`
  - `$REPO_ROOT/.codex/skills`
- custom prompts: `$CODEX_HOME/prompts` (user scope only)
- global agents: `$CODEX_HOME/AGENTS.md`
- repo agents: `<repo>/AGENTS.md`

Deploy rules:
- skills: copy directories (no symlinks)
- prompts: copy `.md` files into the prompts directory
- instructions:
  - global: render base `AGENTS.md` into `$CODEX_HOME/AGENTS.md`
  - project: render into repo-root `AGENTS.md` (default)
  - (future) finer-grained subdir override

### 5.2 `claude_code` target (files mode)

Paths:
- repo commands: `<repo>/.claude/commands`
- user commands: `~/.claude/commands`
- repo skills (optional): `<repo>/.claude/skills`
- user skills (optional): `~/.claude/skills`

Deploy rules:
- command modules are single `.md` files; filename = slash command name
- skill modules are directories copied under the enabled skills root(s):
  - `<skills_root>/<skill_name>/...`
- if the body uses `!bash`/`!`bash``: the YAML frontmatter must declare `allowed-tools: Bash(...)`
- (future) plugin mode is possible (write `.claude-plugin/plugin.json`), but not implemented yet

### 5.3 `cursor` target (files mode)

Paths:
- project rules: `<project_root>/.cursor/rules` (project scope only)

Deploy rules:
- instructions:
  - for each enabled `instructions` module, write one Cursor rule file:
    - `<project_root>/.cursor/rules/<module_fs_key>.mdc`
  - each rule file includes YAML frontmatter (`description`, `globs`, `alwaysApply`) and the module’s `AGENTS.md` content.

Notes:
- `cursor` currently supports project scope only; `scope: user` is invalid.

### 5.4 `vscode` target (files mode)

Paths:
- project Copilot instructions: `<project_root>/.github/copilot-instructions.md` (project scope only)
- project prompt files: `<project_root>/.github/prompts/*.prompt.md`

Deploy rules:
- instructions:
  - collects enabled `instructions` modules into a single `copilot-instructions.md` file
  - when multiple modules exist, agentpack uses per-module section markers to preserve module attribution (same marker format as `codex` `AGENTS.md` aggregation)
- prompts:
  - copies each `prompt` module’s single `.md` file into `.github/prompts/`
  - if the source filename does not end with `.prompt.md`, agentpack writes it as `<name>.prompt.md` for VS Code discovery

Notes:
- `vscode` currently supports project scope only; `scope: user` is invalid.

## 6. JSON output spec

See: `JSON_API.md`.

All `--json` outputs must include:
- `schema_version: number`
- `ok: boolean`
- `command: string`
- `version: string` (agentpack version)
- `data: object` (empty object on failure)
- `warnings: [string]`
- `errors: [{code, message, details?}]`

Path field convention:
- Whenever a JSON payload contains filesystem paths (e.g. `path`, `root`, `repo`, `overlay_dir`, `lockfile`, ...), it should also provide a companion `*_posix` field using `/` separators.
- This is additive (no `schema_version` bump): original fields remain unchanged; automation should prefer parsing `*_posix` for cross-platform stability.

`plan --json` `data` example:

```json
{
  "profile": "work",
  "targets": ["codex", "claude_code"],
  "changes": [
    {
      "target": "codex",
      "op": "update",
      "path": "/home/user/.codex/skills/agentpack-operator/SKILL.md",
      "path_posix": "/home/user/.codex/skills/agentpack-operator/SKILL.md",
      "before_sha256": "...",
      "after_sha256": "...",
      "update_kind": "managed_update",
      "reason": "content differs"
    }
  ],
  "summary": {"create": 3, "update": 2, "delete": 0}
}
```

`status --json` `data` example:

```json
{
  "drift": [
    {
      "target": "codex",
      "path": "...",
      "path_posix": "...",
      "expected": "sha256:...",
      "actual": "sha256:...",
      "kind": "modified"
    }
  ]
}
```

## 7. Compatibility and limitations

- No symlinks by default (unless a future experimental `--link` flag is added).
- Do not execute third-party scripts.
- Prompts do not support repo scope (follow Codex docs); use a skill to share prompts.

## 8. References

(Same as `PRD.md`.)


---

# Appendix B：JSON API 合约（docs/JSON_API.md v0.6.0）

# JSON API (the `--json` output contract)

> Current as of **v0.6.0** (2026-01-15). `SPEC.md` is the semantic source of truth; this file focuses on the stable `--json` contract.

## 1) Stability guarantees (principles)

Agentpack’s `--json` output is treated as a programmable API:
- If you pass `--json`, **stdout is always valid JSON** (even on failure; `ok=false` in the envelope).
- `schema_version` is the envelope structure version; current value is `1`.
- For common, actionable failures, `errors[0].code` provides stable error codes (see `ERROR_CODES.md`).
- `warnings` are primarily for human diagnosis; do not rely on string matching for critical branching.

Compatibility policy (`schema_version = 1`):
- **Adding new fields is allowed** (additive; backward-compatible).
- **Removing/renaming fields is not allowed**, and semantics must not change without bumping `schema_version`.

## 2) Envelope shape (`schema_version=1`)

All `--json` outputs include:
- `schema_version`: number
- `ok`: boolean
- `command`: string
- `version`: string (agentpack version)
- `data`: object (success payload; empty object on failure)
- `warnings`: string[]
- `errors`: array[{code,message,details?}]

Failure example:
```json
{
  "schema_version": 1,
  "ok": false,
  "command": "deploy",
  "version": "0.6.0",
  "data": {},
  "warnings": [],
  "errors": [
    {
      "code": "E_CONFIRM_REQUIRED",
      "message": "refusing to run 'deploy --apply' in --json mode without --yes",
      "details": {"command": "deploy --apply"}
    }
  ]
}
```

## 3) Mutating guardrails in `--json` mode (must understand)

In `--json` mode, mutating commands require explicit `--yes`, otherwise they return `E_CONFIRM_REQUIRED`.

You can use:
- `agentpack help --json` to obtain the command list, the mutating command set, and the compiled target set (`data.targets[]`)

Common mutating commands (not exhaustive):
- `deploy --apply`, `update`, `lock`, `fetch`, `add/remove`, `bootstrap`, `rollback`
- `overlay edit/rebase`, `doctor --fix`
- `record`, `evolve propose/restore`

## 4) Path field conventions (cross-platform)

To avoid Windows `\` vs POSIX `/` differences forcing heavy branching in automation:
- When a payload includes filesystem paths in `data`, many payloads also include a companion `*_posix` field.
- `*_posix` uses `/` separators and is suitable for cross-platform parsing; the original field remains OS-native for convenience.

Examples: `path` + `path_posix`, `repo` + `repo_posix`, `overlay_dir` + `overlay_dir_posix`.

## 5) Common command payloads (high-level)

Below are the most commonly consumed commands in automation. Field lists focus on stable/high-frequency fields.

### plan

`command = "plan"`

`data`:
- `profile: string`
- `targets: string[]`
- `changes: PlanChange[]`
- `summary: {create, update, delete}`

`PlanChange` fields:
- `target, op(create|update|delete), path, path_posix`
- `before_sha256?, after_sha256?`
- `update_kind? (managed_update|adopt_update)`
- `reason`

### preview

`command = "preview"`

`data`:
- `profile, targets`
- `plan: {changes, summary}`
- Optional: `diff: {changes, summary, files}` (only when `preview --diff --json`)

`diff.files[]`:
- `target, root, root_posix, path, path_posix, op`
- `before_hash?, after_hash?`
- `unified?` (text diff; omitted for large or binary/non-utf8 files with warnings)

### deploy

`command = "deploy"`

`data`:
- `applied: boolean`
- `profile, targets`
- `changes, summary`
- When `applied` is true: `snapshot_id`

Tip:
- If the plan contains `adopt_update`, you must pass `--adopt` or the command returns `E_ADOPT_CONFIRM_REQUIRED` (details include `sample_paths`).

### status

`command = "status"`

`data`:
- `profile, targets`
- `drift: DriftItem[]`
- `summary: {modified, missing, extra}` (additive)
- `summary_total?: {modified, missing, extra}` (additive; present when `status --only` is used)
- `next_actions?: string[]` (additive; suggested follow-up commands)

`DriftItem`:
- `target, path, path_posix`
- Optional: `root, root_posix` (additive; target root that contains `path`)
- `expected? (sha256:...)`
- `actual? (sha256:...)`
- `kind: missing|modified|extra`

### doctor

`command = "doctor"`

`data`:
- `machine_id: string`
- `roots: array[{target, root, root_posix, exists, writable, scan_extras, issues, suggestion?}]`
- `gitignore_fixes: array[{repo_root, repo_root_posix, gitignore_path, gitignore_path_posix, updated}]` (when `doctor --fix` is used)
- `next_actions?: string[]` (additive; suggested follow-up commands)

### overlay.path

`command = "overlay.path"`

`data`:
- `module_id, scope`
- `overlay_dir, overlay_dir_posix`

### evolve.propose (dry-run)

`command = "evolve.propose"`

`data` (when dry-run):
- `created: false`
- `reason: "dry_run"`
- `candidates: [{module_id,target,path,path_posix}]`
- `skipped: [{reason,target,path,path_posix,module_id?,module_ids?,suggestions?}]` (additive)
- `summary: {drifted_proposeable, drifted_skipped, ...}`

`suggestions` (additive):
- `[{action, reason}]`

After execution (non dry-run):
- `created: true`
- `branch, scope, files, files_posix, committed`

## 6) Unstable/fallback code: E_UNEXPECTED

When an error is not classified as a stable UserError, agentpack uses:
- `E_UNEXPECTED`

Do not branch critical automation logic on this; treat it as a “needs human attention” fallback.


---

# Appendix C：错误码（docs/ERROR_CODES.md v0.6.0）

# ERROR_CODES.md (stable error code registry)

> Current as of **v0.6.0** (2026-01-15). `SPEC.md` is the semantic source of truth; this file is the stable registry for `--json` automation (`errors[0].code`).

This file defines stable, externally-consumable error codes for `--json` mode (`errors[0].code`).

Conventions:
- When `ok=false`, the process exit code is non-zero.
- `errors[0].code` is for automation branching; `errors[0].message` is primarily for humans (may be refined over time).
- Do not branch critically on `warnings` (strings are not stable).

## Stable error codes

### E_CONFIRM_REQUIRED
Meaning: in `--json` mode, the command would perform a mutation (filesystem and/or git), but `--yes` is missing.
Typical cases: `deploy --apply --json`, `update --json`, `overlay edit --json`, etc.
Retryable: yes.
Recommended action: confirm you intend to write, then retry with `--yes`, or drop `--json` and use interactive confirmation.
Details: usually includes `{"command": "..."}`.

### E_ADOPT_CONFIRM_REQUIRED
Meaning: `deploy --apply` would overwrite an existing unmanaged file (`adopt_update`), but `--adopt` was not provided.
Retryable: yes.
Recommended action:
- Run `preview --diff` to confirm scope/impact.
- If you truly want to take over and overwrite, retry with `--adopt`.
Details: includes `{flag, adopt_updates, sample_paths}`.

### E_CONFIG_MISSING
Meaning: missing `repo/agentpack.yaml`.
Retryable: yes.
Recommended action: run `agentpack init` to create a skeleton, or point to the correct repo via `--repo`.
Details: typically includes `{path, hint}`.

### E_CONFIG_INVALID
Meaning: `agentpack.yaml` is syntactically or semantically invalid.
Retryable: depends on fixing config.
Recommended action: fix YAML based on `details` and/or error message (e.g., missing default profile, duplicate module id, invalid source, missing target config).

This code MAY also be used when a configured module is structurally invalid (e.g., a `skill` module’s `SKILL.md` has missing/invalid YAML frontmatter).

### E_CONFIG_UNSUPPORTED_VERSION
Meaning: `agentpack.yaml` `version` is unsupported.
Retryable: depends on fixing config or upgrading agentpack.
Recommended action: set `version` to a supported value (currently `1`) or upgrade agentpack.
Details: typically includes `{version, supported}`.

### E_LOCKFILE_MISSING
Meaning: missing `repo/agentpack.lock.json` but the command requires it (e.g., `fetch`).
Retryable: yes.
Recommended action: run `agentpack lock` or `agentpack update`.

### E_LOCKFILE_INVALID
Meaning: `agentpack.lock.json` is invalid JSON or cannot be parsed.
Retryable: depends on repair/rebuild.
Recommended action: fix JSON or delete it and regenerate via `agentpack update`.

### E_LOCKFILE_UNSUPPORTED_VERSION
Meaning: `agentpack.lock.json` `version` is unsupported.
Retryable: depends on upgrading agentpack or regenerating lockfile.
Recommended action: upgrade agentpack, or regenerate the lockfile via `agentpack lock` / `agentpack update`.
Details: typically includes `{version, supported}`.

### E_TARGET_UNSUPPORTED
Meaning:
- `--target` specifies an unsupported value, or
- The manifest config contains an unknown target.
- The target is not compiled into the running agentpack binary (feature-gated builds).
Retryable: yes.
Recommended action:
- `--target` must be `all|codex|claude_code|cursor|vscode` (but feature-gated builds may support a subset; see `agentpack help --json` `data.targets[]`).
- Manifest targets must be built-in targets that are compiled into the running binary.

### E_DESIRED_STATE_CONFLICT
Meaning: multiple modules produced different content for the same `(target, path)`. Agentpack refuses to silently overwrite.
Retryable: depends on config/overlay fixes.
Recommended action: adjust modules/overlays so only one module produces that path, or make the contents identical.
Details: includes both sides’ sha256 and module_ids.

### E_OVERLAY_NOT_FOUND
Meaning: requested overlay directory does not exist.
Retryable: yes.
Recommended action: run `agentpack overlay edit <module_id>` to create the overlay.

### E_OVERLAY_BASELINE_MISSING
Meaning: overlay metadata is missing (`<overlay_dir>/.agentpack/baseline.json`), so rebase cannot proceed.
Retryable: yes.
Recommended action: re-run `agentpack overlay edit <module_id>` to regenerate metadata.

### E_OVERLAY_BASELINE_UNSUPPORTED
Meaning: overlay baseline cannot locate a merge base, so rebase cannot proceed safely.
Retryable: depends on baseline repair.
Recommended action: usually recreate the overlay (new baseline), or ensure upstream is traceable (git) and recreate.

### E_OVERLAY_REBASE_CONFLICT
Meaning: `overlay rebase` produced conflicts that cannot be auto-merged.
Retryable: yes (after resolving conflicts).
Recommended action: open the conflict-marked files under the overlay directory (for patch overlays: `.agentpack/conflicts/<relpath>`), resolve, then re-run `agentpack overlay rebase` (or commit overlay changes directly).
Details: includes `{conflicts, summary, overlay_dir, scope, ...}`.

### E_OVERLAY_PATCH_APPLY_FAILED
Meaning: patch overlay application failed during desired-state generation (the patch could not be applied cleanly).
Retryable: yes (after regenerating/fixing the patch).
Recommended action:
- regenerate the patch against current upstream (or lower overlays) content, or
- switch to a directory overlay for that file.
Details: includes `{module_id, scope, overlay_dir, patch_file, relpath, stderr, ...}`.

### E_POLICY_VIOLATIONS
Meaning: `policy lint` detected one or more governance policy violations.
Retryable: yes (after fixing the violations).
Recommended action:
- Run `agentpack policy lint --json` to get machine-readable issues (suitable for CI gating).
- Fix the reported issues and rerun until `ok=true`.
Details: includes `{root, root_posix, issues, summary}` where `issues[]` items include `{rule, path, path_posix, message, details?}`.

### E_POLICY_CONFIG_MISSING
Meaning: missing `repo/agentpack.org.yaml` when running governance policy commands that require it (e.g., `agentpack policy lock`).
Retryable: yes.
Recommended action: create `repo/agentpack.org.yaml` (governance is opt-in) and retry.
Details: includes `{path, hint}`.

### E_POLICY_CONFIG_INVALID
Meaning: `repo/agentpack.org.yaml` is syntactically or semantically invalid (e.g., invalid YAML, missing/empty `policy_pack.source`, unsupported `policy_pack.source` syntax).
Retryable: depends on fixing config.
Recommended action: fix YAML based on `details` and retry.
Details: includes `{path, error?}` and MAY include `{field, value, hint}`.

### E_POLICY_CONFIG_UNSUPPORTED_VERSION
Meaning: `repo/agentpack.org.yaml` `version` is unsupported.
Retryable: depends on upgrading agentpack or fixing config.
Recommended action: set `version` to a supported value (currently `1`) or upgrade agentpack.
Details: includes `{path, version, supported}`.

## Non-stable / fallback error codes

### E_UNEXPECTED
Meaning: unexpected failure that was not classified as a stable UserError.
Retryable: unknown.
Recommended action:
- Save `errors[0].message` plus surrounding context (stdout/stderr).
- Retry with a smaller repro.
- For automation: typically “escalate to human” or fail-fast, rather than branching on message text.


---

# Appendix D：CLI reference（docs/CLI.md v0.6.0）

# CLI reference

> Language: English | [Chinese (Simplified)](zh-CN/CLI.md)

This document is for quickly looking up how a command works. For workflow-oriented guidance, see `WORKFLOWS.md`.

## Global flags (supported by all commands)

- `--repo <path>`: path to the config repo (default: `$AGENTPACK_HOME/repo`)
- `--profile <name>`: profile name (default: `default`)
- `--target <codex|claude_code|cursor|vscode|all>`: target selection (default: `all`)
- `--machine <id>`: override machine id (for machine overlays; default: auto-detect)
- `--json`: machine-readable JSON output (envelope) on stdout
- `--yes`: skip confirmations (note: in `--json` mode, mutating commands require explicit `--yes`)
- `--dry-run`: force dry-run behavior (even if `deploy --apply` / `overlay rebase` etc. are requested)

Tips:
- `agentpack help --json` returns a structured command list and the mutating command set.
- `agentpack schema --json` describes the JSON envelope and common `data` payload shapes.

## init

`agentpack init [--git] [--bootstrap]`
- Initializes a config repo skeleton (creates `agentpack.yaml` and example directories)
- By default it does not run `git init`
- `--git`: also initializes the repo directory as a git repo and ensures `.gitignore` ignores `.agentpack.manifest.json`
- `--bootstrap`: also installs operator assets into the config repo (equivalent to `agentpack bootstrap --scope project`)

## add / remove

- `agentpack add <instructions|skill|prompt|command> <source> [--id <id>] [--tags a,b] [--targets codex,claude_code,cursor,vscode]`
- `agentpack remove <module_id>`

Source spec:
- `local:<path>` (repo-relative path)
- `git:<url>#ref=<ref>&subdir=<path>`

Examples:
- `agentpack add instructions local:modules/instructions/base --id instructions:base --tags base`
- `agentpack add skill git:https://github.com/your-org/agentpack-modules.git#ref=v1.2.0&subdir=skills/git-review --id skill:git-review --tags work`

## lock / fetch / update

- `agentpack lock`: generate/update `agentpack.lock.json`
- `agentpack fetch`: fetch external sources into cache/store per lockfile
- `agentpack update`: composite command
  - Default: run lock+fetch when lockfile is missing; otherwise fetch-only
  - Flags: `--lock`/`--fetch`/`--no-lock`/`--no-fetch`

## preview / plan / diff

- `agentpack plan`: show create/update/delete without applying
- `agentpack diff`: show diffs for the current plan
- `agentpack preview [--diff]`: composite command (always runs plan; also runs diff when `--diff` is set)

Notes:
- Updates in a plan can be one of:
  - `managed_update`: updating a managed file
  - `adopt_update`: overwriting an existing unmanaged file (refused by default; see `deploy --adopt`)

## deploy

`agentpack deploy [--apply] [--adopt]`

- Without `--apply`: show plan + diff only
- With `--apply`: write to target roots, create a snapshot, and update per-root `.agentpack.manifest.json`
- If the plan contains `adopt_update`: you must pass `--adopt` or the command fails with `E_ADOPT_CONFIRM_REQUIRED`

Common:
- `agentpack deploy --apply`
- `agentpack --json deploy --apply --yes`
- `agentpack deploy --apply --adopt`

## status

`agentpack status [--only <missing|modified|extra>[,...]]`
- Detects drift (missing/modified/extra) using `.agentpack.manifest.json`
- If no manifests exist (first run or migration), it falls back to “desired vs FS” and emits a warning
- `--only`: filter the reported drift list to a subset of kinds (repeatable or comma-separated)

## tui (optional)

`agentpack tui [--adopt]`

- Feature-gated: build with `--features tui` to enable the command.
- Does not support `--json` output; fails with `E_CONFIG_INVALID` when `--json` is passed.
- Interactive UI for browsing `plan` / `diff` / `status`.
- `a`: triggers apply with an explicit confirmation prompt (equivalent to `deploy --apply` for the current `--profile` / `--target`).
- `--adopt`: allow overwriting existing unmanaged files (adopt updates), same semantics as `deploy --adopt`.

See `TUI.md` for key bindings.

## rollback

`agentpack rollback --to <snapshot_id>`
- Roll back to a deployment/bootstrap snapshot

## doctor

`agentpack doctor [--fix]`
- Checks machine id, target path writability, and common config issues
- `--fix`: idempotently appends `.agentpack.manifest.json` to `.gitignore` for detected git repos (avoid accidental commits)

## remote / sync

- `agentpack remote set <url> [--name origin]`: configure a git remote for the config repo
- `agentpack sync [--rebase] [--remote origin]`: recommended pull/rebase + push sync flow

## bootstrap

`agentpack bootstrap [--scope user|project|both]`
- Installs operator assets:
  - Codex: operator skill
  - Claude Code: `/ap-*` commands

Tip: choose targets via global `--target`:
- `agentpack --target codex bootstrap --scope both`

## overlay

- `agentpack overlay edit <module_id> [--scope global|machine|project] [--sparse|--materialize]`
- `agentpack overlay rebase <module_id> [--scope ...] [--sparsify]` (3-way merge; supports `--dry-run`)
- `agentpack overlay path <module_id> [--scope ...]`

## explain

`agentpack explain plan|diff|status`
- Explains which module and which overlay layer (upstream/global/machine/project) produced a change/drift item.

## record / score

- `agentpack record`: read JSON from stdin and append to `state/logs/events.jsonl`
- `agentpack score`: aggregate events into success/failure stats (skips malformed lines; emits warnings)

## evolve

- `agentpack evolve propose [--module-id <id>] [--scope global|machine|project] [--branch <name>]`
  - Capture drifted deployed content and generate an overlay proposal (creates a branch and writes files)
  - Recommended to start with `--dry-run --json` to inspect candidates
- `agentpack evolve restore [--module-id <id>]`
  - Restore missing desired outputs (create-only; supports `--dry-run`)

## completions

`agentpack completions <shell>`
- Generate shell completion scripts (bash/zsh/fish/powershell, etc.)


---

# Appendix E：Target conformance（docs/TARGET_CONFORMANCE.md v0.6.0）

# TARGET_CONFORMANCE.md

Conformance tests are the quality bar for targets.

## Required semantics
1. Delete protection: plan/apply only delete manifest-managed paths.
2. Manifest: apply writes per-root `.agentpack.manifest.json`.
3. Drift: status distinguishes `missing`/`modified`/`extra` (extras are not auto-deleted).
4. Rollback: restores create/update/delete effects.
5. JSON contract: envelope fields and key error codes remain stable.

## Recommended harness approach
- Use temp directories as fake target roots.
- Run the real pipeline (`deploy --apply`, `status`, `rollback`) against those roots.
- Keep tests hermetic: avoid writing to real `~/.codex` or `~/.claude`.
