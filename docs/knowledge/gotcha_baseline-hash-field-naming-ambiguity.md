---
name: baseline-hash-field-naming-ambiguity
description: 基线/台账里"seed_sha256"之类的哈希字段名必须明确写清楚是哪个阶段的哪份文件，否则会被误读为锚点文件本身的哈希
type: gotcha
created: 2026-10-09
sources: ["2026-10-09 架构独立复核会话", "docs/reviews/2026-10-09-architecture-review.md#F2"]
---

`data/status/baselines/baseline.json` 里曾有字段 `decomp.seed_sha256`，字段名读起来像"锚点种子文件（`reference/seed/a.java`）的哈希"，但实测它存的是反编译**前**的 `a.class`（字节码）哈希——两者是反编译流水线里不同阶段的不同文件，哈希值也不同（`sha256sum reference/seed/a.java` 与 `unzip -p original/*.jar a.class | sha256sum` 结果不同）。

**为什么：** 反编译/转换类流水线里"同一个逻辑资产"会在多个阶段产生多份不同的文件（字节码 → 反编译产物 → 改名产物…），如果哈希字段名不显式标注"哪一阶段/哪份文件"，读者（人或 AI）会默认取最直觉的那份（通常是离锚点红线最近的那份），而真正发挥校验作用的门禁如果信任了这个字段名却读错了文件，会在"认为锚点受保护"的错觉下留下真实的无保护缺口（本例中 `reference/seed/a.java` 直到新增专用字段才被真正纳入门禁）。
**何时使用：** 给任何"多阶段产物"的哈希/校验字段命名时，字段名必须包含文件角色而非仅角色所在的流水线阶段名（例如 `seed_a_java_sha256` 而非 `seed_sha256`，`seed_source_class_sha256` 而非 `source_sha256`）；审查基线/台账 schema 时，对每个哈希字段反问"这个值是用什么命令算出来的、来自哪个具体文件路径"，并实际跑一次命令核对，而不是信任字段名的字面含义。
