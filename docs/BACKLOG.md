# BACKLOG.md

> Current as of **v0.5.0** (2026-01-13). Historical content is tracked in git history.

## Status

- v5 milestone: 完成一轮“可日用 + AI-first 闭环”的收敛（组合命令、overlay rebase、adopt 保护、evolve restore 等）。
- 具体变更请看 `CHANGELOG.md`。

## Next（v0.6+ 候选）

### Targets & ecosystem
- 新增 targets（Cursor / VS Code 等），要求：TargetAdapter + conformance tests 作为合并门槛。
- 为每个新 target 提供：映射规则、examples、migration notes。

### UX & ergonomics
- 更强的 `status` 输出（可选 summary、按 root 分组、可建议动作）。
- 更丰富但仍可脚本化的 warnings（尽量带可操作建议与命令）。
- 考虑轻量 TUI（浏览 plan/diff/status/snapshots），但保持核心可在非交互模式运行。

### Overlays & evolve
- patch-based overlays（可选）：让少量文本改动更易 merge、冲突更可读。
- evolve propose 覆盖面增强：更好的多模块聚合输出回溯（除了 AGENTS.md），并提升 skipped reasons 的结构化解释。
- 为 evolve 输出更明确的“下一步命令”（适合 operator assets 引导）。

### 工程化
- CLI golden tests（JSON 输出/错误码的回归测试）。
- 更强的 conformance harness（临时 roots、跨平台路径用例）。
- 文档持续收敛（本轮已移除 `docs/versions/`，后续靠 git 历史承载迭代）。
