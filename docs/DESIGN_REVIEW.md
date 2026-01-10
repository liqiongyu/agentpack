# Design Review & Gaps (v0.1)

本文档记录我在通读 `docs/PRD.md`、`docs/ARCHITECTURE.md`、`docs/SPEC.md`、`docs/BACKLOG.md` 后，对当前设计完整性、可实现性与路线规划的评审结论与建议补齐项。

> 目标：让 v0.1 的实现“无歧义、可复现、可回滚、可演进”，并避免中途因语义未钉死导致返工。

---

## 1. 总体结论

### 1.1 设计一致性（好的部分）
- 四份文档的主叙事一致：**manifest + overlays + lockfile** 驱动“声明式资产编译”，通过 **plan/diff/apply/validate/snapshot/rollback** 形成可审计闭环。
- v0.1 的范围边界清晰：只做 `codex` 与 `claude_code`，只做 `instructions/skill/prompt/command`，只做 `local_path` 与 `git` 两类 source。
- “copy/render（不依赖 symlink）”是符合现实约束的关键产品决策，并在 PRD/ARCH/SPEC 中保持一致。
- BACKLOG 的分解（A→B→C→D/E→F→G）基本符合实现依赖顺序。

### 1.2 可开工性（能否仅凭文档完成开发）
- **可以开工并完成 v0.1 的主体实现**（核心数据模型、lock/store、overlay 合成、plan/diff/apply/snapshot/rollback、两个 adapter、bootstrap、--json 输出框架）。
- 但 **尚未达到“完全无歧义”**：有几处语义缺口如果不先补齐，会导致实现分叉、行为不稳定、回滚/漂移检测不可靠，甚至引入误删风险。

### 1.3 与“论文设计”的对应性
当前 repo 未包含论文或论文摘要/术语/接口定义，因此无法判断“是否能一一对应论文设计”。
建议在 `docs/` 增加一个 `PAPER_MAPPING.md`（或把论文放入 `docs/paper/`），至少包含：
- 论文的目录（章节标题）
- 核心图（架构图/数据流图）与关键算法描述
- 对外接口/数据结构定义（如果有）

这样可以做“论文章节 → PRD/ARCH/SPEC/BACKLOG”的逐条映射并指出偏差。

---

## 2. v0.1 必须补齐的关键语义（按风险优先级）

> 下列条目是“实现最容易踩坑/最可能返工”的部分，建议视为 v0.1 的 P0 设计补齐项，而不是实现细节。

### 2.1 删除/清理语义与“归属权（ownership）”
当前 SPEC 定义了 `plan` 会输出 create/update/delete，但未定义：
- 什么时候允许 delete？
- delete 的作用域如何限制为“agentpack 管理过的文件”，避免误删用户手工文件？
- 当某个模块被移除/某个 target option 关闭时，目标目录里遗留的旧文件怎么处理？

建议补齐的最小可行方案（推荐）：
- **只允许删除“由 agentpack 管理并记录过”的文件**。
- 在 `state/deployments/<id>.json` 之外，维护一份“当前生效的 managed files 清单”（也可以复用最新 snapshot 作为权威来源，但要定义清楚）。
- `plan` 里的 delete 必须可追溯到“managed registry 中存在，但期望状态中不存在”的条目。
- 对于“同路径但非 managed 的文件”，一律不删，并在 `warnings` 提示“手工文件未托管”。

### 2.2 Overlay 漂移/冲突 warning 的可计算定义（需要基线）
BACKLOG B4 提到“upstream 更新导致 overlay 覆盖文件变动 warning”，但没有定义如何判断：
- overlay 覆盖的 upstream 文件是否发生变化？
- 以哪个版本作为基线？（overlay 创建时？上次 deploy 时？lockfile 中 resolved commit？）

建议补齐的最小可行方案：
- 当执行 `overlay edit`（或首次生成 overlay skeleton）时，为每个被覆盖的文件记录一份 **upstream baseline hash**（例如写到 `repo/overlays/<module_id>/.agentpack-overlay.json`）。
- 后续 lockfile/resolved commit 变化后，如果“当前 upstream 同路径文件 hash ≠ baseline hash”，则输出 warning（可附带 before/after hash 与建议人工 review）。
- v0.1 只做 warning，不自动 merge；v0.2 再引入 patch/3-way merge。

### 2.3 “repo”概念消歧：config repo vs project repo root
文档中同时存在两类“repo”：
- **Config Repo**：`agentpack.yaml`、`overlays/`、`projects/<project_id>/...` 所在的 repo（`--repo` 指向，默认 `$AGENTPACK_HOME/repo`）。
- **Project Repo Root**：用户当前工作的工程 repo（决定 `.claude/commands`、`AGENTS.md`、`$REPO_ROOT/.codex/skills` 的写入位置）。

如果不明确，会导致 adapter 在不同 cwd 下写错位置。

建议补齐：
- 在 SPEC/ARCH 中统一命名（例如 `config_repo` 与 `project_root`），并在所有命令的 JSON 输出中区分字段名。
- 明确定义 project_root 的探测规则（例如：优先 git root；否则 cwd；并把“无 git 仓库时 project_id 用 path hash”的规则与之对齐）。

### 2.4 Hash / file manifest / 可复现边界（跨平台一致性）
lockfile 需要稳定排序与 sha256，但还缺少关键定义：
- 目录遍历是否包含隐藏文件？是否排除 `.git/`、`target/`、`node_modules/`、`__pycache__/` 等？
- 是否跟随 symlink？是否允许 source 内出现 symlink？
- hash 计算是否基于“字节内容”还是“文本归一化”（CRLF/LF）？
- 文件权限位/可执行位是否纳入 hash？（跨平台差异很大）

建议补齐（v0.1 以确定性优先）：
- lockfile 的 hash 基于 **原始 bytes**（不做文本归一化）。
- 默认 **不跟随 symlink**：遇到 symlink 直接报错或警告并跳过（两者择一并写清）。
- 明确排除列表（至少 `.git`），并将其写入 SPEC，保证不同机器一致。
- 所有 file manifest 路径使用 `/` 作为分隔符并按字典序排序（或明确的稳定排序规则）。

### 2.5 Renderer 的模板变量集合与确定性
ARCH 提到支持 `{{project.name}}`、`{{git.remote}}`、`{{os}}` 等变量，但需要明确：
- v0.1 支持哪些变量？变量缺失时如何处理（空值/报错）？
- 是否允许变量影响“文件路径/文件名”？若允许，需要定义安全与转义策略。
- 渲染引擎的 escaping 规则（避免意外生成无效 frontmatter/markdown）。

建议：
- v0.1 先把变量集合收敛到极小（例如 `os`, `arch`, `project_id`, `git_remote`, `repo_root` 等），并把缺失策略写清。
- 先只支持“内容模板”，**不支持路径模板**（或明确限制字符集），降低风险。

### 2.6 validate 的范围、阻断策略与错误码
SPEC/BACKLOG 提到 validate，但未定义：
- validate 失败是否阻断 apply？是否回滚？
- 对不同类型资产做哪些校验？（如 Claude command frontmatter、SKILL.md 必需字段、AGENTS.md 大小限制等）

建议：
- v0.1 validate 以“低成本、低误报”为目标：只做 **必要字段与结构检查**，不做复杂语义解析。
- 定义一组稳定错误码（例如 `E_SCHEMA_INVALID`, `E_TARGET_NOT_FOUND`, `E_VALIDATION_FAILED`），并在 `--json` 中返回结构化 details。
- validate 失败时：
  - `deploy --apply`：默认阻断并不写入（或写入后立即 rollback），二者择一并写清。
  - `plan/diff/status`：只输出 warning/errors，不影响读取。

### 2.7 `--json` schema 的“子命令级”稳定性
SPEC 已给了顶层 `{ok, command, version, data, warnings, errors}`，但缺少：
- 每个命令的 `data` 结构字段定义（至少 plan/diff/status/deploy/rollback/lock/fetch）。
- 错误码枚举与向后兼容策略（字段新增、枚举扩展）。

建议：
- 在 `docs/` 增加 `JSON_SCHEMA.md`：逐命令列出 `data` 的稳定字段与可选字段，并定义错误码/警告码。
- 保持“新增字段不破坏旧消费者”的兼容原则（ARCH 已提到，建议落成文档）。

---

## 3. 对 SPEC 与 BACKLOG 的具体改进建议

### 3.1 建议在 SPEC 增补的章节（v0.1 必须）
- **Ownership & Delete Semantics**：managed registry、delete 的约束、unmanaged 文件处理策略。
- **Overlay Baseline Metadata**：overlay 生成时记录基线 hash，何时触发 warning，warning 的输出格式。
- **Naming & Resolution**：`config_repo` vs `project_root` vs `cwd` 的明确定义与探测规则。
- **Hashing & File Enumeration**：排除规则、symlink 策略、排序与跨平台一致性定义。
- **Template Variables**：支持的变量白名单、缺失策略、路径模板策略（建议 v0.1 禁止）。
- **Validation Rules**：各 module type 的最小校验列表、失败策略、错误码。
- **JSON Contract**：逐命令 `data` 字段定义与错误码枚举。

### 3.2 建议在 BACKLOG 中新增/调整的 P0 项
将下面条目加入 v0.1（P0），避免后期返工：
- [P0] A6. Lock hashing & file enumeration spec 固化（排除规则/symlink/排序/跨平台）
- [P0] B5. Overlay baseline metadata（记录 upstream 基线 hash + warning 计算）
- [P0] C0. Managed files registry（delete 语义/ownership/清理策略）
- [P0] F5. JSON schema 文档化 + error codes（逐命令稳定字段）

并建议策略性前置：
- 将 G2（golden tests：adapter plan 输出快照）与 D/E 同步推进（或拆成 D6/E5），因为外部工具发现规则最容易漂移，越早把语义固化到 golden tests 越好。

---

## 4. 建议的“待确认问题清单”（用于下一轮设计评审）

> 这些问题不回答清楚，最终实现会出现不同合理分支；建议在实现前确认并写入 SPEC。

1) `deploy` 默认是否必须 `--apply` 才写入？`--dry-run` 默认 true 这一点已写，但与 UX/安全策略是否一致？
2) `status` 的“期望状态”来源是什么？（当前 lock + overlays 合成产物）是否需要记录“上次 deploy 的版本”用于对比？
3) 对 Codex repo-scope skills：到底写到 `project_root/.codex/skills` 还是也支持更靠近 cwd 的层级？（ARCH/SPEC 提到了 precedence，但 deploy 时是否要多层写入？）
4) 对 Claude commands：当 repo 与 user 同名 command 冲突时，优先级如何体现？是否需要在 plan 输出中提示 shadowing？
5) `fetch` 的 store 布局：按 module_id？按 source hash？如何避免同一 source 被不同 module 引用造成重复下载？
6) “不执行第三方 scripts”与未来 eval gate 的边界：v0.2 的 eval 脚本由谁提供、如何信任、如何 sandbox？

---

## 5. 建议的下一步文档动作（最小集合）

如果只做最小补齐，我建议按顺序新增三份文档（都放 `docs/`）：
1) `docs/JSON_SCHEMA.md`：逐命令 `--json` 合约与错误码
2) `docs/SEMANTICS.md`：ownership/delete、overlay baseline、hashing/symlink、validate 策略
3) `docs/PAPER_MAPPING.md`：论文章节/术语与本项目 PRD/ARCH/SPEC/BACKLOG 的映射（需要论文材料）
