# BUG-012 累积行全量转储实录：满 16 行的生成机制（task-20）

> 2026-10-16。StateDump 探针：`-Doracle.dumpFull=true`（本批新增，
> reference/oracle/src/oracle/host/StateDump.java FULL_DUMP——默认行为逐字节不变）。
> 场景：`crates/game-oracle/tests/fixtures/fx-lay2a.txt`（floor3 script0 链）。
> 复现：`java -cp reference/oracle/_out/classes:analysis/frames-fixture/variantA:original/囧囧西游-大闹天宫.jar
> -Doracle.out=… -Doracle.max=1100 -Doracle.script=crates/game-oracle/tests/fixtures/fx-lay2a.txt
> -Doracle.preset=crates/game-oracle/tests/fixtures/fx-lay2a.preset -Doracle.dumpStride=1
> -Doracle.dumpFull=true -Doracle.frames=none oracle.host.Runner`（T350 AIOOBE 复现）。

## 结论（BUG-012 机制 B 级候选 → A 级实证，附两处更正）

**满 16 的行 = cellSlotCount[36]，格 (6,11) 的槽行；累积物 = script0 重入循环每 3 拍
新生成的一只 type-84 嫦娥（entityCount 35→51，槽值 36..51 连续不同）——16 圈填满，
第 17 圈 CES_84_6_11 的注册 `cellSlotTable[36][16]` 越界即 T350 崩溃。**

### 更正 1：崩溃指令归属——不是 CES_70_5_8，是第 17 圈 CES_84_6_11

原记载"CES_70_5_8 执行时 AIOOBE"。全量转储时间线：末圈 T347 cursor 0→12（CES_84）、
T348 12→23（MOV）、T349 23→0（重入）、**T350 0→12 = 第 17 次 CES_84_6_11 执行**，
其 spawnEntity 注册落 (6,11) 行 [16] 即崩（C 栈 `spawnEntity←executeScriptInstruction`
的 CES 臂与两解都相符；行归属 [36]=格 (6,11) 排除 CES_70（(5,8) 另有其行））。

### 更正 2：机制候选换马——不是 m_108 槽迁移重复追加

原 B 级候选"m_108 重复追加无 count<16 钳制"。全量转储显示槽值是 **36..51 连续不同值**
（每圈一个新实体的 (byte)entityCount），不是同值重复 ⇒ m_108 重复登记是另一个独立 quirk
（entity_golden relocate_slot 测试已记），与 BUG-012 填充无关。

## A 级证据（全量转储，FLD 索引经 fields.py 核定）

**1. 截断误读源确认**：cellSlotCount=FLD 136（byte[128]，旧 head=16）、
cellSlotTable=FLD 137（byte[128][16]，旧 32 行）——旧行 16..127 从未打印，
"槽计数全行 ≤1"的反常即此（BUG-012 重审段已述）。dumpFull 后 T349 末态：

```
cellSlotCount = {1..35: 各 1, 36: 16}          ← 唯一满行 = 36
cellTypeGrid[11][6] = 36                       ← 行 36 = 格 (6,11)
entityPixel[35..50] = 全部 (192,352) = 格 (6,11) ← 16 只嫦娥叠一格
```

**2. 累积时间线（cellSlotCount[36] / cellSlotTable[36] 行内容）**：

```
T302: [36,…]                    第1只（CES_84_6_11 圈1，entity 35）
T305: [36,37,…]                 第2只（圈2，entity 36）
T308: [36,37,38,…]              第3只
… 每 3 拍一只 …
T347: [36,37,…,50]              第16只（entity 50，行满 16）
T350: 第17只注册 [36][16] → ArrayIndexOutOfBoundsException: Index 16 of length 16
```

**3. 重入循环机制（3 拍一圈，scriptCursor/dialogPhase 时间线 + a.java 行指针）**：

```
圈N T+0: cursor 0→12   CES_84_6_11 → spawnEntity(84,6<<5,11<<5)（行 36 追加 1）
圈N T+1: cursor 12→23  MOV_0_5_11 → findPath 4 步队列 + dialogPhase=3（a.java:8310-8314）
圈N T+2: cursor 23→0   dialogPhase 3→0  ← loadLevelScript(0) 重入（a.java:8053 scriptCursor=0）
```

重入触发链（A 级行指针）：

- 玩家 (2,10) 朝 y+ 撞 (2,11) 剧情点（sprite3 param=0）→ interactWithCell case 83 bump 臂
  `tryRunScene(param, false)`（a.java:5466）；
- tryRunScene `case 0` 门**不看 from_walk**（a.java:8076-8080 仅 `if (var3) flags[19]=true; break;`）
  ⇒ bump 也返回 true；
- loadLevelScript(0) **返回 false**（a.java:8072-8075 `default: return false`）⇒ interact 返回 false
  ⇒ **step 被挡、玩家钉在 (2,10)**（全 run playerCell 恒 (2,10)）——但脚本已从头重跑；
- 每圈 MOV 的 findPath 重出 4 步队列、phase3 只消费 1 步即再撞剧情点 ⇒ 死循环式重入。

**4. 关键刻度**：脚本行 `0 CES_84_6_11 MOV_0_5_11 TAK_8_9 CES_70_5_8 …`（assets/raw/script:17）；
重入使 TAK_8_9/CES_70_5_8 在 T350 前从未执行（cursor 每圈在 23 处归零）。

## 对台账/测试的连带提示（lead 翻格用）

- BUG-012 "机制 B 级候选=m_108 重复追加" → 换马为"script0 重入循环 × CES_84 逐圈注册"（A 级）；
- "CES_70 崩溃"归属 → "第 17 圈 CES_84_6_11 崩溃"（CES_70 未执行）；FX-LAY-2A 夹具的
  "崩溃前缀 349 帧"体例不变，但崩溃锚点语义随此更正；
- Stage B 修复面（原"槽登记钳制"）之外应叠加：tryRunScene case 0 门缺 from_walk 检查 /
  loadLevelScript 返回值语义导致的剧情点重入循环——这才是可修的上游机制（原版 bug，D4 保真期不动）。
