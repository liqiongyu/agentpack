# SECURITY_CHECKS.md

本仓库在 CI 中默认启用两类依赖/安全检查：

## 1) RustSec 漏洞扫描（advisories）

- CI: `.github/workflows/ci.yml` 的 `Security audit` job（`rustsec/audit-check`）
- 本地（可选）：安装后运行 `cargo audit`
  - 安装：`cargo install cargo-audit --locked`

失败处理建议：
- **优先**升级/替换依赖以消除漏洞。
- 若必须临时忽略（不推荐）：需要在 PR/Issue 中写明原因、影响范围、以及移除 ignore 的计划。

## 2) cargo-deny 依赖策略检查（licenses / bans / sources）

- CI: `.github/workflows/ci.yml` 的 `Dependency policy (cargo-deny)` job
- 配置：`deny.toml`
- 当前 CI 运行的 checks：
  - `licenses`：许可证允许列表与例外
  - `bans`：重复依赖版本与通配版本约束（`*`）
  - `sources`：依赖来源（仅允许 crates.io；禁止未知 registry/git）

本地运行：
- `cargo install cargo-deny@0.18.3 --locked`
- `cargo deny check licenses bans sources`

失败处理建议：
- **licenses**：
  - 新增的许可证若可接受：将 SPDX 标识加入 `deny.toml` 的 `licenses.allow`
  - 若仅某个 crate 需要例外：使用 `licenses.exceptions`（并写清楚 reason/issue）
- **sources**：
  - 尽量避免 git 依赖；若确实必要，显式加入 `sources.allow-git`（并写清楚 reason/issue）
- **bans**：
  - `wildcards`（`*`）被拒绝：将依赖版本固定到明确的 semver 约束
  - `multiple-versions` 当前为 warning：可逐步通过 `cargo update -p <crate>` / 依赖升级来收敛
