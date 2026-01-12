# docs/CODEX_WORKPLAN.md

Agentpack 工程改进与开源工程化工作单（Codex CLI 可直接执行）

目标
- 把 agentpack 变成“可复现、可审计、可回滚、对 agent 友好、对社区友好”的产品级开源 CLI。
- 把 CLI 的 `--json` 输出当作稳定 API 来维护：字段/错误码/行为尽可能版本化、可测试、可迁移。

工作方式（建议）
- 每个任务尽量独立成一个 PR（或一个 commit group），确保：
  - 有测试覆盖（至少单测或 CLI golden test）
  - `cargo fmt`、`cargo clippy`、`cargo test` 全绿
  - 文档与变更同时更新（README / SPEC / error codes registry / CHANGELOG）
- 每个 PR 的“验收条件”和“回归测试命令”写在 PR 描述里。

本仓库基础命令（按你 repo 现状自行调整）
- 运行测试：`cargo test`
- 格式化：`cargo fmt`
- 静态检查：`cargo clippy --all-targets -- -D warnings`

------------------------------------------------------------
P0：正确性/可移植性/不踩坑（必须优先）
------------------------------------------------------------

P0-1 模块 ID → 文件系统 key（fs_key）进一步加固
现状：你已引入 module_fs_key 并替换了 overlay/store 的目录命名，同时做了 legacy fallback。
需要补齐/加固：
1) 限制 fs_key 长度，避免超长 module_id 触发文件系统单段长度/总路径长度限制。
   - 建议：`{sanitized_prefix_trunc}--{hash10}`，prefix 截断 40~64 字符。
   - 要求：同一 module_id 必须稳定生成同一 fs_key；不同 module_id 必须几乎不可能碰撞（靠 hash）。
2) 增强 doctor：
   - 检测同一 module 同时存在 legacy 目录与 canonical(fs_key) 目录时给 warning，并提示迁移/清理策略。
3) overlay metadata：
   - 确保 overlay 目录内有“原始 module_id 的可读记录”（例如 `.agentpack/module_id` 或 meta.json）。
   - 若已存在，则 doctor 校验其一致性。

验收条件
- Windows 下 module_id 含 `:` 等字符也能正常 overlay/edit/deploy。
- 构造超长 module_id（>256）时，目录创建/访问仍稳定（fs_key 不超过限制）。
- 存在 legacy+canonical 并存时，doctor 给出清晰提示。

建议测试
- 新增：fs_key 截断测试、legacy/canonical 并存 doctor test、Windows 路径字符用例（至少通过 unit test 模拟 path segment）。

P0-2 原子写（atomic write）全面一致化
现状：你修复了主要路径上的“Update 先删文件”问题，但仍有关键配置文件使用 `std::fs::write`。
需要做：
1) 抽一个统一的 atomic write helper（例如 `fs::write_atomic(path, bytes)`）。
2) 将以下写入全部切换为 atomic write：
   - Manifest 保存
   - Lockfile 保存
   - TargetManifest 保存
   - overlay module_id/meta 写入（如果有）
   - 任何 “写 json/yaml 到磁盘”的路径
3) Windows 行为：
   - 优先使用“原子替换语义”的实现；尽量避免“先 remove 再写”的窗口期。
   - 若必须 fallback，务必将窗口期的风险写入注释与 docs，并加尽可能多的测试/诊断。

验收条件
- 所有关键文件写入都走同一 atomic helper。
- 发生写入失败（权限/被占用/路径为目录）时，错误码与错误信息可诊断，不会产生“半写坏文件”。

建议测试
- 用临时目录 + 预创建目标文件，验证“覆盖写”不会先删除导致短暂缺失（尽量在逻辑层验证）。
- 将目标路径替换为目录，验证能给出明确错误（不要吞掉）。

P0-3 DesiredState 输出路径冲突：human 体验与契约完善
现状：你已实现冲突检测与 `E_DESIRED_STATE_CONFLICT`。
需要补齐：
1) human（非 `--json`）模式下的错误信息更可读：
   - 输出冲突路径
   - 输出冲突 module_id 摘要（两侧各列若干 + “…(n more)”）
   - JSON 仍保留完整 details（hash/module_ids）
2) 文档化：在 SPEC/CLI 文档中写明冲突策略：
   - 同路径同内容：合并 module_ids
   - 同路径不同内容：报错（默认阻止 apply），除非未来引入 `--force-conflicts`（若你要支持）

验收条件
- 发生冲突时，人类一眼能看懂“谁跟谁冲突、冲在哪里”。

P0-4 首次覆盖“非托管文件”的 adopt/override 安全提示
问题：第一次 deploy 时，如果目标目录已有同名文件但不在 target manifest 管理范围内，目前可能被当作 Update 覆盖。
需要做（推荐策略）：
1) 在 plan 阶段区分：
   - managed_update（已托管文件更新）
   - adopt_update（将覆盖非托管但存在的文件）
2) 默认行为：
   - human 模式：对 adopt_update 给显著 warning（必要时要求确认）
   - `--json` 模式：返回稳定错误码（例如 `E_ADOPT_CONFIRM_REQUIRED`）或在 plan 中标记 `requires_confirm=true`
3) deploy 时：
   - 若未显式确认，不允许 apply adopt_update（避免自动化场景误伤）
4) 文档：说明“如何安全接管已有文件”的流程（plan -> confirm -> deploy -> rollback）。

验收条件
- 用户/agent 不会在无感知情况下覆盖非托管文件。
- 自动化（`--json --yes`）场景也必须有显式信号才能覆盖。

------------------------------------------------------------
P1：可维护性/agent 体验/闭环（强烈建议）
------------------------------------------------------------

P1-1 错误码体系“注册表化”（把 `--json` 当 API）
现状：你已增加多类错误码，但需要形成“契约”。
需要做：
1) 新增 `docs/ERROR_CODES.md`：
   - code：语义、触发条件、是否可重试、建议动作
   - 是否 breaking（语义变化算 breaking）
2) CLI 层保证：
   - `--json` 模式下凡是用户可预期的错误尽量不用 `E_UNEXPECTED`
   - `details` 字段结构稳定（schema_version 约束）
3) 增加测试：
   - 针对主要 user error 的 code 固定（golden tests）

验收条件
- agent 可以用 error code 做可靠分支逻辑，不用字符串匹配。

P1-2 events.jsonl 更像“可演进的审计日志”
现状：你已做到“坏行跳过 + warnings”。
建议进一步：
1) 明确定义 schema_version 的兼容策略（文档）：
   - 老版本读取新字段：忽略
   - 老版本读取新 schema_version：跳过并 warning
2) 增加字段（可选，注意向后兼容，只增不改）：
   - command_id、duration_ms、git_rev、snapshot_id、targets 摘要
3) score/report：
   - 输出 “skipped 行数/原因” 的统计
   - 遇到坏行不 fail 全局

验收条件
- 日志出现轻微损坏时，工具仍能工作且给出可诊断提示。

P1-3 evolve propose 覆盖面增强（把 drift 可靠回流到 source）
建议做（按优先级）：
1) 支持 missing drift：
   - `--propose-restore`：生成“恢复建议”（例如引导用户 redeploy 或生成 overlay 恢复文件）
2) 支持聚合文件的可回溯编辑（例如 AGENTS.md、汇总 instructions）：
   - 引入分段 marker：`<!-- agentpack:module=... -->`…`<!-- /agentpack -->`
   - evolve propose 可以定位到某个模块片段并生成对应 overlay/patch
3) 输出解释：
   - 对“为什么不能 propose”给出结构化原因（multi_module / read_error / missing / permissions）

验收条件
- drift 不再只是“检测到”，而是能“回流到可 review 的改动”。

P1-4 CLI 代码结构拆分（降低维护成本）
建议：
- 将 `src/cli.rs` 拆为：
  - `cli/mod.rs`（入口与共用输出）
  - `cli/commands/*.rs`（各子命令 handler）
  - `cli/json.rs`（envelope、schema_version、error mapping）
  - `cli/util.rs`（路径、confirm、print helpers）
- 同时统一：
  - human 输出风格（错误/警告/建议）
  - json 输出风格（envelope + code + details + warnings）

验收条件
- 新增命令不再需要在一个巨大文件里改来改去。
- 编译/阅读成本下降。

------------------------------------------------------------
P2：产品化与扩展（做强“护城河”）
------------------------------------------------------------

P2-1 overlay 从“整树复制”演进到 sparse/patch overlay
建议分阶段：
- 阶段 A（sparse overlay）：overlay init 时不复制上游，仅创建 baseline/meta；用户只放改动文件。
- 阶段 B（patch overlay）：用三方合并或文本 patch（可选）减少与上游漂移的痛苦。

验收条件
- overlay repo 不再随着上游体积线性膨胀。
- 上游更新后合并成本下降。

P2-2 lockfile 中 local_path 的跨机一致性
问题：绝对路径会导致 lockfile 在不同机器 diff 巨大，影响同步/审计。
建议：
- local_path 在 lockfile 中记录 repo-relative 路径或只记录 hash/manifest，不落绝对 resolved path。

验收条件
- 同一配置 repo 在不同机器上生成 lockfile diff 可控。

P2-3 JSON 输出路径标准化
建议：
- JSON 输出统一使用 POSIX 风格路径（`/`），或同时输出 `path`（posix）与 `os_path`（native）。
- human 输出用 OS native。

验收条件
- agent 解析路径不会因 `\`/`/` 差异写一堆特判。

------------------------------------------------------------
开源工程化补齐项（Codex CLI 可执行：写代码/写文档/加 workflow）
------------------------------------------------------------

OSS-1 GitHub Actions CI（三平台矩阵）
要做：
- 新增 `.github/workflows/ci.yml`
  - matrix：ubuntu-latest / macos-latest / windows-latest
  - steps：checkout、安装 Rust（按 rust-toolchain）、cache、`cargo fmt --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`
- 保证 CI job 名稳定，便于后续 branch protection “required checks”。

验收条件
- PR 自动跑完整 CI。
- Windows 也真正覆盖到关键路径（尤其路径/atomic）。

OSS-2 依赖安全扫描：cargo-audit + cargo-deny
要做：
1) 新增 CI job：`cargo audit`（安装 `cargo-audit` 后运行）
2) 新增 `deny.toml` 并在 CI 中运行 `cargo deny check`（可先从 `cargo deny init` 生成默认配置再逐步收紧）
3) 文档化：`docs/SECURITY_CHECKS.md` 说明本仓库采用哪些扫描、失败如何处理。

验收条件
- PR 自动发现 RustSec 漏洞、license/source 风险、重复依赖等。
- 对 false positive 有明确的 allowlist/例外处理机制（写在 deny.toml 与 docs）。

OSS-3 OpenSSF Scorecard（给项目“安全健康度”信号）
要做：
- 新增 `.github/workflows/scorecard.yml` 使用官方 action
- README 加 badge（可选）
- 让结果进入 Security tab（按 action 推荐配置）

验收条件
- main 分支定期/按 push 运行 scorecard。
- scorecard 失败不会阻塞 merge（可先 informational），但要能看到结果并逐步提分。

OSS-4 发布工程：cargo-dist（自动构建多平台二进制与 Release）
要做（推荐直接按 cargo-dist 的 init 流程）：
1) 在本地/CI 里运行：`cargo dist init`
2) 检查生成的 `dist` 配置与 workflows（release.yml 等）
3) README 增加安装方式：
   - `cargo install agentpack --locked`
   - GitHub Releases 下载二进制（由 cargo-dist 产物提供）
4) 增加 `docs/RELEASING.md`：
   - 打 tag → 触发 release
   - 如何写 release notes / 更新 changelog
   - 如何回滚

验收条件
- 打 tag 后自动出 Release，包含三平台产物。
- 安装指引清晰，新用户 30 秒能跑起来。

OSS-5 Issue templates / PR templates（降低贡献沟通成本）
要做：
- 新增 `.github/ISSUE_TEMPLATE/bug_report.yml`
- 新增 `.github/ISSUE_TEMPLATE/feature_request.yml`
- 新增 `.github/ISSUE_TEMPLATE/config.yml`（可选：提供“提问/讨论”的入口）
- 如果已有 PR template，确认内容可执行（复现步骤/测试/文档/变更类型）

验收条件
- 用户提 issue 能提供足够信息（OS、版本、复现步骤、预期/实际、日志）。
- 维护者 triage 成本明显下降。

OSS-6 文档化你的“稳定契约”
要做：
- `docs/JSON_API.md`：列出 `--json` 的 envelope、schema_version、warnings、errors 的稳定性承诺。
- `docs/ERROR_CODES.md`：错误码注册表（见 P1-1）
- `docs/ARCHITECTURE.md`：核心链路（desired -> plan/diff -> apply snapshot -> status/drift -> record/score -> evolve）

验收条件
- 外部贡献者能靠 docs 理解系统边界与演进方式。
- agent 可以把 JSON 契约当 API 用。

OSS-7 版本与变更记录（SemVer + Keep a Changelog）
要做：
- 确保 CHANGELOG 遵循 Keep a Changelog 的结构（Added/Changed/Fixed/Deprecated/Removed/Security）
- 明确 SemVer 策略：哪些变更算 breaking（尤其 JSON schema / error code 语义）
- 每次 release 必须更新 CHANGELOG 并写发布日期

验收条件
- 用户能通过版本号与 changelog 理解升级风险。

------------------------------------------------------------
建议的 PR 切分顺序（从收益/风险比最高开始）
------------------------------------------------------------
1) CI（三平台）+ fmt/clippy/test 全跑通（OSS-1）
2) 原子写一致化（P0-2）
3) adopt_update 安全提示（P0-4）
4) ERROR_CODES/JSON_API 文档化（P1-1 + OSS-6）
5) cargo-audit + cargo-deny（OSS-2）
6) Scorecard（OSS-3）
7) cargo-dist 发布（OSS-4）
8) overlay sparse/patch、lockfile 路径、evolve 聚合回溯（P2/P1-3）
