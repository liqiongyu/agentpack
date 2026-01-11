# TARGET_CONFORMANCE.md (v0.4 draft)

v0.4 的质量护城河：任何 target 都必须通过 conformance tests。

## 必须覆盖的语义
1) 删除保护：plan 只能删除 manifest managed files
2) apply 必写 per-root manifest
3) status 能区分 missing/modified/extra（extra 不触发删除）
4) rollback 可恢复 create/update/delete
5) JSON envelope schema_version 与错误码一致

## 推荐做法
- 使用临时目录模拟 target roots（fixture）
- 通过真实管道跑 plan/apply/status
- 用 golden 输出固定住行为（尤其 plan 排序与 provenance 字段）
