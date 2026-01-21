---
status: proposed
owner: liqiongyu
last_updated: 2026-01-21
baseline: v0.8.0 (shipped)
scope: unified-single-release
audience: maintainers + Codex (dev agent)
---

# Agentpack 统一版本迭代规划（Spec / Epics / Backlog，Codex-Ready）

这是一份“可以直接指导 Codex 开发”的统一版本规划文档：把工程闭环、生态扩展与传播/叙事/上手体验一起纳入同一版本交付。文档结构遵循 **Spec → Epics → Backlog**，每条 Backlog 都写到可拆 PR、可跑测试、可验收。

## 1. 版本目标（North Star）

本版本交付后，Agentpack 要满足下面 4 条“可感知的胜利条件”：

1) **5 分钟价值可感知**：用户 clone 仓库后，不污染真实环境（临时 HOME/AGENTPACK_HOME），一条命令看到 plan/diff，并理解 Agentpack 的价值点与边界。
2) **传播与上手更抓人**：GitHub About 有清晰 tagline；README 有 3 个真实场景 + 一次完整闭环演示 + “为什么不用 X”对比；docs 有架构图 + 90 秒 demo（可复现生成）。
3) **安全可编排且可拒绝**：`--json` 与 MCP 写入类能力默认保守；拒绝路径能给 reason_code + next_actions；两阶段确认（confirm_token/等价绑定）在 E2E 里被锁死，避免回归。
4) **可扩展但不变成巨石**：targets/overlays/mcp/handlers 的内部结构更清晰；新增 target 必须 feature-gate + mapping doc + conformance tests。

> 行业事实接口对齐（用于传播叙事与“为什么现在需要 Agentpack”）：
> - Codex 会读取 `AGENTS.md` 作为项目级自定义指令入口。
> - Codex/Claude Code 都以 `SKILL.md` 为核心组织可复用的 Skills；Claude Code 的自定义 slash commands 已合并进 skills。
> - MCP 的 tools/list + tools/call 模型正在成为 agent 调用外部工具的通用接口。
> 以上会体现在 README/对比页/架构图/示例里。

## 2. 范围与非目标（Scope / Non-goals）

### Scope（本版本要做的）
- 传播资产：tagline / README story / 架构图 / 90 秒 demo / “为什么不用 X”对比页 / 5 分钟 demo。
- 工程质量：Journeys J1–J8（至少 P0 的 4 条先落地）；CI 更稳；拒绝与确认链路可回归。
- 可维护性：围绕 overlays/targets/mcp/handlers 做“行为不变”的结构重整（小 PR 化）。
- 生态扩展：最多新增 **1 个实验 target**（必须 feature-gate + conformance）。

### Non-goals（明确不做）
- 不把 Agentpack 做成通用 dotfiles manager（把边界写清楚，避免误用）。
- 不在本版本做“多云/在线托管服务”。
- 不引入重型图形化依赖链（架构图优先 Mermaid；demo gif 采用脚本化工具，能在本地/CI 重建）。

## 3. 全局不变量（Codex 必须遵守）

### 3.1 外部契约优先（Do not break userspace）
- `--json`：schema_version=1，只允许**新增字段**，不允许删/改语义/重命名。
- `ERROR_CODES`：稳定错误码不随意改；新增错误码必须同步文档与测试。
- 任一 PR 如果影响外部行为：必须补文档（用户向），并且至少跑一条相关 Journey。

### 3.2 写入动作必须可拒绝（安全默认）
- CLI：`--json` 模式下的写入类命令必须显式 `--yes`。
- MCP：写入类工具必须两阶段（plan → apply），apply 必须携带 confirm_token（或等价绑定）。
- 所有拒绝路径必须提供稳定 `reason_code` + `next_actions`（便于 orchestrator/agent 推进）。

### 3.3 Docs-as-code
- CLI reference 由生成流程产出（`agentpack help --markdown` 等），并由 doc-sync 测试防漂移。
- 文档链接检查、版本号同步检查必须在 CI 里强制。

### 3.4 小 PR 原则
- 每个 Backlog item = 1 PR；尽量 200–500 行有效变更（不含测试/文档）。
- PR 描述必须包含：动机、实现、测试命令、影响面、回滚策略。

---

# 4. Spec（统一版本规格）

## 4.1 传播与叙事 Spec（必须补齐：tagline / 场景 / 演示 / 对比 / 架构图 / demo）

### A) GitHub About + README 顶部 tagline（一句话抓住用户）
要求：统一一套中英文短句，明确“是什么/为谁/解决什么”。
- EN（候选）：**“A declarative, safe control plane for deploying coding-agent assets across tools.”**
- ZH（候选）：**“面向 AI 编程代理的本地资产控制面：声明式管理与安全部署 AGENTS/Skills/Commands/Prompts。”**

验收：
- GitHub About（手工操作）与 README 顶部完全一致；仓库内留 `docs/GITHUB_SETUP.md` checklist。

### B) README “为什么你需要它”（故事化呈现）
必须新增三块：
1) **3 个典型真实场景**（跨工具一致性 / 可复用+可回滚 / 多机同步+审计）
2) **一次完整闭环演示**（update → preview --diff → deploy --apply → status → rollback）
3) **“为什么不用 X”入口**（stow/chezmoi/yadm 对比页链接，强调边界与差异）

验收：
- README 读完不需要点进 docs，也能理解价值与边界。
- 演示命令不虚构，必须与当前 CLI 实现一致。

### C) 一张架构图（GitHub 可直接渲染）
形式：Mermaid（可选再导出 SVG）。必须表达链路：
manifest/lock/overlays → compose/materialize → render targets → plan/diff → apply → manifests/snapshots/events。

验收：
- `docs/explanation/architecture.md` 中新增 Mermaid 图；README 与 docs/index 至少一个地方直接展示该图。
- 图足够简洁，不把所有模块细节塞进一张图。

### D) 一个 90 秒 CLI 演示（GIF，且可复现生成）
要求：
- 使用脚本化录制（推荐 `charmbracelet/vhs`，用 `.tape` 生成 gif；或 asciinema+agg 作为备选）。
- 演示必须在 temp HOME/AGENTPACK_HOME 下进行，不污染真实环境。
- 输出放在 `docs/assets/demo.gif`，脚本放在 `docs/assets/demo.tape`，并写明复现步骤。

验收：
- maintainer 在本地可以一条命令重新生成（例如 `just demo-gif`）。
- README 引用该 demo（gif 或 mp4/webm，若 gif 太大可切换格式）。

### E) “5 分钟可运行 demo”（最小可运行示例）
仓库已有 `docs/examples/minimal_repo`，本版本要补齐“一键跑”的体验：
- 新增 `scripts/demo_5min.sh`（可选再加 `scripts/demo_5min.ps1`）
- 新增 `docs/tutorials/demo-5min.md`（中英文）
- 一键脚本必须做到：创建 temp HOME + temp AGENTPACK_HOME + temp workspace；运行 `doctor` + `preview --diff`（默认不 apply）。

验收：
- 新用户 5 分钟内看到 plan/diff，理解下一步（如果想写入，再用显式 `--yes`）。

## 4.2 “为什么不用 dotfiles manager”对比 Spec（避免稻草人）

新增对比页：`docs/explanation/compare-dotfiles-managers.md`（并提供 `docs/zh-CN/...`）。要求：
- 对比对象：GNU Stow / chezmoi / yadm。
- 明确它们各自擅长什么；Agentpack 的差异点是什么（targets、overlay rebase、adopt/rollback、MCP/JSON 合约）；以及“什么时候不该用 Agentpack”。
- 关键论述必须引用官方文档链接（见文末参考）。

验收：
- 不贬低对方，不误导用户；强调边界与组合使用方式（例如：用 chezmoi 管全部 dotfiles，用 Agentpack 管 agent 资产）。

## 4.3 E2E / Journeys Spec（工程质量主线）

目标：把真实用户旅程（而不是单点测试）锁进 CI，防止“写盘语义/确认语义/错误码/JSON 合约”回归。

本版本要求完成 8 条 Journeys（J1–J8），其中 P0 必须先完成：J1/J2/J3/J8。

- J1 From-scratch first deploy（init→update→preview→deploy/apply→status→rollback）
- J2 Import existing assets（user+project）
- J3 Adopt_update 拒绝→adopt→成功（reason_code/next_actions 必须稳定）
- J4 Overlay sparse→materialize→rebase→deploy
- J5 Patch overlay：生成→rebase→apply（冲突工件）
- J6 Multi-machine sync：bare remote + rebase
- J7 Cross-target consistency：同一模块输出多个 targets
- J8 MCP confirm：plan→token→apply→rollback（错 token/无 token 必须拒绝）

真实性约束：
- temp HOME / temp AGENTPACK_HOME / temp workspace
- 默认不依赖联网
- 断言以结构化 JSON 为主，减少跨平台文本漂移

## 4.4 可维护性重构 Spec（行为不变，小步拆分）
原则：不改变 CLI/JSON/ERROR_CODES 的外部行为；每次重构必须有 golden/contract tests + 至少 1 条 Journey 托底。

方向：
- overlays：layout/dir/patch/rebase 分层
- targets：TargetAdapter + registry 结构更清晰
- mcp：server/tools/confirm 分离
- handlers：CLI/MCP/TUI 共用业务层（guardrails 单点）

## 4.5 Targets & ecosystem Spec（最多新增 1 个实验 target）
硬门槛：
- feature-gate（默认关闭）
- mapping doc（按模板）
- conformance tests（进入 CI matrix）

---

# 5. Epics（统一版本的工作分解）

- **E0（P0）传播与上手体验**：tagline / README story / 架构图 / demo gif / 对比页 / 5 分钟 demo
- **E1（P0-P1）Docs 可发现性与一致性**：入口更强、链接检查、doc-sync、更稳的生成 reference
- **E2（P0-P1）E2E Journeys & CI 稳定性**：J1–J8 全量、拒绝/确认链路锁死
- **E3（P1）可维护性重构**：overlays/targets/mcp/handlers 拆分（行为不变）
- **E4（P2）Targets 扩展（可选）**：新增 1 个实验 target + conformance 扩展
- **E5（P2）治理轨（可选）**：强隔离回归测试、policy 工具链完善（若资源允许）

---

# 6. Backlog（细粒度任务清单，可逐条 PR）

> 任务格式约定：
> - **ID**：E<epic>-<type>-<nnn>
> - **Priority**：P0/P1/P2
> - **Deliverable**：代码/文档/资源产物
> - **Acceptance**：可验收清单
> - **Tests**：必须列出可运行命令
> - **Dependencies**：上游依赖（若有）

## E0：传播与上手体验（P0）

### E0-MKT-001（P0）统一 tagline（README + docs + setup checklist）
- Deliverable:
  - `README.md`、`README.zh-CN.md` 顶部 tagline（统一短句）
  - `docs/GITHUB_SETUP.md` 增加 “Repo About/Topics/Website” checklist（含中英文可复制文案）
- Acceptance:
  - README 3 秒内能读懂：是什么/为谁/解决什么
  - checklist 明确“需要手工设置 GitHub About”的字段内容
- Tests:
  - `cargo test --test cli_reference_markdown_generated`
  - `cargo test --test docs_markdown_links`（若已存在；否则由 E1-DOC-006 引入）

### E0-MKT-002（P0）README：3 场景 + 完整闭环演示 + 对比页入口
- Deliverable:
  - README 新增 “Why you need this” 三场景
  - README 新增 “One full loop” 演示（含 rollback）
  - README 链接到对比页（E0-DOC-004）与 5 分钟 demo（E0-MKT-005）
  - 同步 `README.zh-CN.md`
- Acceptance:
  - 演示命令与当前 CLI 一致，不虚构
  - “为什么不用 X”入口清晰
- Tests:
  - `cargo test --test docs_markdown_links`
  - `cargo test --test cli_help_schema`（确保 help/flags 没漂移导致文档错误）

### E0-DOC-003（P0）架构图：在 `docs/explanation/architecture.md` 添加 Mermaid，并在 README 展示
- Deliverable:
  - `docs/explanation/architecture.md` 增加 Mermaid 架构图（简洁）
  - 新增 `docs/zh-CN/explanation/architecture.md`（中文版本，图可复用）
  - README 与 docs/index 至少一个地方直接展示该图（非仅链接）
- Acceptance:
  - 图表达：manifest/lock/overlays→render→plan/diff→apply→manifests/snapshots/events
- Tests:
  - `cargo test --test docs_markdown_links`

### E0-DOC-004（P0）“为什么不用 stow/chezmoi/yadm”对比页（中英文）
- Deliverable:
  - `docs/explanation/compare-dotfiles-managers.md`
  - `docs/zh-CN/explanation/compare-dotfiles-managers.md`
  - `docs/index.md` 与 `docs/zh-CN/index.md` 增加入口
- Acceptance:
  - 1 张对比表（目标对象/部署模型/安全与回滚/targets/overlay rebase/自动化契约）
  - 3 条“何时用它们更合适” + 3 条“Agentpack 的差异点”
  - 引用官方文档链接（见参考）
- Tests:
  - `cargo test --test docs_markdown_links`

### E0-MKT-005（P0）5 分钟 Demo：一键脚本（仅写 temp）+ 教程（中英文）
- Deliverable:
  - `scripts/demo_5min.sh`（必选）
  - `docs/tutorials/demo-5min.md` + `docs/zh-CN/tutorials/demo-5min.md`
  - docs/index 与 README 增加入口
- Implementation notes（建议）：
  - 脚本创建临时目录：`HOME=$(mktemp -d)`、`AGENTPACK_HOME=$(mktemp -d)`
  - 使用 `docs/examples/minimal_repo` 作为 config repo
  - 跑：`agentpack doctor --json`、`agentpack preview --diff --json`（不 apply）
- Acceptance:
  - 一条命令跑通并输出 plan/diff（不污染真实 HOME）
  - 教程明确下一步：如何进入 apply（显式 `--yes`）以及如何 rollback
- Tests:
  - 新增 `tests/cli_demo_5min_script.rs`（执行脚本并断言退出码）
  - 继续保留/扩展 `tests/cli_examples_minimal_repo.rs`

### E0-MKT-006（P1）90 秒 CLI Demo：可复现生成 GIF（VHS）
- Deliverable:
  - `docs/assets/demo.tape`
  - `docs/assets/demo.gif`
  - `docs/assets/README.md`（如何生成/更新 demo）
  - `just demo-gif`（或 `cargo xtask demo-gif`）
- Acceptance:
  - demo 在 temp HOME/AGENTPACK_HOME 下演示：update → preview --diff → deploy --apply --yes → status → rollback
  - 生成过程可复现，不需要手工录屏
- Tests:
  - 新增 `tests/docs_assets_exist.rs`：断言 demo.gif 存在且 README 引用正确

## E1：Docs 可发现性与一致性（P0-P1）

### E1-DOC-001（P0）docs/index 强化入口：5 分钟 demo + 对比页 + 架构图
- Deliverable:
  - `docs/index.md`、`docs/zh-CN/index.md` 增加明显入口链接
- Acceptance:
  - 新用户 30 秒内能选择：Quickstart / Import / Demo-5min / Compare / Architecture
- Tests:
  - `cargo test --test docs_markdown_links`

### E1-DOC-006（P0）文档链接检查纳入 CI（断链即失败）
- Deliverable:
  - 新增或增强 `tests/docs_markdown_links.rs`（覆盖 docs 与 README）
  - `.github/workflows/ci.yml` 中加 job 或把 test 设为 required
- Acceptance:
  - 内部链接断裂必然被 CI 拦住
- Tests:
  - `cargo test --test docs_markdown_links`

### E1-REL-002（P1）版本一致性校验：README/安装片段/CLI reference 自动对齐
- Deliverable:
  - 增强 `tests/cli_reference_markdown_generated.rs` 或新增 `tests/docs_install_snippets_sync.rs`
- Acceptance:
  - 版本号/命令片段与发布产物不再漂移
- Tests:
  - `cargo test --test cli_reference_markdown_generated`
  - `cargo test --test docs_install_snippets_sync`

## E2：E2E Journeys & CI 稳定性（P0-P1）

### E2-TST-001（P0）Journey 基础设施：JSON helper + 拒绝链路断言
- Deliverable:
  - `tests/journeys/common/` 增加 `run_json()`、`assert_reason_code()`、`assert_next_actions()` 等 helper
- Acceptance:
  - 后续 J1–J8 测试文件不再重复样板代码
- Tests:
  - `cargo test --test journeys_smoke`（新增最小 smoke）

### E2-TST-010（P0）J1：From-scratch first deploy（含 rollback）
- Deliverable:
  - `tests/journeys/j1_from_scratch.rs`
- Acceptance:
  - init→update→preview --diff→deploy --apply --yes→status→rollback 全链路
  - 断言退出码 + JSON 合约关键字段 + manifest/snapshot 产物
- Tests:
  - `cargo test --test journeys_j1_from_scratch`

### E2-TST-011（P0）J2：Import existing assets（user+project）
- Deliverable:
  - `tests/journeys/j2_import.rs`
- Acceptance:
  - import dry-run 不写盘；apply + `--yes` 才写盘
  - import 后 preview/deploy 成功
- Tests:
  - `cargo test --test journeys_j2_import`

### E2-TST-012（P0）J3：Adopt_update 拒绝→adopt→成功（reason_code/next_actions）
- Deliverable:
  - `tests/journeys/j3_adopt_update.rs`
- Acceptance:
  - 无 adopt 时拒绝且错误码稳定；有 adopt 后成功且不重复要求 adopt
- Tests:
  - `cargo test --test journeys_j3_adopt_update`

### E2-TST-017（P0）J8：MCP confirm_token（plan→token→apply→rollback）
- Deliverable:
  - `tests/journeys/j8_mcp_confirm.rs`
- Acceptance:
  - 错 token/无 token 必须拒绝，且给 reason_code/next_actions
  - 正确 token apply 成功，并可 rollback
- Tests:
  - `cargo test --test journeys_j8_mcp_confirm`

### E2-TST-013（P1）J4：Overlay sparse→materialize→rebase→deploy
- Deliverable: `tests/journeys/j4_overlay_rebase.rs`
- Tests: `cargo test --test journeys_j4_overlay_rebase`

### E2-TST-014（P1）J5：Patch overlay 冲突工件（生成→rebase→apply）
- Deliverable: `tests/journeys/j5_patch_overlay.rs`
- Acceptance:
  - 冲突时生成可定位工件 + 稳定错误码 + next_actions
- Tests: `cargo test --test journeys_j5_patch_overlay`

### E2-TST-015（P1）J6：Multi-machine sync（bare remote + rebase）
- Deliverable: `tests/journeys/j6_multi_machine_sync.rs`
- Tests: `cargo test --test journeys_j6_multi_machine_sync`

### E2-TST-016（P1）J7：Cross-target consistency（同一模块输出多个 targets）
- Deliverable: `tests/journeys/j7_cross_target.rs`
- Tests: `cargo test --test journeys_j7_cross_target`

## E3：可维护性重构（P1）

> 约束：行为不变；每个重构任务必须跑至少：`cli_error_codes`、`cli_help_schema`、以及 J1 或 J8 之一。

### E3-REF-001（P1）overlays 拆分：layout/dir/patch/rebase
- Deliverable: `src/overlays/*` 重排；公共逻辑抽到子模块
- Acceptance:
  - 文件规模下降；接口更清晰；行为不变
- Tests:
  - `cargo test --test cli_error_codes`
  - `cargo test --test cli_help_schema`
  - `cargo test --test journeys_j1_from_scratch`

### E3-REF-002（P1）targets 分层：TargetAdapter + registry 更清晰
- Deliverable: `src/targets/*` 与 registry 结构更可扩展
- Tests:
  - `cargo test --test conformance_targets`

### E3-REF-003（P1）mcp 分层：server/tools/confirm 分离
- Deliverable: `src/mcp/server.rs`、`src/mcp/tools.rs`、`src/mcp/confirm.rs`（示例）
- Tests:
  - `cargo test --test journeys_j8_mcp_confirm`

### E3-REF-004（P1）handlers 统一：CLI/MCP/TUI 共用业务层（guardrails 单点）
- Deliverable: 业务逻辑集中到 `src/app/*`（示例）
- Tests:
  - `cargo test --test cli_guardrails`
  - `cargo test --test journeys_j8_mcp_confirm`

## E4：Targets & ecosystem（P2，可选）

### E4-TGT-001（P2）新增 1 个实验 target（feature-gate + mapping doc + conformance）
- Acceptance:
  - 默认关闭；打开 feature 后 conformance 全过
  - `docs/reference/targets.md` 标注 maturity（experimental）
- Tests:
  - `cargo test --test conformance_targets --no-default-features --features <new_target_feature>`

### E4-DOC-002（P2）targets 能力矩阵（文档）
- Deliverable: `docs/reference/targets.md` 增加一张能力矩阵表
- Acceptance: 用户一眼知道哪些 target 成熟、支持哪些 module types/范围

## E5：治理轨（P2，可选）

### E5-GOV-001（P2）强隔离回归测试：core 永不读取 org config
- Deliverable: 新增/增强 `tests/cli_org_config_isolation.rs`
- Tests: `cargo test --test cli_org_config_isolation`

---

# 7. Release Checklist（统一版本发版清单）

1) 运行全量测试：`cargo test --all --locked`（以及 CI matrix）
2) 确认 `--json` schema_version=1 未破坏，ERROR_CODES 稳定（相关测试必须绿）
3) 生成并提交 CLI reference（若为生成物）：`agentpack help --markdown > docs/reference/cli.md`（或你们现有生成流程）
4) 更新 README/Docs：tagline、架构图、对比页、demo-5min、demo.gif
5) Demo 资产更新：`just demo-gif`（确保 demo.gif 与 tape 同步）
6) 更新变更日志 `CHANGELOG.md`，并确保版本号/安装片段一致（doc-sync tests 绿）
7) GitHub About 手工设置（按 `docs/GITHUB_SETUP.md` checklist）

---

# 8. 参考（用于 README/对比页/教程引用的权威链接）

> 下面列的是“叙事与传播”里需要引用的官方/权威资料，写文档时请尽量优先用它们。

## Codex / AGENTS.md / Skills
- OpenAI Codex：Custom instructions with `AGENTS.md`
  https://developers.openai.com/codex/guides/agents-md/
- OpenAI Codex：Agent Skills（`SKILL.md` 结构）
  https://developers.openai.com/codex/skills/
- OpenAI Codex：Create skills（用法与动机）
  https://developers.openai.com/codex/skills/create-skill/

## Claude Code：skills / slash commands
- Claude Code Docs：Extend Claude with skills
  https://code.claude.com/docs/en/skills
- Claude Code Docs：Slash commands（并入 skills 的说明）
  https://code.claude.com/docs/en/slash-commands

## MCP：tools/list & tools/call
- MCP Spec：Tools（2025-06-18 revision）
  https://modelcontextprotocol.io/specification/2025-06-18/server/tools

## 文档方法论（Diátaxis）
- Diátaxis framework
  https://diataxis.fr/
- Diátaxis in five minutes（四象限：tutorial/how-to/reference/explanation）
  https://diataxis.fr/start-here/

## dotfiles managers（用于“为什么不用 X”对比页）
- GNU Stow manual（symlink farm manager）
  https://www.gnu.org/s/stow/manual/stow.html
- chezmoi（manage dotfiles across multiple machines）
  https://chezmoi.io/
- yadm（dotfiles in $HOME, git-based）
  https://yadm.io/

## 录制可复现 CLI demo（VHS / 备选 agg）
- VHS（Your CLI home video recorder）
  https://github.com/charmbracelet/vhs
- VHS man page（VHS reads .tape files and renders GIFs）
  https://man.archlinux.org/man/extra/vhs/vhs.1.en
- agg（asciinema gif generator）
  https://docs.asciinema.org/manual/agg/
