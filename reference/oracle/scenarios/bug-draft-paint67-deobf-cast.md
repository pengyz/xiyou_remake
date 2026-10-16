# BUG 新条目稿：deobf 投影型 67 拼装臂 byte[]→Object[] 强转（task-10 移交 lead 入库；本文件可删）

> 提交者：items-evidence（task-10 FX-BOSS-67A 夹具铺设时发现）。建议按 BUG-010 编号入库
> （以 ledger 实际序号为准）。

## 条目文稿（建议入 docs/bug-ledger.md 表格）

| 列 | 内容 |
|---|---|
| 现象 | **C 变体（deobf 重编译）画到型 67 即崩**：`ClassCastException: class [B cannot be cast to class [Ljava.lang.Object;` at `paintEntityLayer(a.java:6592)` ← `paint(a.java:2388)` ← `serviceRepaints` ← `run(a.java:4280)`；A（原 jar）/B（rename）两变体同场景正常且逐字节一致 |
| 复现 | `python3 tools/frames_fixture.py gen crates/game-oracle/tests/fixtures/fx-boss67a.txt`（floor15 出生 (6,6)，(6,5) 型67 在屏即触发首帧 paint）——C trace T236 尾 `EVT thread-exit / END all-threads-exited`，A trace 同拍继续 |
| 根因 | deobf 投影 **a.java:6588-6601**（T-2 型67 拼装臂）把片件表 `f_byte_arr2_00`（`byte[][]`）的行读成 `Object[]`（`(Object[])(var45 = var23.f_byte_arr2_00[var42[var59]])` 再取 [0]..[4]）——byte[] 行无法在运行期强转 Object[]；**原字节码的真实取值语义必然不同**（反编译类型投影失真，行文本逐字≠语义逐真） |
| 影响 | ① C 变体不可参与型 67 在屏场景的 A/B/C 三方对拍（fx-boss67a 夹具以 **A==B** 双变体生成，帧链 sha 记 frames.txt 头）；② **T-2 移植的取证源被污染**：adbdee0 的 Rust 拼装臂系按该文本实现——本场景 A==B 帧链与 Rust 全帧对拍 **560/560 一致**（渲染层侥幸或实现者自行纠偏，像素级等价成立），但类型语义合同仍应按字节码复核（javap）后在 spec 原地更正 |
| 保真/处置 | Stage A：行为以 A 帧为准（已钉夹具）；deobf 源 6588-6601 建议按 javap 字节码更正投影并注明日期（§3.6）；C 变体重建后 fx-boss67a 应回归三方 |
| 状态 | open |

## 复现锚点

- 夹具：`crates/game-oracle/tests/fixtures/fx-boss67a.*`（frames.txt 头注释记 A==B 双方 sha256=f9e024c5…）
- 测试：`fx_boss67a_kill_spawn_frames_match`（crates/game-core…/game-oracle/tests/fixture_frames.rs）
- C 崩溃栈实录：analysis/frames-fixture/runs/C-fx-boss67a/stderr.txt（一次性工作区，可由 gen 复现）
