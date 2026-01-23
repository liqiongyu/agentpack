# Demo：5 分钟看到价值（安全，不写真实环境）

> Language: 简体中文 | [English](../../tutorials/demo-5min.md)

目标：跑一个安全 demo，让你在不污染真实 HOME 的前提下，看到一次真实的 plan/diff。

## 运行

在仓库根目录执行：

```bash
./scripts/demo_5min.sh
```

Windows（PowerShell）下：

```powershell
pwsh -NoProfile -File .\\scripts\\demo_5min.ps1
```

脚本会按优先级选择运行方式：
- 如果设置了 `AGENTPACK_BIN`（一个可执行文件路径），优先用它
- 否则用 PATH 里的 `agentpack`
- 否则（如果已安装 Rust）使用本仓库的 `cargo run`

## 它做了什么

- 创建临时 `HOME` 与临时 `AGENTPACK_HOME`
- 把 `docs/examples/minimal_repo` 复制到临时 workspace
- 运行：
  - `agentpack doctor --json`
  - `agentpack preview --diff --json`

脚本不会使用 `--apply`，因此不会写入任何 target 文件；即便解析到了 target 路径，也是在临时 `HOME` 里。

## 下一步

- 建一个你自己的 config repo：
  - `agentpack init`
  - `agentpack update`
  - `agentpack preview --diff`
- 真正写入（显式确认）：
  - `agentpack deploy --apply --yes`
- 需要撤销时：
  - `agentpack rollback --to <snapshot_id>`
