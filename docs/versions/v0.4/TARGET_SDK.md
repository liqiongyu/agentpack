# TARGET_SDK.md (v0.4 draft)

目标：让新增 target 的贡献者“照着做就行”。

## 1) 新增 target 的最小 checklist
1. 定义 target id（例如 cursor）
2. 定义 roots（哪些目录算一个 managed root；每个 root 都写 manifest）
3. 定义 mapping（modules -> paths）
4. 最小 validate（结构/必需字段校验）
5. 通过 conformance tests

## 2) 贡献者应该避免的坑
- 不要扫描用户目录并删除“非托管文件”
- 不要依赖 symlink（默认 copy/render）
- target-specific 的覆盖/优先级语义必须写进 docs
