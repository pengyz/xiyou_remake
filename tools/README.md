# tools/ — 工具链（入库）

| 工具 | 用途 | 阶段 |
|---|---|---|
| `extract-assets/` | JAR → `analysis/`、`assets/`（解包 + packed PNG 剥头 + .mid→.ogg 转换） | P0/P3 |
| `rename-pipeline/` | 反混淆/命名管线（常量池手术重映射 + javap 等价验证 → 台账 → 符号改名） | P1 |
| `trace-diff/` | 参考版 trace vs Rust trace 比对器 | P4 |
| `decompile.py` | 从 original/ 一键再生反编译投影并与 `reference/seed/` 比对（CFR pinned + sha256 校验） | P1 |
| `decompile.sh` | `decompile.py` 的 POSIX 薄包装（Windows 直接用 `python3 tools/decompile.py`） | P1 |
| `cfr_env.py` | CFR 0.152 定位/完整性校验/获取的单一真相源（vendor → 缓存 → urllib 下载） | P1 |
| `vendor/` | 入库的第三方依赖（`cfr-0.152.jar`，sha256 钉死，全新 clone 离线可用） | P1 |

原则：派生资产必须能从 `original/` 一键再生；工具自身变更必须跑门禁自测（AGENTS.md §2 gates 红线）。
跨平台约定：逻辑放 `.py`（纯标准库，不依赖 curl/unzip/diff/sha256sum），`.sh` 只做薄包装；
第三方 jar 作为依赖 vendor 入库并 sha256 钉死（见 `cfr_env.py`）。
