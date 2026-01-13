# GitHub 设置清单（需要你手工做）

这份清单专门放“代码里做不了/需要仓库管理员权限”的事情。

建议顺序：先把 CI 跑起来并稳定，然后再上分支保护与安全策略。

## 1) 基础仓库设置

在 GitHub 仓库页面：Settings → General

- Features
  - Issues：建议开启
  - Discussions：可选（如果你希望把“使用问题/想法”从 Issues 分流出去）

- Pull Requests
  - 勾选：Always suggest updating pull request branches
  - 勾选：Allow squash merging（并建议关闭 merge commit，保持线性历史）
  - 可选：Automatically delete head branches

## 2) 分支保护（Branch protection rules）

Settings → Branches → Add branch protection rule

以 `main` 为例，建议打开：
- Require a pull request before merging
  - Require approvals: 1~2（视团队而定）
  - Require review from Code Owners（如果你有 CODEOWNERS）
  - Dismiss stale PR approvals when new commits are pushed
- Require status checks to pass before merging
  - 把 CI 工作流里的关键 checks 设为 Required（例如：fmt/clippy/test/security audit/cargo-deny）
- Require conversation resolution before merging
- Require linear history
- (可选) Require signed commits

提示：required checks 的名字要与 workflow job 名对齐，避免未来重命名导致保护失效。

## 3) Actions 与发布权限

Settings → Actions

- General
  - 允许 GitHub Actions 运行（默认即可）
- Workflow permissions
  - 如果你的 release workflow 需要写 Release/Tag：选择 “Read and write permissions”
  - 勾选：Allow GitHub Actions to create and approve pull requests（如果你后续要自动化版本 bump/更新 changelog）

## 4) 安全设置（建议全开）

Settings → Security & analysis

建议开启：
- Dependency graph
- Dependabot alerts
- Dependabot security updates
- Secret scanning
- Secret scanning push protection（如果你的组织/计划支持）

如果你打算用 GitHub 的 Code scanning（可选）：
- Code scanning alerts
- 配置 CodeQL workflow（Rust 支持良好，但会增加 CI 时间）

## 5) Releases

如果你使用 `cargo-dist`（仓库内已有 release workflow）：
- 确认默认分支与 tag 触发规则符合你的发布策略（例如 `vx.y.z`）
- 发布前：检查 `CHANGELOG.md` 已更新
- 发布后：检查 assets、checksums 是否齐全

## 6) Issue/PR 模板与标签

建议你在仓库里添加（可由代码 PR 完成）：
- `.github/ISSUE_TEMPLATE/bug_report.yml`
- `.github/ISSUE_TEMPLATE/feature_request.yml`
- `.github/pull_request_template.md`

但这些“默认标签（labels）”通常需要你在 UI 里准备（或用脚本/terraform 管理）：
- `bug`, `enhancement`, `docs`, `good first issue`, `help wanted`, `breaking`, `security`, `target:codex`, `target:claude_code` ...

## 7) 安全漏洞上报入口

如果你希望用户通过 GitHub 的 Private vulnerability reporting：
- Settings → Security & analysis → Private vulnerability reporting（如果可用则开启）

并建议在根目录提供 `SECURITY.md`（说明披露流程与响应 SLA）。

## 8) 社区运营（可选）

如果你希望它成为“真正优秀的开源项目”，建议逐步开启：
- Discussions（Q&A / Ideas）
- GitHub Projects（Roadmap 可视化）
- Sponsors（如果你打算接受赞助）

注意：这些不影响核心质量，但能显著降低维护者的沟通成本。
