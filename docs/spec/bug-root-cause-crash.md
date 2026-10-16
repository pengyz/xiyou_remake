# 崩溃判例根因重审：BUG-009 / BUG-012 三层怀疑顺序定性

> 角色：porter-arms（task-16）。框架：**用户裁定怀疑顺序 ① shim 层与真机不一致 →
> ② a.class→a.java 投影引入 → ③ 原版 bug**（"原版"是最后手段）。
> 方法：javap 指令级复核（`analysis/rename-pipeline/verify/a.before.javap.txt`
> = 原 jar class a 反汇编）+ FLD 转储复核 + A/B/C 三变体崩溃栈对照。
> 证据分级：本文行级结论均 **A 级**（偏移+指令逐字）；推断处显式标 B 级。
> 取证日期：2026-10-16。配套：[bug-ledger.md](../bug-ledger.md) BUG-009/012、
> [evidence-todos.md](evidence-todos.md)、reference/oracle/scenarios/bug-draft-*.md。

## 0. 结论速查

| 判例 | ① shim | ② 投影 | ③ 原版 | 重定性 |
|---|---|---|---|---|
| BUG-009 边界出生 AIOOBE Index -1/13 | 排除（JVM 标准越界语义，游戏自有数组） | **排除**（"先调后判"系原字节码实况，投影忠实） | **坐实（A 级，指令级）** | 维持"原版 bug"——且比原判更强：型 60 块先调后判是**原字节码复制粘贴不对称**（型 61/62 有守卫） |
| BUG-012 CES_70 槽越界 Index 16 | 排除（同上；触发链=语料脚本+sprite 数据） | **排除**（分配/登记/迁移三处指令级逐一核对忠实，含 byte 截断语义） | **坐实（A 级，崩溃存在性：三变体同栈）**；机制 B 级候选=槽迁移重复追加累积 | 维持"原版 bug（D4 保真）"；**"未解反常"撤销**——是探针截断伪影（§3.2），非矛盾 |

## 1. 方法与证据基

- **字节码**：`analysis/rename-pipeline/verify/a.before.javap.txt`（原 jar `a.class`
  的 javap -c -p 反汇编，rename-pipeline --verify 可再生）。
- **转储**：oracle 逐 tick FLD 状态转储（`analysis/frames-fixture/runs/{A,B,C}-fx-lay2a/trace.txt`）。
- **三变体**：A=原 jar 字节码 / B=机械改名 / C=deobf 重编译；三者同 tick 同栈崩溃
  ⇒ 崩溃存在性与投影无关（对②的最强排除）。
- 复现：
  ```bash
  python3 tools/frames_fixture.py gen crates/game-oracle/tests/fixtures/fx-lay2a.txt   # BUG-012
  python3 tools/frames_fixture.py gen crates/game-oracle/tests/fixtures/fx-bug-aioobe-border.txt  # BUG-009(y=12)
  # 字节码复核（本页全部偏移）：
  grep -n "Method b:(III)I\|Field d:\[\[B\|multianewarray" analysis/rename-pipeline/verify/a.before.javap.txt
  ```

## 2. BUG-009：边界出生 AIOOBE（Index -1 / Index 13，length 13）

### 2.1 字节码实况（applyStepCellEffects，obf `private void k()`）

崩溃链：`findBlockingEntity`（obf `b(III)I`）内**无边界检查**的
`cellTypeGrid[y][x]` 越界读 ← 型 60 四邻扫描**先调后判**：

```
型60 块（x-1 方向，dump 偏移 384-409）——**CALL BEFORE CHECK**：
  384: aload 8
  386: iload 9              ← x
  388: iconst_1
  389: isub                 ← x-1
  390: iload 10             ← y
  392: bipush 60
  394: invokespecial b:(III)I   ← findBlockingEntity(x-1, y, 60)  ← 先调
  397: istore 7
  399: iload 9
  401: ifle 412             ← if (x <= 0) skip                   ← 后判！
  404: iload 7
  406: iflt 412
  409: iinc 12, 1
```

同块 y-1（447-464）、x+1（412-436）、y+1（488-512）全部同构：**四次调用均无前置守卫**。
x=0 ⇒ `cellTypeGrid[y][-1]` ⇒ **Index -1 of length 13**；y=12 ⇒ `cellTypeGrid[13][x]`
⇒ **Index 13 of length 13**——与 BUG-009 双子例现象逐字吻合（preset-alchemy floor4 (5,0)
原现象；fx-bug-aioobe-border floor5 (9,12) 复现）。

**对照组（同方法内另两块是先判后调的短路守卫）**：

```
型61 块（dump 88-184）：88: iload 9; 90: ifle 141        ← 先判 x>0
                       93-103: b(x-1,y,61)              ← 后调
                       109-118: x < Y-1 检查 → 121-131: b(x+1,y,61)
                       141-143: ifle 194（y>0）→ 146-156: b(x,y-1,61)
型62 块（dump 228-341）：231-233: ifle 255（x>0）→ 236-246: b(x-1,y,62)
                       255-264: x < Z-1 → 267-277: b(x+1,y,62)
                       286-288: ifle 310（y>0）→ 291-301: b(x,y-1,62)
                       310-319: y < Z-1 → 322-332: b(x,y+1,62)
```

⇒ 型 60 块是**原字节码里的复制粘贴缺陷**（三块并列、唯独 60 丢了守卫），不是投影颠倒。

### 2.2 三层判定

- **②投影排除（A 级）**：先调后判是原字节码实况（上引指令序）；且 A/B/C 三变体
  同 tick 同点崩溃（fx-bug-aioobe-border 夹具三方 trace 含崩溃尾一致）——C 的
  投影不可能制造 A 的崩溃。
- **①shim 排除**：崩溃面是游戏自有数组 `cellTypeGrid`（byte[high][wide]）的 JVM
  标准越界语义；触发路径=出生点/RMS 预置+行走 bump，与 MIDP shim 渲染/按键实现
  无关。真机 CLDC VM 对 `cellTypeGrid[-1]` 同样抛 ArrayIndexOutOfBoundsException。
- **③原版坐实（A 级）**。**原 ledger 定性（"保留原行为（崩溃即原版语义）"）成立**
  且可升级为指令级证据。

### 2.3 顺带更正（证伪更正文化，原主张留档）

> 【2026-10-16 更正】bug-ledger BUG-009 补强注曾称"**型62 块 x-1 同病**
> （a.java:5575-5590）"——字节码复核**不成立**：型 62 块（dump 228-341）四方向
> 均有前置短路守卫（上引）。同病的只有型 60 块。原主张留档于 ledger 原文。
>
> 【附带观察，B 级】型 60 块 x+1 方向的边界判用 `Z:I`（mapCellsHigh）而非
> `Y:I`（mapCellsWide）（dump 427-436，对照型 61 块 109-118 用 Y）——方图
> （13×13/26×26）上不可观测，非方图时是潜伏误判。不影响本次定性。

### 2.4 处置

- Stage A：维持 D4 保真（崩溃即原版语义，夹具钉崩溃前缀已由 task-10 完成）。
- Stage B 修复面：型 60 块四方向改先判后调（或 findBlockingEntity 入口钳制）。

## 3. BUG-012：CES_70 槽越界（Index 16 of length 16，spawnEntity）

### 3.1 "未解反常"先行撤销：转储截断伪影（A 级）

原反常："崩溃前 T349 转储 cellSlotCount **全行 ≤1**（35+1 实体各占独立格行，
nextCellTypeId=36）与'某行满 16'矛盾"。

**根因 = 探针截断**：`reference/oracle/src/oracle/host/StateDump.java`
`MAX_ARRAY_ELEMS = 32`（嵌套数组只 dump 前 32 行）、`int n = Math.min(len, 16);`
（平面数组 head 只列前 16 值）⇒ 逐字复核 fx-lay2a C-run 尾块（trace.txt T344 块）：

- `FLD 136 [B ARR[128] head=0,1,1,1,…` —— **head 只是 row 0..15**；row 16..127 **从未被打印**；
- `FLD 137 [[B ARR[128]{ARR[16]…}` —— 只显示前 32 行；
- 同块佐证身份：`FLD 135 [[B[13][13]` = cellTypeGrid（稀疏 id 1..36，(6,11)=36 =
  CES_84 落位）、`FLD 138 byte 36` = nextCellTypeId ✓ 与"35+1 实体"吻合。

⇒ "全行≤1"是对**截断头**的误读，与"某行满 16"**不构成矛盾**；此前"强烈暗示②"
的最重要依据已消解。**结论修正：反常不是矛盾，是探针伪影（②方向的证据失效）**。

### 3.2 投影忠实性复核（逐指令，A 级）——②排除

按 lead 指定的三处（维度/读槽/登记序列）逐一核对 `a.before.javap.txt`：

1. **分配（initCellGrid，obf `s()`，dump 0-43）**——与投影逐字段一致：
   ```
    2: getfield Z:I   6: getfield Y:I
    9: multianewarray [[B  13: putfield c:[[B    ← cellTypeGrid = byte[high][wide]
   17: sipush 128  20: newarray byte  22: putfield k:[B   ← cellSlotCount = byte[128]
   26: sipush 128  29: bipush 16  31: multianewarray [[B  35: putfield d:[[B
                                                    ← cellSlotTable = byte[128][16] ✓
   39: iconst_0  40: putfield p:B                   ← nextCellTypeId = 0 (byte)
   ```
2. **登记序列（spawnEntity，dump 152-250）**——索引语义/byte 语义全对：
   ```
   155: loop(var4 < var5)                            ← var5 = entitySpriteW[T]>>5（18-29，clamp≤0→1 at 53-59）
   162-176: var6 = cellTypeGrid[var8][var7+var9]      ← baload（byte→int 符号扩展）✓
   178: ifne 212                                     ← 已分配则跳过创建
   181-205: grid[y][x+dx] = (byte)(p+1); p = (byte)(p+1)  ← i2b 截断（128 回绕）✓ 投影如实
   212-230: cellSlotTable[var6][cellSlotCount[var6]] = (byte)entityCount  ← bastore 截断 ✓
   231-241: cellSlotCount[var6] = (byte)(count+1)     ← 无上界检查（原版如此）
   ```
   `cellSlotTable` 行下标与 `cellSlotCount` 下标**同为 var6**（216/223 同 iload 6）——
   "读对槽吗"的疑问排除。
3. **removeGroupAtCellByType（obf `a(III)V`，dump 0-111）**：就地压缩+计数递减
   (103-110)，与投影一致。

⇒ ②在崩溃点的三个指控点全部排除。

### 3.3 机制（B 级候选，未全钉）

`cellSlotTable[row][16]` 的列下标 = `cellSlotCount[row]` ⇒ 崩溃行的计数在崩溃瞬间
确为 16。两处追加点**都无上界检查**：

- spawnEntity 登记（§3.2-2）；
- **槽迁移 m_108（obf `f(III)V`，dump 0-185）**：摘旧槽（25-87，未命中则不动）→
  目的格追加（159-184，`row[count]=(byte)(idx+1); count++`，**同样无钳制**）。

且 m_108 有**已归档的重复追加 quirk**（crates/game-core/src/entity.rs relocate_slot
注记，A 级语义）："起点格由调用方传入（MOV 五参写的 f_int_129/130，全程不随实体
移动更新）——同一实体重复调用（m_106 途中击杀后路径未尽再收尾，a.java:8848+8806）
时，首次已迁走 ⇒ 二次摘除落空，但**追加照做**（removed 实体重新登记入槽）"。
⇒ **候选机制**：路径实体反复迁移/收尾重复追加 ⇒ 某行计数累积至 16 ⇒ 下一次落入
该行的 spawnEntity 登记即 `Index 16 of length 16`。350 拍的 fx-lay2a 时程（floor3
35 个出层实体 + 脚本链自动推进）为累积提供了时长。**未全钉**：具体哪行/哪拍累积到
16 需一次**全量转储**（StateDump 的 16/32 截断是当前唯一障碍——扩展探针或 /tmp
一次性探针均可，属运行时取证，超出本 task 写域）⇒ 如实留档不下最终结论。

对照"Rust 同构槽表未溢出存活"：Rust `relocate_slot`/`spawn` 同为 16 槽无钳制
（保真），未溢出仅因现有夹具未复现该累积路径——不是移植偏差。

### 3.4 三层判定与重定性

- **②排除（A 级）**：§3.2 逐指令忠实 + 三变体同栈（A 的崩溃在原字节码内）。
- **①排除**：崩溃面=游戏自有槽表（byte[128][16]）的标准越界；触发链 = 语料脚本
  script0（CES_84/70 链）+ sprite3 出层数据 + RMS 预置——真机推进 floor3 剧情对话
  即达（此前 B 级"疑真机同崩"保持，真机证词仍是最后缺口）。
- **③坐实（崩溃存在性 A 级）**。**维持 ledger"原版 bug（D4 保真）"定性**；
  "根因未全钉"注记升级为：反常已解（探针伪影）+ 机制候选已收敛（m_108 重复追加，
  B 级）。
- 处置：Stage A 维持（崩溃前缀夹具 + LAY/RES 不可达终裁不变）；Stage B 修复面 =
  **两处追加点加 `count < 16` 钳制**（spawnEntity 登记 + m_108 追加）+ 根因深挖
  （全量转储跑一发坐实累积行）。

## 4. 遗留（如实登记）

1. BUG-012 累积行/累积拍未实证（需全量转储运行；StateDump 扩展或 /tmp 探针）。
2. BUG-009/012 真机行为无证词（B 级"疑真机同崩"维持）。
3. §2.3 附带观察：型 60 x+1 边界判用 Z 非 Y（非方图潜伏误判，B 级）。
4. 本 task 写域仅本文档：ledger 行更新（BUG-009 补强注更正、BUG-012 反常销记）
   由 lead 路由；StateDump/夹具/补丁改动未动。
