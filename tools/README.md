# tools/ — 工具链（入库）

| 工具 | 用途 | 阶段 |
|---|---|---|
| `extract-assets/` | JAR → `analysis/`、`assets/`（解包 + packed PNG 剥头 + .mid→.ogg 转换） | P0/P3 |
| `rename-pipeline/` | 反混淆/命名管线（常量池手术重映射 + Vineflower 再生 + 投影修复补丁 + javac 可编译门禁 + javap 等价验证） | P1 |
| `trace-diff/` | 参考版 trace vs Rust trace 比对器 | P4 |
| `vf_env.py` | Vineflower 1.12.0 定位/完整性校验/获取的单一真相源（vendor → urllib 下载） | P1 |
| `vendor/` | 入库的第三方依赖（`vineflower-1.12.0.jar`，sha256 钉死，全新 clone 离线可用） | P1 |

原则：派生资产必须能从 `original/` 一键再生；工具自身变更必须跑门禁自测（AGENTS.md §2 gates 红线）。
跨平台约定：逻辑放 `.py`（纯标准库，不依赖 curl/unzip/diff/sha256sum），`.sh` 只做薄包装；
第三方 jar 作为依赖 vendor 入库并 sha256 钉死（见 `vf_env.py`）。

> 【2026-10-10 更正】CFR 0.152 工具链（`cfr_env.py`/`decompile.py`/`decompile.sh`/
> `vendor/cfr-0.152.jar`）已随反编译引擎切换 Vineflower 而退役移除——决策与实测数据见
> [docs/knowledge/decision_decompiler-cfr-to-vineflower.md](../docs/knowledge/decision_decompiler-cfr-to-vineflower.md)。
> `reference/seed/a.java`（CFR 历史投影冻结件）保留不动，溯源注记见
> [docs/knowledge/reference_cfr_decompiler.md](../docs/knowledge/reference_cfr_decompiler.md)。
