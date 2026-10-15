---
name: provision-trace-labels-a-stem-only
description: crates/ 里 `join("reference/oracle/_out/<label>/trace.txt")` 会被 provision_tests 扫描并要求 label=A-<场景名>；引用 EV-* 取证 trace 会让 rust-tests 门禁在缺夹具环境（CI）直接 FAIL
type: gotcha
created: 2026-10-15
sources: ["S2 壳集成测试设计（reference/oracle/provision_tests.py REF_RE / scenario_for）"]
---

`reference/oracle/provision_tests.py` 用正则 `join\("reference/oracle/_out/([A-Za-z0-9_-]+)/trace\.txt"\)`
扫描 `crates/**.rs`，对缺失的 trace 按约定 `A-<stem>` ↔ `scenarios/<stem>.txt` 再生；label 不以
`A-` 开头（如取证用的 `EV-hold-walk`）或无对应场景脚本 ⇒ `SystemExit` ⇒ rust-tests 门禁 FAIL。
本地 `_out/` 恰好存在时不会暴露，CI 全新 checkout 必炸。

**为什么：** trace 不入库，门禁靠「代码里出现的路径」反推需要再生哪些夹具；取证脚本
（tools/audio/evidence.py 的 `EV-*`）不在场景套件里，供给器无从再生。

**何时使用：** 新测试要对照 oracle 取证 trace 时二选一：①把期望值（位置/帧哈希/EVT 拍号）
**嵌入常量**并在注释写再生命令（`hold_walk_and_bgm_match_java_trace` 的做法）；②把输入脚本
正式加入 `reference/oracle/scenarios/<stem>.txt` 并以 `A-<stem>` 引用（会进 `--scenarios` 套件，
需带 `# ticks:`/`# preset:` 头）。不要在 crates 里 join EV-* 路径。
