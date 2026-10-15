# tools/ — 工具链（入库）

| 工具 | 用途 | 阶段 |
|---|---|---|
| `extract-assets/` | JAR → `analysis/`、`assets/`（解包 + packed PNG 剥头） | P0/P3 |
| `audio/` | BGM 离线转换 `.mid → .ogg`（rustysynth + 入库 SoundFont + rusty_vorbis，独立 cargo workspace）+ 选轨语义 oracle 取证（`evidence.py`）；见 [audio/README.md](audio/README.md) | P3 |
| `rename-pipeline/` | 反混淆/命名管线（常量池手术重映射 + Vineflower 再生 + 投影修复补丁 + javac 可编译门禁 + javap 等价验证） | P1 |
| `trace-diff/` | 参考版 trace vs Rust trace 比对器 | P4 |
| `vf_env.py` | Vineflower 1.12.0 定位/完整性校验/获取的单一真相源（vendor → urllib 下载） | P1 |
| `method_reach.py` + `method_reach/` | 参考版方法触达实测：JDI 一次性断点（`MethodReach.java`）测每场景每方法首次命中 tick，FRAME 链对照 A-trace（夹具场景对照 `.frames.txt`）；产物 `method_reach/snapshot.json`（入库）；`--check` 判陈旧、`--rev REV --only 场景` 历史版场景对比（不入快照） | P4 |
| `cover_matrix.py` | 覆盖矩阵（层次一账本）：javap 方法全集 × 触达快照 × Rust 移植账 → `docs/coverage-matrix.md`；`--check` 账面一致校验；判据见 [docs/spec/completeness.md](../docs/spec/completeness.md) §3.4 | P4 |
| `dead_code_ruling.py` | 白项 b 类死代码字节码机械复核：两 class 的 CFG + 字段值域不动点 + MVS 数据域引理 → 证死方法全集与逐调用点切断边；`--check` 退出码判裁决成立；裁决书 [docs/spec/dead-code-rulings.md](../docs/spec/dead-code-rulings.md) | P4 |
| `frames_fixture.py` | 帧夹具管线：`extract`（既有 A-trace → `<stem>.frames.txt`）/ `gen`（夹具场景在隔离工作区 `analysis/frames-fixture/` 跑 A/B/C，三方一致才抽帧）/ `check`（夹具自洽）/ `preset`（RMS 预置构造） | P4 |
| `vendor/` | 入库的第三方依赖（`vineflower-1.12.0.jar`、`TimGM6mb.sf2` + 许可全文，sha256 钉死，全新 clone 离线可用） | P1/P3 |

> 【2026-10-14 更正】`extract-assets/` 行原写「+ .mid→.ogg 转换」——该转换从未在 extract
> 中实现（其 docstring 自注「未实现」）；现由 `audio/` 承担。

原则：派生资产必须能从 `original/` 一键再生；工具自身变更必须跑门禁自测（AGENTS.md §2 gates 红线）。
跨平台约定：逻辑放 `.py`（纯标准库，不依赖 curl/unzip/diff/sha256sum），`.sh` 只做薄包装；
第三方 jar 作为依赖 vendor 入库并 sha256 钉死（见 `vf_env.py`）。

> 【2026-10-10 更正】CFR 0.152 工具链（`cfr_env.py`/`decompile.py`/`decompile.sh`/
> `vendor/cfr-0.152.jar`）已随反编译引擎切换 Vineflower 而退役移除——决策与实测数据见
> [docs/knowledge/decision_decompiler-cfr-to-vineflower.md](../docs/knowledge/decision_decompiler-cfr-to-vineflower.md)。
> `reference/seed/a.java`（CFR 历史投影冻结件）保留不动，溯源注记见
> [docs/knowledge/reference_cfr_decompiler.md](../docs/knowledge/reference_cfr_decompiler.md)。
