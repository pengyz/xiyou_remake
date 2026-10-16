# 证据附件：三型交互臂（型 2 红门 / 型 6 显形 / 型 82 传怪点）+ 型 11 墙文案条件

> 角色：spec-writer 取证产物。证据源：`reference/src/deobf/a.java`（10883 行）。
> 方法：grep 定位 case 标签 → 读上下文 → 逐字摘录，每条结论带行号指针。
> 证据分级：本文全部条目均为 **A 级**（反编译原文行号 + 逐字摘录）；推断处显式标注 B 级。
> 取证日期：2026-10-16。

术语约定（按 Java 原文字段 → Rust `crates/game-core/src/scene.rs` 现名）：

| Java 字段 | Rust 字段 | 语义 |
|---|---|---|
| `entityVisible` | `entities.visible` | 显隐 |
| `entityState` | `entities.solid` | 实体状态（0=常态，1=移除/跳过，2=显形/过渡，3=战斗中——a.java:5377/5414/5482） |
| `entityAnimRow` | `entities.anim_idx` | 动画行 |
| `var6`（interactWithCell 局部） | `allowed` | bump 放行判定 |
| `var7`（interactWithCell 局部） | `stair_count` | 楼梯/喷泉计数，>0 豁免地形终审 |
| `var8`（interactWithCell/applyStep 内层循环变量） | `s` / `s2` | 格槽倒序游标；置 0 = Java 停扫（Rust 置 -1） |

---

## R-INT-2：型 2 红门——耗红钥匙（code 27）开门，无钥匙提示阻挡

### A 级证据

bump 触发链（tryStep → interactWithCell 目标格）：

```
5351:       boolean var5;
5352:       if (var5 = this.interactWithCell(var2, var3)) {
5353:          this.walkPhase = 1;
5354:       } else if (this.overlayActive && this.walkPhase == 1) {
5355:          this.pathArrowsVisible = false;
5356:          this.walkStepCount = 0;
5357:          this.walkPhase = 0;
5358:       }
```

interactWithCell 内层 `switch (var5)`（var5=entityType）case 2，**原文**：

```
5389:                         case 2:
5390:                            if (var6 = this.consumeKeyForDoor((byte)27)) {
5391:                               this.markEntityRemoved(var4);
5392:                            } else {
5393:                               this.walkStepCount = 0;
5394:                               this.showOverlayMessage((byte)0, "你没有红钥匙", (byte)0, (byte)0);
5395:                            }
5396:                            continue;
```

consumeKeyForDoor——**code 27 = 红钥匙**（redKeyCount），原文：

```
7746:          case 27:
7747:             if (this.redKeyCount > 0) {
7748:                this.spawnPopup((byte)1, 1, this.playerPixelX, this.playerPixelY);
7749:                this.redKeyCount--;
7750:                var2 = true;
7751:             }
7752:             break;
```

同构对照（同一 switch 内三门并列，a.java:5381-5404）：case 1 → `(byte)26` 黄钥匙、文案 `"你没有黄钥匙"`；case 3 → `(byte)28` 蓝钥匙、文案 `"你没有蓝钥匙"`。

markEntityRemoved（开门效果），原文：

```
6349:    private void markEntityRemoved(int var1) {
6350:       this.entityState[var1] = 1;
...
6355:                case 1:
6356:                case 2:
6357:                case 3:
6358:                   this.entitySpriteW[var1] = 45;
6359:                   this.entitySpriteH[var1] = 56;
6360:                   this.entityAnimRow[var1] = 6;
6361:                   this.entityAnimFrame[var1] = 0;
6362:                   break;
...
6370:             this.buildMinimap();
```

跳过已开门的门槛（interactWithCell 槽扫描头），原文：

```
5377:             if (this.entityState[var4] != 1) {
```

### 语义总结

- **触发路径 = bump**（tryStep 于目标格调 interactWithCell，a.java:5333-5352；step 落格的 applyStepCellEffects **无**耗钥臂——其 `case 2: case 4:`（a.java:5719-5722）是 renderCategory 级道具拾取 `pickupItemType + markEntityRemoved`，与门无关）。
- **条件**：格内实体 entityType==2（红门，renderCategory 1）且 `entityState != 1`；`consumeKeyForDoor(27)` 即 `redKeyCount > 0`。
- **有钥匙效果**（严格顺序）：`spawnPopup(kind=1, icon=1, 玩家像素坐标)` → `redKeyCount--` → `markEntityRemoved`（entityState=1 + 换开门动画 45×56 行 6 帧 0 + buildMinimap）→ `continue`（续扫同格后继槽）。
- **无钥匙效果**：`walkStepCount = 0` → `showOverlayMessage((byte)0, "你没有红钥匙", (byte)0, (byte)0)` → var6=false。
- **返回值/allowed**：有钥匙 var6=true（放行，tryStep 置 walkPhase=1，玩家走进原门格）；无钥匙 var6=false（阻挡，若 overlayActive && walkPhase==1 还清路径/相位，a.java:5354-5357）。
- 可判定性：**机器可判定**（trace：redKeyCount 减 1、popup 事件、entityState 快照 0→1、overlay 文案 id）。

### Rust 落点建议

**已移植**，无需插入——`crates/game-core/src/scene.rs:4444-4483` 的 `1 | 2 | 3 =>` 臂：

- `2 => crate::combat::door_code::RED`（scene.rs:4451）；`RED: u8 = 27`（`crates/game-core/src/combat.rs:138`）✓
- 无钥匙文案 `"你没有红钥匙"`（scene.rs:4475）✓；`walk_step_count = 0`（4469）✓；`allowed = false`（4481）✓
- 需人工核对的一处保真点：Java 成功路径 popup 发生在 consumeKeyForDoor **内部**（计数减一之前），Rust 把 popup 放在 `consumed` 之后外层发（scene.rs:4458-4466）——同 tick 内顺序不可观测则可接受，若差分 trace 记 popup 顺序需对齐。

---

## R-INT-6：型 6 隐形实体——bump 显形（visible→entityState=2→animRow=9）

### A 级证据

interactWithCell 内层 `switch (var5)` case 6，**原文**（逐字，含顺序）：

```
5410:                         case 6:
5411:                            var7++;
5412:                            if (!this.entityVisible[var4]) {
5413:                               this.entityVisible[var4] = true;
5414:                               this.entityState[var4] = 2;
5415:                               this.entityAnimRow[var4] = 9;
5416:                            }
5417:                            continue;
```

var7 的地形豁免作用（interactWithCell 尾），原文：

```
5533:          if (var6 && var7 <= 0) {
5534:             var6 = this.isCellWalkable(var1, var2);
5535:          }
```

进层载入时型 6 生成为隐藏（m_122 的 sprite{n} 解析分支），原文：

```
9514:                      case 6:
9515:                      case 12:
9516:                         var5 = var13.spawnEntity(var5, var6, var7, 0);
9517:                         var13.entityVisible[var5] = false;
9518:                         break;
```

step 落格臂（applyStepCellEffects，供对照——与显形无关），原文：

```
5650:                      case 6:
5651:                         this.f_bool_07 = true;
5652:                         continue;
```

### 语义总结

- **触发路径 = bump**（tryStep → interactWithCell 于目标格；每次向该格迈步都会执行，含未成功的迈步）。
- **精确条件与顺序**：
  1. `var7++` **恒执行**（不论是否已显形）——计数置位在显形判定**之前**；
  2. `if (!entityVisible)` 才显形；写序固定为 **visible=true → entityState=2 → entityAnimRow=9**；
  3. `continue`（不改 var6，不置 var8=0，续扫同格后继槽）。
- **效果**：已显形的实体再次 bump 仅 var7++；显形三元组一次性（visible 置位后条件不再满足）。spawnEntity 默认 visible=true（a.java:6407），隐藏是载入时事后覆盖（9517）。
- **返回值/allowed**：case 6 本身不改 var6；但 var7>0 使 5533 行地形终审被跳过 ⇒ 只要没有其他槽把 var6 置 false，bump 型 6 **放行**（可走上该格；落格后 applyStepCellEffects case 6 置 f_bool_07=true，即喷泉浮沉位——Rust `player_bob_applied`）。
- 可判定性：**机器可判定**（trace：首次 bump 后三字段快照 {visible:0→1, solid:0→2, anim_idx:→9}；二次 bump 三字段不变、仅地形豁免）。

### Rust 落点建议

**缺口——需插入**。锚点：`crates/game-core/src/scene.rs:4515` 当前为

```rust
6 | 7 | 8 => stair_count += 1,
```

丢失了 a.java:5412-5416 的显形块。建议拆分为：

```rust
6 => {
    stair_count += 1;                    // var7++（a.java:5411，恒执行于显形判定前）
    if !self.entities.visible[e] {
        self.entities.visible[e] = true;  // a.java:5413
        self.entities.solid[e] = 2;       // entityState=2（a.java:5414）
        self.entities.anim_idx[e] = 9;    // animRow=9（a.java:5415）
    }
}
7 | 8 => stair_count += 1,
```

写序必须保持 visible→solid→anim_idx（A 级原文顺序）。step 臂 `6 => self.player_bob_applied = true`（scene.rs:4988）已移植 ✓。

---

## R-INT-82：型 82 传怪点——载入 spawn+hidden，MVS 脚本成对互传怪物

### A 级证据

**spawn+hidden 语义**（m_122（a.java:9449 起）解析 sprite{n} 的 case 82，与 76 共臂），原文：

```
9601:                      case 76:
9602:                      case 82:
9603:                         var8 = var25.read() | var25.read() + 1 << 8;
9604:                         var5 = var13.spawnEntity(var5, var6, var7, var8);
9605:                         var13.entityVisible[var5] = false;
9606:                         break;
```

spawnEntity 默认值（hidden 是事后覆盖的证据），原文：

```
6406:       this.entityRemoved[this.entityCount] = false;
6407:       this.entityVisible[this.entityCount] = true;
6408:       this.entityParam[this.entityCount] = (short)var4;
6409:       this.entityState[this.entityCount] = 0;
6410:       this.entityCount++;
```

**传怪动作 = 脚本指令 MVS**。分派处原文（`!equals("MVS")` 为假即 MVS 臂）：

```
8354:          } else if (!var3.equals("MVS")) {
```

MVS 处理臂（a.java:8519-8591）核心，原文：

```
8519:          } else {
8520:             a var13 = this;
8521:             this.f_short_arr_04 = null;
8522:             var13.f_short_arr_04 = new short[32];
...
8528:             while (--var6 >= 0) {
8529:                if ((var56 = var13.entityType[var6]) == 82) {
8530:                   int var43 = var13.entityParam[var6] & 255;
8531: 
8532:                   for (int var7 = 0; var7 < 32; var7 += 2) {
8533:                      if ((var70 = var13.f_short_arr_04[var7] - 1) <= 0) {
8534:                         var13.f_short_arr_04[var7] = (short)(var6 + 1);
8535:                         break;
8536:                      }
8537: 
8538:                      if (var43 == (var13.entityParam[var70] & 255)) {
8539:                         var13.f_short_arr_04[var7 + 1] = (short)(var6 + 1);
8540:                         break;
8541:                      }
8542:                   }
8543:                }
8544:             }
```

配对后的怪物搬运（正/反两个方向对称，原文其一 + 反向循环头）：

```
8561:                   for (int var12 = 0; var12 < var8; var12++) {
8562:                      int var10 = var13.cellSlotTable[var44][var12] - 1;
8563:                      var56 = var13.entityType[var10];
8564:                      if (var13.renderCategoryByType[var56] == 8) {
8565:                         var13.removeGroupAtCellByType(var13.entityPixelX[var9] >> 5, var13.entityPixelY[var9] >> 5, 82);
8566:                         var13.removeGroupAtCellByType(var86, var6, 82);
8567:                         var13.m_107(var10, var13.entityPixelX[var9] >> 5, var13.entityPixelY[var9] >> 5);
8568:                      }
8569:                   }
8570: 
8571:                   int var45 = var70;
8572:                   var70 = var9;
8573:                   var9 = var45;
```

m_107（搬运原语）与 removeGroupAtCellByType（点位消耗），原文：

```
8872:    private void m_107(int var1, int var2, int var3) {
8873:       int var4 = this.entityPixelX[var1];
8874:       int var5 = this.entityPixelY[var1];
8875:       short var6 = this.entityParam[var1];
8876:       this.spawnEntity(this.entityType[var1], var2 << 5, var3 << 5, var6);
8877:       this.removeGroupAtCellByType(var4 >> 5, var5 >> 5, this.entityType[var1]);
8878:    }
```

```
6434:    private void removeGroupAtCellByType(int var1, int var2, int var3) {
...
6440:             if (this.entityType[this.cellSlotTable[var6][var5] - 1] != var3) {
6441:                this.cellSlotTable[var6][var4++] = this.cellSlotTable[var6][var5];
6442:             } else {
6443:                this.entityRemoved[this.cellSlotTable[var6][var5] - 1] = true;
6444:             }
```

bump/step 均无 case 82 臂的原文依据（两处内层 switch 的 default）：

```
5472:                         default:
5473:                            continue;
```
（interactWithCell 内层 switch 尾；case 标签实录：1/2/3/4/6/7/8/9/10/11/12/81/83——无 82）

```
5716:                      default:
5717:                         continue;
```
（applyStepCellEffects 内层 switch 尾；case 标签实录：5/6/7/8/76/83——无 82）

### 语义总结

- **spawn+hidden 触发路径 = 进层载入**（m_122 解析 sprite{n}，changeFloor/LOAD 链进层时执行；恢复分支 a.java:9635-9638 从存档 blob 回读 visible 位）。**不是 bump、不是 step、不是出层**。
- **spawnEntity 参数**：`spawnEntity(82, x, y, param)`，x/y 为 sprite 记录的 2×u16 像素坐标；`param = b0 | ((b1 + 1) << 8)`（原文 `var25.read() | var25.read() + 1 << 8`，Java `+` 优先于 `<<`，A 级行文本 + B 级语法规则推导）。配对键 = `entityParam & 255` = b0（a.java:8530/8538）。
- **hidden 置位时机**：spawnEntity 返回后**立即** `entityVisible[idx] = false`（9605）——spawnEntity 内部先写 true（6407），隐藏是显式覆盖；同 tick 完成，无中间可观测态。
- **传怪动作触发**：脚本指令 **MVS**（executeScriptInstruction，a.java:8354 分派 / 8519-8591 处理）。MVS 语义：全实体表**倒序**扫 type==82，按 `param&255` 两两配对进 `short[32]`（16 对，槽位判空 `buf[i]-1 <= 0`）；每对 (A,B)：A 格上的怪物（renderCategory==8）→ 双向 `removeGroupAtCellByType(两格, 82)`（82 点位被**消耗**：摘槽 + entityRemoved=true）→ `m_107(怪物→B 格)`；再 A/B 交换（8571-8573）重复一遍（B 格怪物→A 格）。即**成对互传 + 一次性消耗两点位**。
- **m_107 = 传送**：在目标格 spawnEntity（同型同 param）+ 对**旧格全部同型**实体 removeGroupAtCellByType（摘槽 + entityRemoved=true）——是"复制-删除"而非搬移（若旧格同型多只会一并消失，quirk，Stage A 保真）。
- **bump/step 影响**：无专门臂 → default continue，不改 allowed、不置 var6=false；型 82 对行走**完全透明**（纯标记实体）。
- 可判定性：**机器可判定**（trace：进层后 82 实体快照 visible=0；执行 MVS 后两点位 entityRemoved=1、目标格新增怪物实体、怪物数守恒）。

### Rust 落点建议

1. **载入侧已移植**：恢复分支 `entities.visible[idx] = false`（scene.rs:951-953，注释已含 82）；bump/step 无臂 = 已正确（scene.rs:4604 `_ => {}`、scene.rs:5011 `_ => {}`）。
2. **MVS 未移植——需插入**。锚点：`crates/game-core/src/scene.rs:2103` 的 `} else {`（opcode 链尾，内部是 `todo!`，scene.rs:2104）。在其前插入 `} else if op == "MVS" { ... }`：
   - 16 对短缓存（f_short_arr_04 等价物，局部数组即可，Java 每次 MVS 重建）；
   - 倒序扫 `entity_type == 82`，配对键 `param & 0xff`；
   - 每对两向：`render_category == 8` 的槽实体 → `entity::remove_group_at_cell_by_type`（entity.rs:515，已存在）清两格 82 点 + m_107 等价搬运；
   - m_107 未移植：等价式 = `entity::spawn(同型, tx<<5, ty<<5, param)` + `entity::remove_group_at_cell_by_type(旧格, 同型)`（旧格全同型删除的 quirk 需保留）；
   - 尾部 `self.script_cursor += 1`（a.java:8591 `scriptCursor = var2 + 1`，var2 = opcode 起点+3，同链其他臂语义）。
3. 参数编码解析层已由 `crates/game-data/src/sprite.rs` 承担（记录格式注释 + `7 | 8 | 76 | 82 => 2` 变长表，sprite.rs:50-51）。

---

## R-INT-11：型 11 墙文案——`currentFloor != 23` 才显示（23 层静默但仍阻挡）

### A 级证据

interactWithCell 内层 `switch (var5)` case 11，**原文**：

```
5441:                         case 11:
5442:                            if (this.currentFloor != 23) {
5443:                               this.showOverlayMessage((byte)0, "障碍物：墙\n只有金勺子、玄明石可以凿开。或者剧情打开！", (byte)0, (byte)0);
5444:                            }
5445: 
5446:                            var6 = false;
5447:                            var8 = 0;
5448:                            continue;
```

### 语义总结

- **触发路径 = bump**（tryStep → interactWithCell）。
- **精确条件**：仅 `this.currentFloor != 23` 时执行 showOverlayMessage；文案原文 `"障碍物：墙\n只有金勺子、玄明石可以凿开。或者剧情打开！"`（含换行与句号，kind=0、左右键参数 0,0）。23 层 **跳过文案但不跳过阻挡**。
- **效果**：无条件 `var6 = false`（阻挡）+ `var8 = 0`（停扫同格后继槽，Java 循环头 `while (--var8 >= 0)` 置 0 后退出）+ `continue`。
- **返回值/allowed**：恒 false——无论是否显示文案，bump 型 11 一律被挡；且同格排在该槽之后的实体不再处理。
- 可判定性：**机器可判定**（trace：非 23 层 bump → overlay 文案事件 + allowed=false；23 层 bump → 无 overlay 事件 + allowed=false）。

### Rust 落点建议

**已移植**，无需插入——`crates/game-core/src/scene.rs:4567-4579` 的 `11 =>` 臂：

- `if self.floor != 23 { show_overlay_message(0, "障碍物：墙\n只有金勺子、玄明石可以凿开。或者剧情打开！", 0, 0) }` ✓（scene.rs:4569-4576，文案逐字一致）
- `allowed = false; s = -1;`（4577-4578）= var6=false + var8=0 停扫 ✓

---

## 附带登记：失效行号指针（供 parent 更正，本文件不改他处）

- `crates/game-data/src/sprite.rs:21`（文档注释 `a.java:7337-7342` 为 76/82 参数编码）与 `sprite.rs:39/50`（`a.java:7276`、`a.java:7309-7314`）的 a.java 行号指针**已失效**：当前 a.java 7325-7350 是道具效果 switch（原文 `this.gainGold(1000, ...)`、`this.addItemToItemStack(13)`）。
- 实证行号（A 级，grep `read() + 1 << 8` 全文件唯一命中）：76/82 参数编码 = **a.java:9603**；7/8 参数编码 `var25.read() | var25.read() << 8` = **a.java:9521**；二者均在 m_122（a.java:9449 起）的 sprite{n} 解析分支（记录循环 a.java:9499-9503）。请由有权限的角色更新 sprite.rs 注释指针。

## 遗留假设（C 级，不入结论）

- "型 6 = 喷泉/隐形路径"的语义命名属 C 级直观（依据仅 f_bool_07 喷泉浮沉 + 首碰显形）；本文只写行为，不锁命名。
- MVS 缩写词源（Monster Vanish/Shift?）未考证，不入结论。

---

## 补遗（2026-10-16 lead 审校补录，来源 = 取证 agent 报告消息）

1. **型 2 popup 时机差**：Java 的 spawnPopup(1,1) 在 consumeKeyForDoor **内部**（a.java:7746-7752，redKeyCount-- 同段）；Rust 在外层调用。移植体在 scene.rs:4444-4483（door_code::RED=27，combat.rs:138）——task-5 核对时机对拍差异。
2. **型 6 Rust 缺口**：scene.rs:4515 现为 `6 | 7 | 8 => stair_count += 1,`——需拆出 6 臂加显形三元组（visible=true → solid/state=2 → anim_idx=9，写序固定，a.java:5410-5417）。
3. **sprite.rs 注释行号失效**：game-data/src/sprite.rs 注释指 a.java:7337-7342/7309-7314/7276 已失效（该区现为道具效果 switch）；实证行号 = a.java:9603（76/82 编码 `read() | (read()+1)<<8`）与 a.java:9521（7/8 编码）。grep 全文件唯一命中。→ 待 task-5 或独立小 commit 更正注释。
4. **MVS 未移植**：型 82 传怪动作 = 脚本指令 MVS（分派 a.java:8354、处理 8519-8591）：倒序扫 type82 按 param&255 两两配对，怪物（cat8）双向 m_107 互传（a.java:8872-8878）+ removeGroupAtCellByType 消耗两 82 点位（a.java:6434-6449）。Rust opcode 链尾 else 是 todo!（scene.rs:2103-2105）——task-5 在其前插 `op == "MVS"` 臂。
