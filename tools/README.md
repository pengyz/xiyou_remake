# tools/ — 工具链（入库）

| 工具 | 用途 | 阶段 |
|---|---|---|
| `extract-assets/` | JAR → `analysis/`、`assets/`（解包 + packed PNG 剥头 + .mid→.ogg 转换） | P0/P3 |
| `rename-pipeline/` | 语义命名管线（javap/调用图分析 → 台账 → 符号改名） | P1 |
| `trace-diff/` | 参考版 trace vs Rust trace 比对器 | P4 |
| `decompile.sh` | 从 original/ 一键再生 reference/seed（CFR pinned） | P1 |

原则：派生资产必须能从 `original/` 一键再生；工具自身变更必须跑门禁自测（AGENTS.md §2 gates 红线）。
