# BUG 新条目稿：script0 CES_70 出生崩溃（spawnEntity 槽登记越界）+ FX-LAY-2A 受阻论证（task-10 移交 lead 入库；本文件可删）

> 提交者：items-evidence（FX-LAY-2A 夹具铺设时发现）。建议按 ledger 实际序号入库。
> 这是继 BUG-009 边界出生、paint67 deobf 投影后的**第三例**原版崩溃族。

## 条目文稿（建议入 docs/bug-ledger.md 表格）

| 列 | 内容 |
|---|---|
| 现象 | floor3 scene0 链对话推进至 `CES_70_5_8` 执行时 **T350 AIOOBE: Index 16 out of bounds for length 16**，主线程退出（trace 尾 `EVT thread-exit / END reason=all-threads-exited ticks=349`）。A/B/C 三变体同 tick 同点同因崩溃（三方 trace sha256 含崩溃尾一致） |
| 复现 | `python3 tools/frames_fixture.py gen crates/game-oracle/tests/fixtures/fx-lay2a.txt`（floor3 (2,10) 南步入 (2,11) 触发 script0 → 对话推进）；**对照**：`fx-scriptwalk-floor3`（同链、不推进对话）不崩——差异 = 是否走完 TAK_8_9 到达 CES_70 |
| 根因（部分钉死） | C 变体可读栈：`spawnEntity(a.java:6418) ← executeScriptInstruction(a.java:8401)`；javap 行表复核 line 6418 = 槽登记语句 `this.cellSlotTable[var6][this.cellSlotCount[var6]] = (byte)this.entityCount;`（cellSlotTable = `byte[128][16]`，a.java:6966-6967 initCellGrid）⇒ **列下标 16 = 某行槽计数满 16**。**未解反常**：崩溃前 T349 全量 FLD 转储显示 cellSlotCount 全行 ≤1（35+1 实体各占独立格行，nextCellTypeId=36），与"某行满 16"矛盾——真实触发条件需深挖（疑与 CES 变宽 sprite 多格登记/字节符号扩展/行复用有关，**不下结论**） |
| 影响 | ① **FX-LAY-2A 受阻（A 级理由）**：script0 的 LAY_2/RES_0 语料唯一点（68 条脚本 `LAY_`/`RES_` 各 1 处）位于 CES_70 **之后**——Java 侧不可达，帧差分无法触达 LAY/RES 臂（Rust 侧已移植且有单测，但无 Java 对拍）；② GUT 臂不受影响（语料 7 处，他链可达）；③ 正常游玩可达性：floor3 (2,11) 为普通格 + fx-scriptwalk 已证 bump 可触发 ⇒ 原版推进 floor3 剧情对话即触发——**疑原版 floor3 剧情链在真机同样崩溃**（需真机证词，B 级保留） |
| 保真/处置 | Stage A：崩溃即原版语义（D4）；FX-LAY-2A 夹具建议裁法二选一：(a) 钉崩溃前缀（349 帧 + 崩溃锚点，同 FX-BUG-AIOOBE 体例）+ LAY/RES 账本行按"Java 不可达-原版崩溃"终裁；(b) 若 closure-typist 不接受终裁，维持 ⬜ 并挂本 BUG 指针。Stage B 修复 = 槽登记钳制（cellSlotCount<16 检查）或扩容 + 根因深挖后定 |
| 状态 | open（根因深挖 pending） |

## 复现锚点

- 夹具草案：`crates/game-oracle/tests/fixtures/fx-lay2a.{txt,preset}`（frames.txt 未生成——gen 因崩溃尾 tick 数不符拒写；A/B/C trace 存 analysis/frames-fixture/runs/{A,B,C}-fx-lay2a/）
- 仪器化复现（逐 tick 全量转储）：`java -cp analysis/frames-fixture/classes:analysis/frames-fixture/variantA:original/囧囧西游-大闹天宫.jar -Doracle.dumpStride=1 …`（见本文件同批 commit message）
- 对照夹具：`crates/game-oracle/tests/fixtures/fx-scriptwalk-floor3.*`（同链不崩，TAK 停留）
