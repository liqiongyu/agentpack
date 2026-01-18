# Agentpack — Unified Spec + Epics + Backlog (Codex-ready)

> 注意：本文档是以 **v0.6.0（2026-01-15）** 为基线生成的规划快照，可能与当前 `main` 分支实现存在偏差。
> 当前实现级契约请以 `docs/SPEC.md` 为准；`--json` 合约请以 `docs/JSON_API.md` / `docs/ERROR_CODES.md` 为准；需要走提案流程的变更请使用 `openspec/`。

> 这份文档的目标是：把 Agentpack 的 **实现级 Spec**、**迭代 Epic**、**可执行 Backlog** 合并成一份“可直接喂给 Codex 开发”的 Markdown。
> 重点是 **一致性无冲突** + **细粒度** + **每个 Backlog 条目都能对应一个小 PR**。

- **文档日期**：2026-01-14
- **基线版本**：Agentpack v0.6.0（以本仓库当前代码 + `docs/SPEC.md` 为准）
- **适用读者**：维护者 / 贡献者 / 通过 Codex（或其它 coding agent）协作开发的人

---

## 0. 使用方式（给 Codex 的执行规则）

当你要实现某个 Backlog item（无论你是人还是 Codex）：

1) 先读相关的 Spec（见附录 A：`docs/SPEC.md`）。
2) 变更要 **小**、**可审查**（一个 Backlog item = 一个 PR）。
3) 任何 CLI 行为变化都必须：
   - 增补/更新测试（`tests/`），把行为锁住
   - 如果涉及 `--json` 输出：**只允许向后兼容的字段增加**（additive-only），除非明确要 bump `schema_version`（极少见）
4) 任何稳定契约 / 文件格式变化都必须：
   - 走 OpenSpec：在 `openspec/changes/<change-id>/` 写 proposal
   - `openspec validate <change-id> --strict --no-interactive`
5) 本地必跑：
   - `cargo fmt --all -- --check`
   - `cargo clippy --all-targets --all-features -- -D warnings`
   - `cargo test --all --locked`

每个 PR 的 Done 标准（DoD）：
- 编译/测试全绿
- 新行为有测试覆盖
- `docs/SPEC.md` 不撒谎（必要时同步更新）
- 不破坏 JSON envelope / stable error codes（除非该 Epic 明确允许且做了版本化）


## 1. 顶层动机 / 愿景（从用户视角 + AI 原生软件视角）

Agentpack 的核心定位：**AI-coding 资产的“本地控制面（local control plane）”**。

它解决的不是“写代码”，而是“让你在多工具、多机器、多项目里，持续地把 AI-coding 相关资产部署到正确的位置，并且可审计、可回滚、可复现”。

这里的“资产”主要包括：
- 指令（`AGENTS.md`）
- Codex Agent Skills（`SKILL.md`）
- Codex custom prompts（`~/.codex/prompts`）
- Claude Code slash commands（`.claude/commands`）
- 以及其它目标（Cursor rules、VS Code copilot instructions/prompts）

Agentpack 的价值前提（非常关键）：
- 用户确实是“多工具重度 AI coding 用户”，并且愿意把自己的 AI 工作流当成工程资产来管理（而不是随手复制粘贴）。
- 用户愿意接受一个“安全/可审计/可回滚”的流程：`plan -> diff -> apply -> snapshot -> rollback`。
- 用户愿意让“配置 repo（agentpack.yaml + modules + overlays）”成为单一事实源（SSOT）。

非目标用户（明确不迎合）：
- 更偏好 `git + dotfiles 管理器（chezmoi/yadm/stow）+ 少量脚本` 的用户（他们会觉得 Agentpack 过重）。
- 只用单一工具、单一机器、不在意回滚/审计的人（Agentpack 的收益不明显）。

---

## 2. 设计原则（必须长期保持稳定）

这些原则决定了后续所有 Epic 的边界，避免“越做越像 dotfiles 工具 / 越做越像通用包管理器”。

A) 稳定契约优先
- `--json` 输出是 API：stdout 永远是合法 JSON；`schema_version=1` 下只做向后兼容的字段增加。
- 稳定错误码（`E_*`）优先于字符串匹配。

B) 安全与可回滚优先
- 默认不做破坏性写入；任何 apply/mutate 行为可预览、有确认（`--yes`），并能 snapshot/rollback。
- 只管理自己写出去的文件（依靠 `.agentpack.manifest.<target>.json` 做边界，兼容 legacy `.agentpack.manifest.json`），不去“清理用户文件”。

C) 目标生态变化很快，但内部抽象要稳
- TargetAdapter 是稳定抽象；生态变化（Codex/Claude/Cursor/VS Code 规则变）只需要改 adapter + conformance tests。
- 适配流程（OpenSpec + conformance + golden tests）要足够顺滑，才能跟上变化速度。

D) 个人重度用户体验优先；组织治理是长期目标但必须隔离
- 在组织/团队功能没有成熟之前：个人体验不能被“企业治理”的约束污染（配置、交互、默认行为都不能被强行改变）。
- 组织治理能力必须是**显式 opt-in**（例如独立子命令/独立 feature/独立配置文件），并且可被彻底忽略。


## 3. 交互面：CLI / Skills / MCP（为什么要都保留）

为了满足“AI-first + 人类可控 + 自动化可编排”，Agentpack 需要同时拥有 3 个层次的交互面：

1) CLI（核心）
- 优点：可脚本化、可审计、最少依赖、跨工具通用（任何 agent 都能跑 bash/cli）。
- 角色：**唯一的最终真相执行器**（plan/diff/deploy/rollback 都以 CLI 语义为准）。
- 约束：CLI 仍然要把 `--json` 作为稳定 API 输出。

2) Operator assets（Skills / Slash Commands）
- 形态：Codex skill、Claude Code slash commands。
- 优点：大幅降低“AI 直接上手”的门槛（agent 不需要理解全部 CLI 细节；只要调用约定好的 skill/command）。
- 角色：把“最佳实践流程”打包成更短、更确定的操作入口（例如 `/ap-preview`、`agentpack-operator` skill）。
- 约束：这些资产必须被 Agentpack 自己部署/更新（bootstrap），并且版本可追踪（status 会提示过期）。

3) MCP server（结构化工具层，面向 agent runtime）
- 优点：对接越来越多的 agent host（Codex、VS Code、Claude Agent SDK 等）时，MCP 是“标准化的 tool registry”，比纯 CLI 更结构化。
- 角色：让 Agentpack 能作为“工具服务器”被消费：工具列表、参数 schema、返回 JSON 都是强约束。
- 约束：安全风险更高（MCP server 运行在高权限本地环境），必须把“可变更操作”做成显式批准（approval）语义，并默认最小权限。

结论（建议的策略）：
- **短期**：CLI + operator assets 继续作为主线，快速提升个人重度用户体验。
- **中期**：引入 MCP server，但内部实现尽量复用现有引擎/CLI JSON contract（不要造第二套语义）。
- **长期**：当 org/governance 成熟后，MCP server 才可能成为“组织级自动化”的更强入口；但仍然不能替代 CLI 的可审计性。


## 4. Spec（实现级契约）如何在本计划中使用

- 本仓库的 **实现级唯一权威 Spec** 是：附录 A（`docs/SPEC.md` 的完整内容）。
- 本文档的 Epic/Backlog 在提出新能力时，会标注“需要更新 Spec 的哪些章节”，但在代码未合入前，都视为“提案/草案”。

对 Codex 的要求：
- 实现任何 Backlog 前，先在 Spec 里找到“相关契约”（例如 overlay、JSON envelope、error codes）。
- 如果 Backlog 要求改契约：必须走 OpenSpec proposal + 更新 `docs/SPEC.md` + 更新 tests。


## 5. Roadmap 总览（里程碑 + Epic）

这里的里程碑不是“日期承诺”，而是“依赖顺序 + 优先级”。

### M0（v0.6 方向）：个人重度用户体验拉满（不引入破坏性新概念）
目标：
- 更“顺滑”的日常循环：`update -> preview -> deploy -> status -> evolve propose -> rollback`
- 更强的可观测性（status/doctor/evolve 输出更可执行）
- 文档与测试消除漂移（让新贡献者不会迷路）

### M1（v0.7 方向）：覆盖“资产定制”的硬需求
目标：
- 让 overlays 更适合小改动（patch overlays）
- 提供一个轻量的 TUI（可选）提升沉浸式体验，但不牺牲 CLI 可编排性

### M2（v0.8 方向）：生态对接 + 可扩展性
目标：
- TargetAdapter 模块化（减少生态变动时的维护成本）
- MCP server 作为结构化工具入口（面向 agent host）

### M3（v1+ 长期）：组织/团队治理（显式 opt-in，不影响个人体验）
目标：
- 把“AI coding 规范 / 合规检查 / 资产分发”做成工程治理的一部分（CI/Policy）
- 但必须以“隔离的治理层”实现：个人用户不需要也不被强制依赖


## 6. Epics + Backlog（细粒度，可直接开 PR）

说明：
- **优先级**：P0=必须做/阻塞后续；P1=强烈建议；P2=可选（但有明显价值）；P3=探索性。
- **每个条目都是“一个 PR 的大小”**（如果写着“可能需要拆分”，就先拆成多个条目）。
- **所有条目默认不引入破坏性变更**，除非条目里明确写了“breaking”。

---

### Epic M0-DOC：消除文档/实现漂移，提升新用户上手（P1）

#### M0-DOC-1（P1）修正文档中的 target 列表（与 v0.6.0 实现一致）
动机：目前仓库里不同文档对 `--target` 支持集合的描述存在漂移，容易误导自动化脚本/新用户。

范围（in）：
- 更新 `docs/ERROR_CODES.md` 中 `E_TARGET_UNSUPPORTED` 的说明与示例：支持 `codex|claude_code|cursor|vscode|all`
- 扫描 `docs/JSON_API.md`、`docs/BACKLOG.md`、`README*`、`docs/CLI*.md` 是否存在旧示例，一并改正

范围（out）：
- 不改变任何 CLI 行为，只做文档修正

验收标准：
- `rg "claude_code\)" docs/ERROR_CODES.md` 不再出现“只支持 codex/claude_code”的描述
- CI 全绿（文档修改不应影响代码）

建议改动位置：
- `docs/ERROR_CODES.md`
- `docs/BACKLOG.md`（把“新增 Cursor/VS Code target”改成“新增更多 target（JetBrains/Zed/…）”）

---

#### M0-DOC-2（P1）补齐“个人重度用户”的 Quickstart（多工具、多 scope）
动机：当前文档很多是参考手册式，缺一个“从 0 到日常循环”的路线图。

范围（in）：
- 新增 `docs/QUICKSTART.md`（英文）与 `docs/zh-CN/QUICKSTART.md`（中文）至少包含：
  - 典型 3 套组合：
    a) Codex(user) + Claude(project)
    b) Codex(user+project)
    c) VS Code(project) + Cursor(project)
  - 最短路径命令序列：`init -> add -> update -> preview --diff -> deploy --apply -> status -> rollback`
  - 解释 overlays/machines 的使用场景（多机器差异）

范围（out）：
- 不做新功能开发，只补文档与示例

验收标准：
- README 里有 Quickstart 链接
- Quickstart 中所有命令在 v0.6.0 上可跑通（不要写未来命令）

---

#### M0-DOC-3（P1）为“OpenSpec 改动流程”补一页图解（降低贡献门槛）
动机：项目是 spec-driven 的，但很多贡献者不知道什么时候需要 OpenSpec。

范围（in）：
- 新增 `docs/CONTRIBUTING_SPECS.md`：
  - 什么时候需要提 OpenSpec（文件格式/稳定 JSON/错误码/命令语义）
  - 最小流程（create proposal -> validate -> implement -> archive）
  - 常见坑（只改 docs/SPEC 但忘了 openspec/specs 同步；或反过来）

验收标准：
- 在 `CONTRIBUTING.md` 里加链接
- 文档内容与 `.codex/skills/agentpack-dev/SKILL.md` 一致不冲突

---

### Epic M0-UX：把“日常循环”做得更顺滑（P1）

> 说明：v0.6.0 已经有 `status.summary` / `status.next_actions` 等能力。本 Epic 的目标是“更好用、更少噪音、更可编排”，而不是重复造轮子。

#### M0-UX-1（P1）Status 输出的“建议动作”去重/排序稳定化（human + json）
动机：status 的 next_actions 是高价值入口；需要稳定排序与更少的误导建议。

范围（in）：
- 保证 next_actions 在 human 输出与 json 输出都有稳定排序规则（例如按重要性 + 字典序）
- 只在确实相关时给出建议（例如未部署过就提示 bootstrap/deploy；有 drift 才提示 evolve propose）

验收标准：
- 更新/新增测试：`tests/cli_json_golden_core_commands.rs` 或新增专门测试，锁定 next_actions 顺序
- `--json` 只做 additive（字段不变，仅值更稳定）

---

#### M0-UX-2（P1）Doctor 的“下一步建议”对齐 Status 的 next_actions 体系
动机：doctor 是新手第一入口，应该输出可执行的 next step，而不是仅打印警告。

范围（in）：
- 在 `doctor --json` 的 data 中加入可选字段 `next_actions`（additive-only）
- human 输出也增加 “Next actions” 段落（当存在可修复项时）

验收标准：
- 新增 golden snapshot：`tests/golden/doctor_json_data.json` 更新（或新增 doctor-only golden）
- Spec（附录 A）中 `doctor` 的 JSON payload 说明保持一致（若需补充则更新 `docs/JSON_API.md`）

---

#### M0-UX-3（P2）增加 `status --only <missing|modified|extra>` 过滤（面向重度用户）
动机：重度用户 drift 很多时，需要快速聚焦。

范围（in）：
- 新增 flag：`agentpack status --only missing`（可多次传参或逗号分隔，二选一）
- `--json` 输出仍是 drift 数组，但仅包含过滤后项；summary 反映过滤后或原始？（建议：summary 反映过滤后，并额外提供 `summary_total`（additive）保留总数）

验收标准：
- CLI reference 更新（`docs/CLI.md` + zh-CN）
- 新增测试：过滤行为 + json summary 字段

非目标：
- 不做复杂查询语法（例如 path glob），保持简单

---

### Epic M0-OPS：Operator 资产（skills / slash commands）持续可用（P1）

#### M0-OPS-1（P1）Codex operator skill 的 frontmatter/结构校验增强
动机：skill 是 AI-first 上手的关键；格式错误会导致“看起来部署了但不可用”。

范围（in）：
- 在 deploy/doctor 或 validate 流程中增加对 `SKILL.md` frontmatter 的更严格校验（但错误信息要对用户友好）
- 明确“缺字段/字段类型错”的错误码策略：是否属于稳定错误码？（如果进入 stable error codes，需要更新 registry）

验收标准：
- 增加/更新测试（可在 `tests/cli_status_operator_assets.rs` 或新增）
- 文档：在 `docs/SPEC.md` 的 operator assets 部分描述清楚校验规则

---

#### M0-OPS-2（P1）Claude slash commands 的 allowed-tools 最小集一致化 + 文档化
动机：Claude Code 对 allowed-tools 有强约束；需要模板与实际输出一致。

范围（in）：
- 审计 `templates/claude/commands/*.md`，确保所有命令都声明了合理的 allowed-tools
- 在 `docs/BOOTSTRAP.md`（如不存在则新增）明确说明 allowed-tools 的设计原则

验收标准：
- `agentpack bootstrap --scope user|project` 产物在本地 smoke test（可在 tests 中验证 frontmatter 存在）
- 不引入新的运行时依赖

---

#### M0-OPS-3（P2）新增“Claude Agent Skill”（可选）
动机：如果 Claude Agent SDK / Claude Desktop 更偏好 skill/agent 的形式，这能降低跨工具心智负担。

范围（in）：
- 新增 module type 或模板（优先模板）用于生成 Claude Agent Skill 资产
- bootstrap 能一键安装（user/project）

约束：
- 必须 opt-in（不影响只用 slash commands 的用户）
- 不改变现有 `claude_code` target 的默认输出

验收标准：
- conformance test 增加一个最小 smoke（如果该能力算 target 输出的一部分）

---

### Epic P2-1：Patch-based overlays（可选，Issue #180）（P2）

目标：为“只改几行”这种 overlay 场景减少 churn，并让冲突更可读。必须完全向后兼容现有目录 overlay。

#### P2-1-0（P1）OpenSpec：定义 patch overlay 格式与优先级
范围（in）：
- OpenSpec proposal：overlay 元数据引入 `overlay_kind: "dir"|"patch"`（或等价字段）
- patch 存储路径与命名规则（例如 `<overlay_dir>/.agentpack/patches/<relpath>.patch`）
- precedence 规则（patch 与 dir overlay 同时存在如何处理；建议：同一 overlay 只能一种 kind）
- 非 UTF-8 / 二进制文件策略（禁止 or fallback）

验收标准：
- `openspec validate ... --strict` 通过
- `docs/SPEC.md` 更新对应章节（overlay + merge/rebase）

---

#### P2-1-1（P2）实现 patch overlay 的“应用”（deploy/render 阶段）
范围（in）：
- desired state 生成时：如果 overlay_kind=patch，则从 upstream/base 读取原文 + 应用 patch 得到最终 bytes
- 失败时给出可理解的错误（必要时引入新的 stable error code）

验收标准：
- 新增测试：happy path（patch 生效）
- 对现有 overlay 不产生行为变化（回归测试）

---

#### P2-1-2（P2）实现 patch overlay 的 rebase（冲突稳定化）
范围（in）：
- `overlay rebase` 支持 patch overlay：更新 patch base，并在冲突时产生稳定的冲突输出/错误码
- 冲突文件/冲突信息要可定位（path 列表 + 建议动作）

验收标准：
- 新增测试：conflict path（产生稳定 error code 与 details）
- conformance harness（如需要）更新

---

#### P2-1-3（P2）`overlay edit` 增加 `--kind patch`（或等价 flag）
动机：让用户能显式创建 patch overlay，而不是手工摆放文件。

验收标准：
- `overlay edit --kind patch` 会创建 overlay_dir + baseline/metadata + patches 目录
- 文档与 `overlay path` 语义一致

---

### Epic P2-2：Lightweight TUI（可选，Issue #181）（P2）

目标：提供“沉浸式但不臃肿”的 TUI，用于快速浏览 plan/diff/status，并支持一键触发 apply（仍然遵守确认语义）。

#### P2-2-0（P1）设计决策：命令名 + feature gate + 依赖策略
建议：
- `agentpack tui` 子命令
- Cargo feature：`tui`（默认关闭或默认打开？建议默认关闭，避免依赖膨胀）

验收标准：
- 写一页短设计文档（可放 `docs/TUI.md`）
- 说明如何在 CI/发行版中处理 feature

---

#### P2-2-1（P2）实现只读 TUI（plan/diff/status 三屏）
范围（in）：
- 读取现有引擎输出（不要复制业务逻辑）
- 展示：summary、变更列表、diff 预览（大文件/二进制遵循现有 diff 规则）

验收标准：
- 至少一个 integration test（启动 TUI 的核心逻辑可测试；UI 细节不必过测）
- 文档：如何退出、按键说明

---

#### P2-2-2（P2）实现“安全 apply”（仍然需要确认）
范围（in）：
- 在 TUI 里触发 `deploy --apply` 必须显式确认（类似 `--yes` 语义但通过 UI）
- 失败时展示 stable error code 与建议动作

验收标准：
- 不允许静默写入
- 与 CLI 的 confirm 语义一致

---

### Epic P2-3：TargetAdapter 模块化（Issue #182）（P2）

目标：把 target 适配成本降到最低，并为未来新增 targets 做好隔离（同时不牺牲稳定性）。

#### P2-3-0（P1）OpenSpec/Docs：定义 target feature/注册策略
范围（in）：
- 明确 feature naming：`target-codex` / `target-claude-code` / `target-cursor` / `target-vscode`
- 明确默认 feature 集合
- `help --json` / `schema` 如何反映“当前构建包含哪些 targets”

验收标准：
- OpenSpec proposal + `docs/SPEC.md` 更新（target 列表与帮助输出）

---

#### P2-3-1（P2）代码重构：target adapters 按 feature 条件编译
范围（in）：
- 把每个 adapter 的实现拆到独立 module（或独立 crate），并用 `cfg(feature=...)` 控制
- `adapter_for()` / target registry 只暴露已编译 targets

验收标准：
- `cargo test --all-features` 通过
- `cargo build --no-default-features --features target-codex` 通过（至少验证一个子集）

---

#### P2-3-2（P2）测试与 CI：按 feature 跑 conformance
范围（in）：
- conformance tests 对 feature 做 `cfg`（缺失 target 时跳过）
- GitHub Actions 加 matrix：全 features + 子集 features

验收标准：
- CI 能在不同 feature 组合下稳定通过
- 不引入网络依赖

---

### Epic P2-4：Agentpack MCP server（Issue #196）（P2）

目标：提供一个 `agentpack-mcp`（或 `agentpack mcp serve`）作为 MCP server，暴露结构化工具调用接口。

#### P2-4-0（P1）工具契约设计（OpenSpec）+ 安全模型
范围（in）：
- 工具列表（最小集合）：`plan` / `diff` / `status` / `deploy_apply` / `rollback` / `doctor`
- 每个工具的参数 schema（profile/target/repo/machine/dry_run/yes 等）
- 返回值必须复用 `--json` envelope（减少第二套 contract）
- 明确“可变更工具”的批准语义（例如 deploy_apply 必须传 `yes=true`，否则返回 `E_CONFIRM_REQUIRED`）

验收标准：
- OpenSpec proposal + `docs/SPEC.md` 更新（新增 MCP server 章节）
- 安全/权限边界写清楚（默认只提供本地 stdio transport）

---

#### P2-4-1（P2）实现 MCP server skeleton（stdio transport）
建议实现方式：
- Rust 使用 MCP 官方/主流 SDK（例如 `rmcp` 系列），避免手写协议

范围（in）：
- 进程启动后能正确完成 init/handshake
- 能列出 tools（tool registry）
- 任意工具调用能返回 JSON（复用 envelope）

验收标准：
- 新增 integration test：用内置 client 或最小协议驱动做一次 tool 调用

---

#### P2-4-2（P2）实现 read-only 工具：plan/diff/status/doctor
验收标准：
- 行为与 CLI 一致（同一参数组合得到同语义输出）
- 错误码与 CLI 一致（尽量复用 `UserError` 分类）

---

#### P2-4-3（P2）实现 mutating 工具：deploy_apply / rollback（显式批准）
验收标准：
- 缺少批准参数时返回 `E_CONFIRM_REQUIRED`
- 成功时返回 snapshot_id 等信息（与 CLI JSON 一致）

---

#### P2-4-4（P1）文档：如何在 Codex 中配置 MCP server
范围（in）：
- 写 `docs/MCP.md`：示例配置、常见坑（stdio 路径、权限、工作目录）
- 补充到 README / Quickstart 的“高级用法”

验收标准：
- 文档不引用未来特性，能在当前实现上跑通

---

### Epic M3-GOV：组织/团队治理（长期，必须隔离）（P3）

> 这里是“长期计划”，不应阻塞 M0/M1/M2。任何治理能力都必须显式 opt-in，并且不改变个人用户默认行为。

#### M3-GOV-0（P3）治理层的“隔离设计”决策
建议方向（二选一，倾向 A）：
A) 独立子命令 + 独立配置文件
- 新增 `agentpack org ...`（或 `agentpack policy ...`）
- 只有当用户显式调用该子命令，才会读取 `agentpack.org.yaml`

B) 独立二进制
- `agentpack-org` / `agentpack-policy`，完全与核心 CLI 隔离

验收标准：
- 写一个架构说明（`docs/GOVERNANCE.md`），明确隔离边界与未来扩展点

---

#### M3-GOV-1（P3）Policy/Lint（非破坏性）第一步：把规范检查做成 CI 友好的命令
范围（in）：
- 新命令：`agentpack policy lint`（read-only）
- 检查项（可逐步扩展）：
  - 所有 `SKILL.md` frontmatter 必须完整
  - Claude commands 必须有 allowed-tools
  - 禁止在 operator assets 里出现“危险默认”（例如隐式 apply，无确认语义）
- `--json` 输出提供 machine-readable 结果（可作为 CI gate）

验收标准：
- 不影响任何现有命令
- 可在 GitHub Actions 中独立运行

---

#### M3-GOV-2（P3）Policy Pack：组织级规则的分发与版本化
范围（in）：
- 允许在 `agentpack.org.yaml` 引用一个“policy pack”（本地路径或 git source）
- 版本固定（类似 lockfile）以保证组织可审计

验收标准：
- 仍然是 opt-in（个人用户不配置就完全无感）

---

#### M3-GOV-3（P3）组织级“资产分发策略”（只在治理层生效）
设想：
- 组织希望规定：哪些 modules 必须启用、哪些 targets 必须写入、哪些路径禁止覆盖
- 但这些都应通过 `policy` 层约束，而不是污染 core deploy 语义

验收标准：
- 形成一份最小可行 spec（字段、错误码、与 deploy 的交互方式）

---

（Backlog 可以持续追加，但必须遵守：个人体验优先、契约稳定、治理层隔离。）


---

## Appendix A — Full implementation spec (verbatim from docs/SPEC.md)

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
      write_repo_skills: false         # optional in v0.1 (can keep off)
      write_user_skills: false

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

### 2.3 `<target root>/.agentpack.manifest.<target>.json` (target manifest)

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
- For backwards compatibility, agentpack MAY read the legacy filename `<target root>/.agentpack.manifest.json`, but MUST treat it as belonging to the selected target only when `tool == <target>`.

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
- (future) patch/diff overlays (e.g. 3-way merge based) are possible but not implemented yet

### 3.3 Overlay editing commands (see CLI)

`agentpack overlay edit <module_id> [--scope global|machine|project] [--sparse|--materialize]`:
- if the overlay does not exist: by default it copies the entire upstream module tree into the overlay directory (scope path mapping below)
- opens the editor (`$EDITOR`)
- after saving: changes take effect via deploy

Implemented options:
- `--sparse`: create a sparse overlay (write metadata only; do not copy upstream files; users add only changed files).
- `--materialize`: “fill in” missing upstream files into the overlay directory (copy missing files only; never overwrite existing overlay edits).

`agentpack overlay rebase <module_id> [--scope global|machine|project] [--sparsify]`:
- reads `<overlay_dir>/.agentpack/baseline.json` as merge base
- performs 3-way merge for files modified in the overlay (merge upstream updates into overlay edits)
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
- `--git`: ensure `.gitignore` contains `.agentpack.manifest*.json` (idempotent).
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
  - writes `.agentpack.manifest.<target>.json` under each target root
- delete protection: only deletes managed files recorded in the manifest (never deletes unmanaged user files)
- overwrite protection: refuses to overwrite existing unmanaged files (`adopt_update`) unless `--adopt` is provided
- without `--apply`: show plan only (equivalent to `plan` + `diff`)

Notes:
- `--json` + `--apply` requires `--yes` (otherwise `E_CONFIRM_REQUIRED`).
- If the plan contains any `adopt_update`, apply requires `--adopt`; in `--json` mode, missing `--adopt` returns `E_ADOPT_CONFIRM_REQUIRED`.
- Even if the plan is empty, if the target root is missing a manifest, agentpack writes a manifest (so drift/safe-delete works going forward).

### 4.7 `status`

`agentpack status`
- if the target root contains a compatible target manifest (`.agentpack.manifest.<target>.json`, or legacy `.agentpack.manifest.json` when `tool` matches): compute drift (`modified` / `missing` / `extra`) based on the manifest
- if there is no manifest (or the manifest has an unsupported `schema_version`): fall back to comparing desired outputs vs filesystem, and emit a warning
- if installed operator assets (bootstrap) are missing or outdated: emit a warning and suggest running `agentpack bootstrap`
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
- asset contents come from embedded templates shipped with agentpack (updated with versions)
- each operator file includes a version marker: `agentpack_version: x.y.z` (frontmatter or comment)

Requirement:
- If a Claude command uses bash execution, it must declare `allowed-tools` (minimal set).

Notes:
- In `--json` mode, `bootstrap` requires `--yes` (it writes to target roots; otherwise `E_CONFIRM_REQUIRED`).

### 4.10 `doctor`

`agentpack doctor [--fix]`
  - prints machineId (used for machine overlays)
  - checks target roots exist and are writable, with actionable suggestions (mkdir/permissions/config)
  - git hygiene (v0.3+):
  - if a target root is inside a git repo and `.agentpack.manifest*.json` is not ignored: emit a warning (avoid accidental commits)
  - `--fix`: idempotently appends `.agentpack.manifest*.json` to that repo’s `.gitignore`
    - in `--json` mode, if it writes, it requires `--yes` (otherwise `E_CONFIRM_REQUIRED`)

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

`agentpack schema`
- prints a brief JSON schema summary (human mode)
- `agentpack schema --json` documents:
  - `data.envelope` (the `schema_version=1` envelope fields/types)
  - `data.commands` (minimum expected `data` fields for key read commands)

## 5. Target adapter details

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

Deploy rules:
- command modules are single `.md` files; filename = slash command name
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
