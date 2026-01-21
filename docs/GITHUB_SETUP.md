# GitHub setup checklist (manual / admin-only)

This checklist captures repository settings that cannot be done from code, or require admin privileges.

Recommended order: get CI green and stable first, then enable branch protection and security hardening.

## 1) Basic repo settings

GitHub repo page: Settings → General

- About (repo homepage sidebar)
  - Description (EN): A declarative, safe control plane for deploying coding-agent assets across tools.
  - Description (ZH): 面向 AI 编程代理的本地资产控制面：声明式管理与安全部署 AGENTS/Skills/Commands/Prompts。
  - Topics (suggested): ai, agent, codex, claude-code, mcp, cli, rust, developer-tools
  - Website (optional): https://crates.io/crates/agentpack

- Features
  - Issues: recommended
  - Discussions: optional (useful if you want to offload “usage questions/ideas” from Issues)

- Pull Requests
  - Enable: Always suggest updating pull request branches
  - Enable: Allow squash merging (and consider disabling merge commits to keep linear history)
  - Optional: Automatically delete head branches

## 2) Branch protection rules

Settings → Branches → Add branch protection rule

For `main`, recommended:
- Require a pull request before merging
  - Require approvals: 1–2 (team dependent)
  - Require review from Code Owners (if you use CODEOWNERS)
  - Dismiss stale PR approvals when new commits are pushed
- Require status checks to pass before merging
  - Mark key CI checks as Required (e.g. fmt/clippy/test/security audit/cargo-deny)
- Require conversation resolution before merging
- Require linear history
- (Optional) Require signed commits

Tip: Required check names must match workflow job names; renaming jobs can silently weaken protection if you don’t update the rule.

## 3) Actions and release permissions

Settings → Actions

- General
  - Allow GitHub Actions to run (default is fine)
- Workflow permissions
  - If your release workflow needs to create Releases/Tags: choose “Read and write permissions”
  - Optional: enable “Allow GitHub Actions to create and approve pull requests” (useful for automated version bump/changelog PRs)

## 4) Security & analysis (recommended to enable)

Settings → Security & analysis

Recommended:
- Dependency graph
- Dependabot alerts
- Dependabot security updates
- Secret scanning
- Secret scanning push protection (if available for your org/plan)

If you plan to use GitHub Code Scanning (optional):
- Code scanning alerts
- Set up a CodeQL workflow (Rust support is solid, but it increases CI time)

## 5) Releases

If you use `cargo-dist` (this repo ships a release workflow):
- Verify the default branch and tag trigger rules match your release strategy (e.g. `vX.Y.Z`)
- Before releasing: ensure `CHANGELOG.md` is updated
- After releasing: verify assets and checksums are present

## 6) Issue/PR templates and labels

Recommended templates (can be added via PR):
- `.github/ISSUE_TEMPLATE/bug_report.yml`
- `.github/ISSUE_TEMPLATE/feature_request.yml`
- `.github/pull_request_template.md`

Default labels usually need to be created in the UI (or managed via scripts/terraform), for example:
- `bug`, `enhancement`, `docs`, `good first issue`, `help wanted`, `breaking`, `security`, `target:codex`, `target:claude_code`, ...

## 7) Vulnerability reporting

If you want GitHub’s Private vulnerability reporting:
- Settings → Security & analysis → Private vulnerability reporting (enable if available)

Also consider adding a top-level `SECURITY.md` describing the disclosure process and response SLA.

## 8) Community ops (optional)

If you want to invest in long-term OSS hygiene, consider enabling over time:
- Discussions (Q&A / Ideas)
- GitHub Projects (roadmap visualization)
- Sponsors (if you want to accept sponsorship)

These don’t affect core quality, but they can significantly reduce maintainer communication overhead.
