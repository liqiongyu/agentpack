# CODEX_WORKPLAN.md

Agentpack 工程改进与开源工程化工作单（给 Codex CLI/自动化直接执行用）

> Current as of **v0.5.0** (2026-01-13). v0.5.0 已完成一轮关键正确性收敛（fs_key 长度上限、全链路 atomic write、adopt 保护、sparse overlay + rebase、CLI 拆分、overlay metadata/doctor 检查等）。本工作单聚焦“把它做成真正优秀的开源项目”的下一步。

使用方式建议：
- 每个任务一个 PR（或一个 commit group），PR 描述包含：目的、验收条件、回归测试命令。
- 每个 PR 至少跑：`cargo fmt`、`cargo clippy --all-targets -- -D warnings`、`cargo test`。

------------------------------------------------------------
P0：回归测试与契约锁定（优先）
------------------------------------------------------------

P0-1 JSON 契约 golden tests（强烈建议）
- 目标：锁住 `schema_version=1` 的 envelope 字段与关键错误码（避免无意 breaking）。
- 建议做法：
  - 用临时目录创建“伪 AGENTPACK_HOME”与“伪 target roots”。
  - 运行真实 CLI（或调用内部命令入口），对 `plan/preview(deep)/deploy --apply/status/rollback` 以及关键错误码场景做 snapshot/golden。
  - 至少覆盖：
    - `E_CONFIRM_REQUIRED`（--json 缺 --yes）
    - `E_ADOPT_CONFIRM_REQUIRED`（adopt_update）
    - `E_DESIRED_STATE_CONFLICT`（冲突）
    - `E_OVERLAY_REBASE_CONFLICT`（冲突）

P0-2 Conformance harness
- 目标：新增 target 前，必须能跑一套“语义一致性测试”。
- 覆盖点（见 `TARGET_CONFORMANCE.md`）：
  - delete protection（只删托管）
  - manifest（每个 root 写 `.agentpack.manifest.json`）
  - drift（missing/modified/extra）
  - rollback（可恢复）

P0-3 Windows 路径与权限用例
- 目标：确保 Windows 下 overlay 与 deploy 不被路径字符/长度/权限轻易打爆。
- 建议：
  - 单测覆盖 `module_fs_key` 截断/稳定性已完成；补集成测试：overlay edit/rebase 生成的路径在 Windows runner 上可用。

------------------------------------------------------------
P1：产品体验（可日用）
------------------------------------------------------------

P1-1 status 输出增强（不破坏 JSON）
- human 模式：按 root 分组、给下一步建议（例如“run bootstrap”/“run deploy --apply”）。
- json 模式：可以 additive 增加 `summary`，但不要删除/重命名现有字段。

P1-2 evolve propose 的可解释性
- 把 skipped reasons 做成更结构化、可行动：
  - missing → 推荐 `evolve restore` 或 `deploy --apply`
  - multi_module_output → 建议补 marker 或拆分输出

P1-3 docs/examples（用户更友好）
- 本轮 docs 已收敛；后续建议补：
  - 一个最小示例 repo（含 `agentpack.yaml` + 几个模块）
  - “从 0 到多机器同步”的录屏/动图（可选）

------------------------------------------------------------
OSS：开源工程化（需要 GitHub 配置的另见 docs）
------------------------------------------------------------

OSS-1 贡献者体验
- `CONTRIBUTING.md`（根目录）+ issue/pr templates
- `CODE_OF_CONDUCT.md`
- `SECURITY.md`（漏洞报告入口）
- `LICENSE`

OSS-2 发布与分发
- 确认 `cargo-dist` release workflow 在三平台产物、校验和、签名（可选）上都稳定。

（GitHub 侧需要手工设置的事项，见 `GITHUB_SETUP.md`。）
