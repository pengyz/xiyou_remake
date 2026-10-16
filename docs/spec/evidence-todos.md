# 证据附件：scene.rs 三处 todo!() 的 Java 语义取证（task-5 前置）

> 角色：spec-writer 取证产物。证据源：`reference/src/deobf/a.java`（10883 行）+ 语料
> `assets/raw/script`（3861 B，GAME:/GUTS: 两段）+ 出层清单 `assets/raw/sprite{0..54}`。
> 方法：grep 定位 → 读上下文 → 逐字摘录 + Python 字节级解析语料/精灵清单，每条结论带行号指针。
> 证据分级：本文条目均为 **A 级**（行号 + 逐字摘录 / 可复现解析命令）；推断处显式标注 B 级。
> 取证日期：2026-10-16。配套：[evidence-interact-types.md](evidence-interact-types.md)（型 82/MVS 语义）、
> [type-closure.md](type-closure.md)、`docs/coverage-matrix.md` §5。

对应 todo!() 锚（本文唯一写作目标文件之外仅引用）：

| # | todo!() | 现锚 | Java 对应 |
|---|---|---|---|
| T-1 | `executeScriptInstruction 的 {op:?} …` | `crates/game-core/src/scene.rs:2104` | executeScriptInstruction（a.java:8257-8605） |
| T-2 | `paintEntityLayer case 8 的 67 多段拼装` | `crates/game-core/src/scene.rs:5603` | paintEntityLayer case 8（a.java:6560-6613） |
| T-3 | `paintEntityLayer 空图标签分支` | `crates/game-core/src/scene.rs:5660` | paintEntityLayer 空图 else（a.java:6672-6683） |

---

## T-1 executeScriptInstruction else 分支：op 标签全清单、语料分布与"未知 op"忠实行为

### 结构性更正（先于一切结论）

**Java 的 opcode 分派根本没有"未知 op 兜底 else"。** Rust `scene.rs:2104` 的
`else { todo!(…) }` 在 Java 侧对应的 `} else {`（a.java:8519）是 **已知 op `MVS` 的处理臂**
（型 82 配对传送，语义见 evidence-interact-types.md 补遗 #4）。"未知 op"在 Java 里
走**静默停摆**路径（下文 R-TODO-1b）。两件事必须拆开落点。

### A 级证据

分派骨架（3 字符标签 + 逐臂 else-if 链）：

```
8257:   private void executeScriptInstruction(String var1, int var2) {
...
8265:      if (var2 + 3 <= var1.length()) {
8266:         var3 = var1.substring(var2, var2 + 3);
8267:         if (var3.equals("TAK")) {
```

**全部 16 个 op 标签**（grep `var3.equals` @8257-8605，逐一命中）：

```
8267:         if (var3.equals("TAK")) {
8274:         } else if (var3.equals("MOV")) {
8310:         } else if (var3.equals("GUT")) {
8314:         } else if (var3.equals("DES")) {
8339:         } else if (var3.equals("SWD")) {
8354:         } else if (!var3.equals("MVS")) {
8355:            if (var3.equals("LAY")) {
8362:            } else if (var3.equals("ROS")) {
8389:            } else if (var3.equals("CES")) {
8424:            } else if (var3.equals("GIN")) {
8438:            } else if (var3.equals("ADD")) {
8467:            } else if (var3.equals("GLV")) {
8470:            } else if (var3.equals("RES")) {
8487:            } else if (var3.equals("SEE")) {
8510:            } else if (var3.equals("END")) {
8515:            } else if (var3.equals("SMS")) {
```

MVS 守卫是**取反条件**，其 `else` 即 MVS 处理臂（内层链 12 空格闭合于 8518，
`else` 在 9 空格与 `!MVS` 守卫同级）：

```
8515:            } else if (var3.equals("SMS")) {
8516:               this.parseScriptInt(var1, var2 + 1, " ");
8517:               this.scriptCursor = this.lastDelimPos + 1;
8518:            }
8519:         } else {
8520:            a var13 = this;
8521:            this.f_short_arr_04 = null;
8522:            var13.f_short_arr_04 = new short[32];
...
8528:            while (--var6 >= 0) {
8529:               if ((var56 = var13.entityType[var6]) == 82) {
8530:                  int var43 = var13.entityParam[var6] & 255;
```

MVS 臂收尾（无参 op，游标 = 标签后 1 字符位）与方法尾：

```
8591:            this.scriptCursor = var2 + 1;
8592:         }
8593:
8594:         if (this.scriptCursor < this.dialogScriptLine.length()) {
8595:            return;
8596:         }
8597:      } else {
8598:         this.dialogPhase = 4;
8599:         if (this.currentScriptIndex != 32) {
8600:            this.scriptLineFlags[this.currentScriptIndex] = true;
8601:         }
8602:
8603:         this.lockCameraOn(0);
8604:      }
8605:   }
```

**语料统计**（`assets/raw/script` GUTS: 指令区，297 条指令，0 条解析失败；复现命令见附录 A）：

| op | Java 行号 | Rust 现状（scene.rs） | 语料次数 | 判定 |
|---|---|---|---|---|
| TAK | 8267 | 1785 已移植 | 81 | 活码 |
| CES | 8389 | 2001 已移植 | 48 | 活码 |
| MOV | 8274 | 1904 已移植 | 47 | 活码 |
| DES | 8314 | 1951 已移植 | 41 | 活码 |
| ROS | 8362 | 2056 已移植 | 30 | 活码 |
| SEE | 8487 | 1869 已移植 | 30 | 活码 |
| GUT | 8310 | 1897 已移植 | 7 | 活码 |
| GIN | 8424 | 1804 已移植 | 4 | 活码 |
| SWD | 8339 | 1975 已移植 | 3 | 活码 |
| LAY | 8355 | 1793 已移植 | 1 | 活码 |
| RES | 8470 | 1818 已移植 | 1 | 活码 |
| ADD | 8438 | 1837 已移植 | 1 | 活码 |
| GLV | 8467 | 2037 已移植 | 1 | 活码 |
| END | 8510 | 2041 已移植 | 1 | 活码 |
| SMS | 8515 | 2053 已移植 | 1 | 活码 |
| **MVS** | **8354 守卫 / 8519 处理臂** | **未移植（scene.rs:2104 todo!）** | **0** | **死码候选** |

注意：naive `grep -o` 会把 GAME: 头段的 `SMS=0`、`END` 等误计（substring 计数 SMS=2/GUT=8）；
上表按 GUTS: 指令区 token 精确解析（`([A-Z]{3})((?:_-?\d+)*)` 全匹配，0 失败 token）。

**"未知 op"（16 标签之外）在 Java 的行为**：所有 else-if 落空 ⇒ **没有任何臂写
`this.scriptCursor`**（各臂自赋值，见上；无 default）⇒ 8594-8595 以**原 cursor**
`return` ⇒ 下一拍重新 `substring(cursor, cursor+3)` 解析同一 token ⇒ **静默停摆**
（dialogPhase 不变、scriptLineFlags 不置、行内后续指令永不执行）。

**MVS 语料死码的第二重证据**：全 55 层 sprite 出层清单解析（附录 B）——
**型 82 全库 0 个**（型 83 有 52 个）。MVS 臂的作用对象（成对型 82）在出层数据里不存在。

### 语义总结

- **R-TODO-1a（MVS 臂，已知未移植）**：`MVS` 是无参指令（a.java:8591 游标=标签后 1 位）；
  行为 = 型 82 配对传送（倒序收集 type==82 按 `param&255` 配对 → 双向把 cat8 怪物
  `m_107` 互传到对端 → `removeGroupAtCellByType(…, 82)` 消耗两 82 点位；
  a.java:8519-8592 + m_107 8872-8878：旧格删同型、目标格 `spawnEntity(同型, x<<5, y<<5, 同param)`）。
  **语料 0 调用 + 出层 0 实体 ⇒ 双重不可达（死码）**；但它是"已知 op"，不是"未知 op"。
- **R-TODO-1b（真未知 op，标签外）**：Java = **静默停摆**（不推进 scriptCursor、不改
  dialogPhase、下一拍原地重解析）。这不是防御性 skip，是实打实的卡死语义。
- 返回值/allowed 影响：无（executeScriptInstruction 返回 void；停摆表现为脚本冻结）。
- 可判定性：**机器可判定**——单测喂合成行 `"XYZ_1 "` → 断言 script_cursor/dialog_phase
  逐 tick 不变；MVS 臂用合成型 82 对 + 怪物对拍互传与点位消耗。

### Rust 落点建议

`crates/game-core/src/scene.rs:2104` 现 else 一刀切 todo!()，建议拆二：

1. **`op == "MVS"` 臂**（插在 `ROS` 臂后、else 前）：按 a.java:8519-8592 实现
   （语义细节引 evidence-interact-types.md 补遗 #4）；或按 dead-code-rulings 流程
   终裁 ☠️ 死码（证据：MVS 语料 0 + 型 82 出层 0，附录 A/B 命令可复现），终裁后
   **仍须保留显式注记**，不得用静默 skip 伪装已移植。
2. **else 兜底（真未知 op）**：忠实行为 = 直接 `return`，**不改** `self.script_cursor` /
   `self.dialog_phase`（对齐 a.java:8594-8595 的原地 return）；叠加
   `debug_assert!/log::warn` 作对拍哨兵。**禁止**按 3+1 字节跳过——Java 没有这个行为。
   若保留 `todo!()` 作未实现哨兵也可，但其语义注释须写明"MVS ≠ 未知 op"。

---

## T-2 paintEntityLayer case 8 的型 67 多段拼装（scene.rs:5603）

### A 级证据

case 8（renderCategory=8 怪物/Boss）分支头——抖动、开门动画、普通三态，67/69 特判入口：

```
6560:                     case 8:
6561:                        if (var13 == 3) {
6562:                           var9 += this.randomBelow(5) - 2;
6563:                           var10 += this.randomBelow(5) - 2;
6564:                        }
6565:
6566:                        if (var13 == 1 || var13 == 2) {
6567:                           this.drawImageClipped(this.f_Image_arr2_00[2][9], var9 + 2, var10 + 2 - (this.entityAnimFrame[var14] << 3), var5, 0, 27, 29);
6568:                        } else if (var11 != 67 && var11 != 69) {
6569:                           this.f_Graphics_00.drawImage(this.f_Image_arr2_00[5][0], var9 + 8, var10 + 22, 0);
6570:                           var10 -= 4;
6571:                           this.drawImageClipped(var4, var9 + (32 - var7 >> 1), var10 - (var8 - 32), var5, 0, var7, var8);
6572:                        } else {
6573:                           var7 = var10;
6574:                           var31 = var9;
6575:                           var5 = var14;
6576:                           a var22 = this;
6577:                           var8 = this.entityType[var5];
6578:                           var5 = var22.animOffsetTable[var22.entityAnimRow[var5]][var22.entityAnimFrame[var5]];
```

型 67 拼装臂**逐字**（var7=屏幕 y、var31=屏幕 x、var5=帧偏移；6582-6586 是反编译
变量倒手：基点=(x+16, y+16)、var7=帧偏移）：

```
6579:                           switch (var8) {
6580:                              case 67:
6581:                                 var22.f_Graphics_00.drawImage(var22.f_Image_arr2_00[5][0], var31 + 8, var7 + 22, 0);
6582:                                 int var10001 = var31 + 16;
6583:                                 int var10002 = var7 + 16;
6584:                                 var7 = var5;
6585:                                 var31 = var10002;
6586:                                 var5 = var10001;
6587:                                 a var23 = var22;
6588:                                 byte[] var42 = var22.f_byte_arr2_01[var7];
6589:                                 Object var45 = null;
6590:
6591:                                 for (byte var59 = 0; var59 < var42.length; var59 += 4) {
6592:                                    var12 = (byte)((Object[])(var45 = var23.f_byte_arr2_00[var42[var59]]))[0];
6593:                                    var23.drawImageClipped(
6594:                                       var23.f_Image_arr2_00[9][var12],
6595:                                       var5 + var42[var59 + 1],
6596:                                       var31 + var42[var59 + 2],
6597:                                       (int)((Object[])var45)[1],
6598:                                       (int)((Object[])var45)[2],
6599:                                       (int)((Object[])var45)[3],
6600:                                       (int)((Object[])var45)[4]
6601:                                    );
6602:                                 }
6603:                                 break;
```

> 【2026-10-16 更正】上引逐字块存在**类型投影失真**：片件表行被投影为
> `Object[]` 强转取 [0]..[4]（`(Object[])(var45 = …f_byte_arr2_00[…])`），
> 重编译 C 变体画型 67 即 `ClassCastException: [B cannot be cast to
> [Ljava.lang.Object;`（BUG-011）。javap 字节码实证（原 jar class a，
> `analysis/rename-pipeline/verify/a.before.javap.txt` 方法偏移 1153-1220）：
> `f_byte_arr2_00=[[B` 取行 **aaload**、元素 [0]..[4] 全为 **baload**、
> 无 checkcast/无元素级 aaload ⇒ 片件表真实取值语义 = `byte[][]` **逐字节读**；
> 循环槽 var59 为 istore/iinc 整型（`byte var59` 同为投影噪声）。本节表值
> （682-697）与画序语义**不变**，取值**类型**语义以字节码为准。已按
> "补丁即数据"正规化登记 `data/patches/vf-projection/VF-ACC-PAINT67.json`
> （原投影文本留档于补丁 old 字段），run.py 重生成 + javac 零错重建 C 变体后
> fx-boss67a 三方回归 **A==B==C**（trace sha256=f9e024c48237…，560 帧
> 逐帧对拍）。Rust 侧（adbdee0）以 (sheet,srcX,srcY,w,h) 元组建模 = 字节码
> 语义，无需改动。

元数据表**逐字**（a.java:682-697）——`f_byte_arr2_00` 片件表 9 行
`(sheetIdx, srcX, srcY, w, h)`，`f_byte_arr2_01` 帧拼装表 3 行 × 5 段 × 4 字节
`(pieceId, dx, dy, 第4字节)`：

```
682:      this.f_byte_arr2_00 = new byte[][]{
683:         {0, 0, 0, 33, 56},
684:         {0, 33, 0, 33, 56},
685:         {0, 66, 0, 33, 56},
686:         {2, 0, 0, 18, 26},
687:         {3, 0, 0, 18, 34},
688:         {1, 0, 0, 55, 61},
689:         {4, 0, 0, 14, 26},
690:         {4, 14, 0, 14, 26},
691:         {4, 28, 0, 24, 26}
692:      };
693:      this.f_byte_arr2_01 = new byte[][]{
694:         {6, -1, -79, 0, 3, -26, -5, 0, 4, 11, 6, 0, 5, -25, -53, 0, 0, -16, -27, 0},
695:         {7, -1, -80, 0, 3, -26, -5, 0, 4, 11, 6, 0, 5, -25, -54, 0, 2, -16, -26, 0},
696:         {8, -1, -81, 0, 3, -26, -5, 0, 4, 11, 6, 0, 5, -25, -55, 0, 1, -16, -25, 0}
697:      };
```

帧偏移域闭合（A 级）：`animRowByType[67] = 3`（a.java:498 数组，解析复现同附录 A 口径），
`animOffsetTable[3] = {0, 1, 2, 1}`（a.java:485-497）⇒ 型 67 帧偏移 ∈ {0,1,2}，
与 `f_byte_arr2_01` 3 行严格吻合；6578 取的是**裸帧偏移**（对照通用臂 6528
`var5 = var7 * animOffsetTable[…]` 乘精灵宽——拼装臂**不乘宽**，直接当拼装表行号）。

图源（A 级）：`dispatchSpriteSize` case 67 → `this.loadPackedImages(9);`（a.java:6303-6305），
`loadPackedImages(var1)` 载 `this.resourceNames[var1]` 进 `f_Image_arr2_00[var1]`
（a.java:4516-4523）；`resourceNames[9] = "xtq"`、`resourceImageCounts[9] = 6`
（a.java:450-453）⇒ `f_Image_arr2_00[9][0..5]` = **xtq 6 张拼装片图集**（型 69 用 [9][5]，
a.java:6611，与片件表 sheet 0-4 合计 6 张一致）。型 67 精灵盒 32×32（6323-6324 落底赋值）。

### 语义总结

- **触发路径 = 渲染**（paint → paintEntityLayer），不是 bump/step/出层。
- **条件**：cat8 且 `entityState ∉ {1,2}`（1/2 走 a.java:6566-6567 开门动画短路）且 type=67；
  state==3 时坐标先加 ±2 抖动（6561-6564）。
- **效果**（画序固定）：
  1. 影子 `f_Image_arr2_00[5][0]` 整图 @ (x+8, y+22)（6581）；
  2. 基点 (x+16, y+16)（6582-6586 倒手后 var5/var31）；
  3. 取段表 `f_byte_arr2_01[帧偏移]`，每 4 字节一段，共 5 段（688-691 行内 20 字节）；
  4. 每段：片件 `f_byte_arr2_00[段[0]] = (sheet, sx, sy, w, h)`，画
     `f_Image_arr2_00[9][sheet]` 到 (x+16+段[1], y+16+段[2])，源 (sx, sy)，尺寸 w×h（6591-6601）。
- **三帧差异**：首段片件 6/7/8（三个头件 14×26/14×26/24×26，sheet 4）随帧轮换，
  末段片件 0/2/1（三个体件 33×56，sheet 0）随帧轮换；中段 (3,-26,-5)、(4,11,6)、
  (5,-25,-53) 恒定（5 号件 = sheet 1 55×61 大件）。
- **第 4 字节**：段记录 `[…, 未读第 4 字节]` 在 6591-6601 的绘制循环里**只读 +0/+1/+2**
  与片件元数据 [1..4]——第 4 字节三行恒 0 且**读而不用**（保真保留，语义未知）。
- 返回值/allowed：无（纯渲染）。
- 可判定性：**机器可判定**——合成场景放型 67 + 指定 animRow/animFrame，逐帧像素 diff；
  另可对 682-697 表值做数据单测。

### Rust 落点建议

`crates/game-core/src/scene.rs:5603` 的 `todo!("paintEntityLayer case 8 的 67 多段拼装")`
替换为拼装臂，建议形态：

```rust
// a.java:6580-6603：型 67 多段拼装。影子(x+8,y+22) → 基点(x+16,y+16) →
// 5 段循环：xtq[piece.sheet] @ (x+16+dx, y+16+dy) src(sx,sy) w×h
// 段表 SEG67_FRAMES[frame_off]（a.java:693-697），片件表 SEG67_PIECES（a.java:682-692）
// frame_off = ANIM_OFFSET_TABLE[anim][frame]（裸值，勿乘宽——对照 a.java:6578 vs 6528）
```

- 常量两表照抄 682-697 逐值（第 4 字节保留占位，不参与绘制）；
- 画序、抖动（state==3）与开门短路（state 1/2）维持 case 8 现结构（scene.rs:5557-5562 已有）；
- 图源 `self.images.xtq[0..5]`（型 69 已用 `xtq[5]`，scene.rs:5588 附近，同源）。

---

## T-3 paintEntityLayer 空图标签分支（scene.rs:5660）——"floor1 不可达"论断裁定

### A 级证据

循环门与空图判定（分支前置）：

```
6505:      for (int var14 = 0; var14 < this.entityCount; var14++) {
6506:         if (!this.entityRemoved[var14] && this.entityVisible[var14]) {
...
6518:               if ((var4 = this.entityTypeImage[var11]) != null) {
```

分支**逐字**（a.java:6672-6683）：

```
6672:               } else {
6673:                  String var20 = this.objectTypeNames[this.entityType[var14]];
6674:                  int var21 = this.f_Font_00.stringWidth(var20) + 8 >> 1;
6675:                  this.f_Graphics_00.setColor(-1);
6676:                  this.f_Graphics_00.fillArc(var9, var10, 32, 32, 0, 360);
6677:                  var10 += 32 - this.fontHeight >> 1;
6678:                  var9 -= var21 - 16;
6679:                  this.f_Graphics_00.fillRect(var9, var10, var21 << 1, this.fontHeight);
6680:                  this.f_Graphics_00.setColor(0);
6681:                  this.f_Graphics_00.drawRect(var9, var10, (var21 << 1) - 1, this.fontHeight - 1);
6682:                  this.f_Graphics_00.drawString(this.objectTypeNames[this.entityType[var14]], var9 + 4, var10, 0);
6683:               }
```

空图类型收口（m_122 图表装载，a.java:9456-9485 节选）——1-81、84-87 全部有图，
**型 82/83 无任何赋值 ⇒ 恒 null**：

```
9456:      for (int var4 = 1; var4 <= 12; var4++) {
9457:         var17.entityTypeImage[var4] = var17.f_Image_arr2_00[4][var4 - 1];
...
9483:      var17.entityTypeImage[81] = var17.f_Image_arr2_00[4][3];
9484:      var17.entityTypeImage[71] = var17.entityTypeImage[70];
```

标签文本源（88 项，a.java:954 起）：

```
1037:         "传怪点",     // objectTypeNames[82]
1038:         "剧情点",     // objectTypeNames[83]
```

型 82/83 出层即隐藏（m_122 的 sprite 解析分支）：

```
9601:                     case 76:
9602:                     case 82:
9603:                        var8 = var25.read() | var25.read() + 1 << 8;
9604:                        var5 = var13.spawnEntity(var5, var6, var7, var8);
9605:                        var13.entityVisible[var5] = false;
9606:                        break;
...
9615:                     case 83:
9616:                        var8 = var25.read();
9617:                        var5 = var13.spawnEntity(var5, var6, var7, var8);
9618:                        var13.entityVisible[var5] = false;
```

**全库 entityVisible 置 true 的写点**（grep `entityVisible\[` 全清单核对）：
6407（spawnEntity 默认 true）、5413（型 6 首碰显形）、5455（型 12 变形显形）、
8345（SWD 指令：型 81→4 显形）、9638（存档还原照抄存档位）——**无一条作用于型 82/83**；
6407 唯一可达路径是运行时 CES/ADD 生成，而语料 CES/ADD 类型参数**不含 82/83**
（附录 A 输出：CES ∈ {6,20,22,36,40,44,46,47,51,56,60,61,69,70,72,73,77,84}，ADD={72}）。

**出层/语料实测**（附录 B 命令，字节级可复现）：

- `sprite1`（floor1）：35 实体，类型直方
  `{1:7, 7:1, 15:1, 26:5, 29:1, 30:1, 31:4, 32:1, 41:4, 42:1, 43:3, 44:1, 45:1, 46:1, 83:3}`——
  **型 83 三个、型 82 零个**；
- 全 55 层 sprite：无图类型仅 `83: 52`（型 82 全库 0 个）。

### 论断裁定：**"floor1 不可达"成立，B 级升 A 级**（且更强：全语料不可达）

推理链（每步 A 级）：空图分支要求 `entityTypeImage[t]==null`（6518）⇒ t ∈ {82, 83}
（9456-9485）⇒ floor1 的 3 个 t=83 出层即隐藏（9615-9618）⇒ 全部可见化写点不覆盖 83
且 CES/ADD 语料不生成 82/83 ⇒ 循环门 6506 `entityVisible` 恒 false ⇒ **paintEntityLayer
空图分支 floor1 不可达**。同口径把 sprite1 换成全 55 层（83: 52 全隐藏、82: 0）⇒
**该分支在整个出层+语料域内不可达（死码）**，floor1 只是其子集。

边界（B 级，不入结论）：若未来语料以 CES/ADD 生成型 82/83（或 SWD 式变形改型 83），
分支立即可达——保真期**不得删除分支**，只能实现或显式挂死码哨兵。

### 语义总结（分支本身的忠实行为，供实现）

- **触发路径 = 渲染**；条件 = 实体在屏内（6512 窗口测试）∧ 未删 ∧ 可见 ∧ 图 null。
- **效果**：白圆底（fillArc 32×32 全角 = 圆）@ (x, y)；`var21 = (stringWidth(name)+8)>>1`
  （半宽+4）；标签带 y += (32-fontHeight)>>1（垂直居中）；x -= (var21-16)（水平居中）；
  白底矩形 w=2·var21、h=fontHeight；黑框 drawRect(w-1, h-1)；文本左上锚 (x+4, y)。
  运算符优先级按 Java 原文：`stringWidth + 8 >> 1`、`32 - fontHeight >> 1` 均先加减后移位。
- 返回值/allowed：无（纯渲染）。
- 可判定性：**机器可判定**（注入可见无图实体 → 标签像素断言），但**语料/差分不可达**
  ⇒ 无 oracle 覆盖，只能单测兜底。

### Rust 落点建议

`crates/game-core/src/scene.rs:5660` 的 `todo!("…floor1 不可达")` 二选一（均须留注记）：

1. **保真实现**：按上节公式逐字转写 6673-6682——`f_Font_00.stringWidth` → Rust 字宽测量
   （paint_font），`fontHeight` → `crate::paint::paint_font().height`，fillArc 全角 =
   白色 32×32 圆（SoftGraphics 以 fill_oval 等价）；
2. **死码终裁**：挂哨兵 + 注记"A 级证据：型 82/83 全程隐藏（a.java:9605/9618 +
   可见化写点清单 + 语料 CES/ADD 无 82/83）"，走 dead-code-rulings 流程 ☠️。
   **禁止静默 `_ => {}` 或删除分支**（Java 行为只增不减）。

---

## 附录 A：语料 op/token/类型参数统计（复现命令）

```bash
cd /home/peng/workspace/xiyou_remake && python3 - <<'EOF'
import re, collections
text = open('assets/raw/script', encoding='utf-8').read().splitlines()
sec, ops, argtypes = None, collections.Counter(), collections.defaultdict(collections.Counter)
for ln in text:
    s = ln.strip()
    if s.endswith(':') and '=' not in s: sec = s; continue
    if sec != 'GUTS:': continue
    parts = s.split(' ', 1)
    if len(parts) < 2: continue
    for tok in parts[1].split(' '):
        if not tok: continue
        m = re.fullmatch(r'([A-Z]{3})((?:_-?\d+)*)', tok)
        if not m: print('UNPARSED TOKEN:', repr(tok)); continue
        op, args = m.group(1), m.group(2)
        ops[op] += 1
        argv = [int(a) for a in args.lstrip('_').split('_')] if args else []
        argtypes[op][tuple(argv)] += 1
print('op counts:', dict(sorted(ops.items(), key=lambda kv: -kv[1])))
for op, idx in (('CES',0), ('ADD',1), ('MOV',0), ('DES',0)):
    seen = collections.Counter()
    for argv, c in argtypes[op].items():
        if argv: seen[argv[idx]] += c
    print(op, 'type args:', sorted(seen.items()))
EOF
```

预期输出（2026-10-16 实测）：`op counts` 含 TAK 81 … SMS 1、**MVS 缺席**；
`CES type args` 无 82/83、`ADD type args: [(72, 1)]`、`MOV/DES` 无 82/83。

## 附录 B：sprite 出层清单解析（复现命令）

记录格式（a.java:9499-9503 + 各 case 尾参字节数）：u32 LE 条数 → 每条
`u8 type, u16 LE x, u16 LE y` + 尾参（4/5/81/9/57/59/70-73/77/78/83→1 字节；
7/8/76/82→2 字节；6/12/default→0）。

```bash
cd /home/peng/workspace/xiyou_remake && python3 - <<'EOF'
import struct, collections
EXTRA = {4:1, 5:1, 81:1, 6:0, 12:0, 7:2, 8:2, 9:1, 76:2, 82:2, 77:1, 78:1, 83:1,
         57:1, 59:1, 70:1, 71:1, 72:1, 73:1}
def parse(path):
    d = open(path,'rb').read(); n = struct.unpack_from('<I', d, 0)[0]; pos = 4; recs = []
    for _ in range(n):
        t = d[pos]; x, y = struct.unpack_from('<HH', d, pos+1); pos += 5
        pos += EXTRA.get(t, 0); recs.append(t)
    assert pos == len(d), (path, pos, len(d)); return recs
h = collections.Counter(parse('assets/raw/sprite1'))
print('sprite1:', len(list(h.elements())), dict(sorted(h.items())))
allt = collections.Counter()
for i in range(55):
    allt.update(parse(f'assets/raw/sprite{i}'))
print('no-image types (0/82/83/>87):', {t:c for t,c in allt.items() if t in (0,82,83) or t > 87})
EOF
```

预期输出（2026-10-16 实测）：`sprite1: 35 {1:7, 7:1, 15:1, 26:5, 29:1, 30:1, 31:4, 32:1,
41:4, 42:1, 43:3, 44:1, 45:1, 46:1, 83:3}`；`no-image types: {83: 52}`。

## 附带登记（供 lead/有权限角色处理，本文件不改他处）

- `docs/coverage-matrix.md` §5 三条 todo! 锚行号已漂移：登记 1894/5393/5450 ↔ 现锚
  2104/5603/5660（scene.rs 插行所致）。建议随 task-5/6 刷账时改锚。
- §5.1 ARM_GAPS 的 `interactWithCell … 型 76/82 交互臂未移植` 一行：Java interactWithCell
  根本无 case 76/82（a.java:5391-5530 switch 清单 1,2,3,4,6,7,8,9,10,11,12,81,83），
  落 `default: continue` = bump 放行——现 `_ => {}` **已忠实**，可按 evidence-interact-types.md
  R-INT-82 结论销账（型 82 的动作在 MVS/渲染侧，不在 bump 侧）。

## 遗留假设（C 级，不入结论）

- MVS 词源（Monster Vanish/Shift?）未考证。
- `f_byte_arr2_01` 段记录第 4 字节语义未知（三行恒 0、绘制循环不读）——保真占位。
- 型 67 若换 animRow 使帧偏移 ≥3（如 row 7 {3,4,5,6}），Java 将 `f_byte_arr2_01[3]`
  越界（AIOOBE）——"永不越界"是 B 级推断（animRowByType[67]=3 固定），保真期不加防御。

---

## 产出契约

- 规则清单：R-TODO-1a/1b（T-1 拆分）、T-2 拼装臂、T-3 标签分支裁定——均带行号+逐字。
- 可判定性：4 项全部**机器可判定**（单测/像素 diff）；T-3 另注"语料不可达，差分无覆盖"。
- 门禁：docs-only 变更（唯一写入 docs/spec/evidence-todos.md），门禁由 lead 统一提交时过闸。
- 遗留假设：3 项（上节 C 级）。
