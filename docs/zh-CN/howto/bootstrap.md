# Bootstrap（AI 自举 / Operator assets）

> Language: 简体中文 | [English](../../howto/bootstrap.md)

Bootstrap 的目标：把“会用 agentpack”这件事交给 AI 自己完成。

执行一次 bootstrap 后：
- Codex 会多一个 `agentpack-operator` skill，教它如何调用 `agentpack` CLI（优先 `--json`），以及推荐的工作流。
- Claude Code 会多一组 `/ap-*` slash commands，封装常用的 `doctor/update/preview/plan/diff/deploy/status/explain/evolve` 操作，并使用最小化的 `allowed-tools`。
- 可选（当启用 `targets.claude_code.options.write_*_skills` 时）：Claude Code 会多一个 `agentpack-operator` Skill，教它“什么时候该用 Agentpack”，并把执行落到 `/ap-*` 命令上（不隐式执行）。

## 1) 命令

`agentpack bootstrap [--scope user|project|both]`

- `--scope` 默认 `both`：同时写 user 与 project 位置
- 选择写哪些 target 用全局 `--target`：
  - `agentpack --target codex bootstrap`
  - `agentpack --target claude_code bootstrap`

## 2) 写入位置

- Codex：
  - user：`~/.codex/skills/agentpack-operator/SKILL.md`
  - project：`<project_root>/.codex/skills/agentpack-operator/SKILL.md`

- Claude Code：
  - user：`~/.claude/commands/ap-*.md`
  - project：`<project_root>/.claude/commands/ap-*.md`
  - user（可选）：`~/.claude/skills/agentpack-operator/SKILL.md`
  - project（可选）：`<project_root>/.claude/skills/agentpack-operator/SKILL.md`

这些文件也会被纳入 target manifest（`.agentpack.manifest.<target>.json`），因此：
- 可以被 `status` 检测
- 可以被 `rollback` 回滚
- 删除只会删托管文件

## 3) 版本标记与更新

Bootstrap 写入的模板会替换 `{{AGENTPACK_VERSION}}` 为当前 agentpack 版本号。

当你升级 agentpack 后，如果 `status` 提示 operator assets 过期：
- 直接重新运行 `agentpack bootstrap` 即可更新。

## 4) dry-run 与 --json

- 预览（不写入）：`agentpack bootstrap --dry-run --json`
- 应用（写入）：
  - human：`agentpack bootstrap`（会交互确认）
  - json：`agentpack --json bootstrap --yes`

说明：bootstrap 属于写入类命令；在 `--json` 下必须显式 `--yes`，否则返回 `E_CONFIRM_REQUIRED`。

## 5) 自定义 operator assets（可选）

Bootstrap 使用内置模板（随版本更新）：
- `templates/codex/skills/agentpack-operator/SKILL.md`
- `templates/claude/commands/ap-*.md`
- `templates/claude/skills/agentpack-operator/SKILL.md`

如果你希望完全自定义：
- 你可以把这些内容做成普通 module（`skill`/`command`），由 manifest 管理；
- 或者在 bootstrap 后用 overlays 覆盖模板写入的文件（更推荐作为“你自己的版本”沉淀进 config repo）。

## 6) Claude Code `allowed-tools`

Bootstrap 写入的 Claude Code slash commands 会在 YAML frontmatter 中声明 `allowed-tools`，用于限制工具权限。

设计原则：
 - 尽量只声明该命令所需的最小 `Bash("...")` 集合。
- 保持命令单一职责（例如 `/ap-plan` 只允许 `agentpack plan --json`）。
- 写入类命令应显式体现批准语义（`--yes --json`），并配合额外护栏（见下一节）。

## 7) Claude Code 写入类命令的安全性

对于会产生写入的 Claude Code operator commands（例如 `/ap-update`、`/ap-deploy`、`/ap-evolve`），模板会带上：
- `disable-model-invocation: true`

这样可以降低模型在未显式请求时程序化触发写入操作的风险；同时 Agentpack 仍会在 `--json` 模式下对写入命令强制要求 `--yes`，作为最后一道护栏。
