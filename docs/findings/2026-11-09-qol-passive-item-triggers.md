# QOL 被动道具触发点考证（15 筋斗云 / 23 太公杖 / 24 捆仙绳 / 25 幸运币）

> 只读考证留档。参考版 `reference/src/deobf/a.java`（10889 行）为唯一裁判基准。
> 证据纪律（AGENTS.md §3）：**A** = 行号+原文；**B** = 调用图推断；**C** = 猜测，只进各节「存疑点/假设区」。
> 交叉引用：`docs/spec/evidence-items.md`、`docs/spec/gameplay.md:184`（R-move-7）、
> `docs/spec/input-operations-inventory.md:41`、`docs/spec/type-closure.md:68`。
> 本档为**独立取证**（逐条回读原文核行号），非转抄既有 spec。
> 代码摘录逐字；**长 desc 文本因排版做了软换行**（源文件中为单行字符串），行号不变。

## 0. 前置：四者为何「在 activateItem 里找不到」

**已证实（A）**：四者在 `activateItem` 全是空分支。

```java
// reference/src/deobf/a.java:7790
private boolean activateItem(byte var1) {
   ...
// a.java:7798-7806（case14 无 break 落入，随后统一 break）
   case 14:
      this.gameMode = 12;
      this.dispatchModeEntry();
   case 15:
   case 23:
   case 24:
   case 25:
   default:
      break;
```

**已证实（A）**：`itemUseCounts`（a.java:944）对四者全 `-1`，永不递减；

```java
// a.java:944
this.itemUseCounts = new byte[]{-1, -1, -1, -1, 1, 1, 1, 3, 1, 1, -1, -1, -1, 1, ...};
// 索引 = itemTypeToStackIndex(t) = t-13（a.java:7713-7725，85→20、86→21）
//   15→2、23→10、24→11、25→12  全 = -1
```

`useItemStack`（a.java:7770-7789）因此走 else 分支只调 `activateItem`，不消耗：

```java
// a.java:7771-7788
if (this.itemStackUses[var1] > 0) {
   if (this.activateItem(...) && --this.itemStackUses[var1] == 0) { ...pop... }
} else {
   this.activateItem(this.itemStackTypes[var1]);   // uses=-1 ⇒ 恒走此臂
}
```

`useItemStack` 全文件仅 2 个调用点：a.java:3302、a.java:3652（物品栏路径）。

**已证实（A）**：物品栏对四者按确定键**只弹描述、不生效**（这就是 desc 写
「放在道具栏中有效」的原因——消费点在别处）：

```java
// a.java:3940-3954
var21.itemUseTargetType = var21.itemStackTypes[var21.itemPanelCursor];
switch (var21.itemUseTargetType) {
   case 13: case 14:
      var21.gameMode = 3; var21.activateItem(var21.itemUseTargetType); break label534;
   case 15: case 23: case 24: case 25:
      var21.showOverlayMessage((byte)3,
         var21.itemDescriptions[itemTypeToStackIndex(var21.itemUseTargetType)], (byte)0, (byte)3);
      break label534;
   ...
```

**穷尽性（A）**：全文件 `findItemStackIndex` 共 8 处出现 = 1 处定义（a.java:7664）+ 7 处调用
（a.java:5293 型13、7728 addItemToItemStack 内部、7950 型25、8198 型16、10001 型23、
10003 型24、10052 型25）。`== 15` 仅 a.java:9305（读档）。
**故下文即 23/24/25 的全部消费点，无遗漏。**

---

## 1. 筋斗云（type 15）：楼梯边瞬间上下层

### 结论

**已证实（A/B）**：筋斗云不走 `activateItem`，而是「拾取置旗 → 邻梯标志 → 数字键 49/55
→ `findFloorGateEntity` → `changeFloor(..., var3=true)`」的四段链。触发条件 = 同时满足
①持有筋斗云（`tutorialSeen==true`）②玩家格与本层楼梯锚点格**四邻且非同格**（`f_bool_13`）
③本层存在 `entityType==7/8 && (entityParam>>9)==0` 的楼梯 ④目标层落在
`[minFloorReached, maxFloorReached]` 探索区间内。**每层楼梯都可用**（只要该层有合规楼梯
且玩家站到它旁边）；与普通上下楼是**两条独立通路**——普通踩梯不受任何筋斗云门控。

### 证据锚（A 级）

| # | 锚 | 内容 |
|---|---|---|
| 1 | `a.java:909` | desc 原文 |
| 2 | `a.java:7534` `a.java:7538-7539` | `pickupItemType` case15 置 `tutorialSeen=true` |
| 3 | `a.java:9299` `a.java:9305-9306` | 读档：先清 false，栈内有 15 则置 true |
| 4 | `a.java:239` | `tutorialSeen` 字段声明（全文件仅 5 处：239/5772/7539/9299/9306） |
| 5 | `a.java:5767-5806`（前置 `a.java:5765-5766`） | `f_bool_13` 四邻判定（`applyStepCellEffects` 尾部）；`var2/var6 = playerCellX/playerCellY` |
| 6 | `a.java:6424-6428` | `spawnEntity` type7→`stairUpMarkerX/Y`、type8→`stairDownMarkerX/Y`；`a.java:9450` `loadFloorData` 置 -1 |
| 7 | `a.java:5265` `a.java:5295-5306` | `handleFieldInput` 的 `case 49` / `case 55` |
| 8 | `a.java:7115-7133` | `findFloorGateEntity(boolean)` |
| 9 | `a.java:7135-7178` | `changeFloor(int,boolean,boolean)` 探索区间门 |
| 10 | `a.java:5638-5664` | 普通踩梯（case 7 / case 8） |
| 11 | `a.java:2406-2414` | HUD 邻梯箭头（`if (this.f_bool_13)`） |
| 12 | `a.java:7864` `a.java:7869` | 上行符(21)/下行符(22) = `currentFloor±1`，`var3=false` |

### 原文摘录

**① 拾取置旗（a.java:7534、7538-7539）** —— case15 无 break，落入 case13..25 公共段执行 `addItemToItemStack`：

```java
private void pickupItemType(int var1) {
   var2 = this.equipTierIndex(var1, 0, 12);
   switch (var1) {
      case 15:
         this.tutorialSeen = true;
      case 13: case 14: case 16: ...
```

读档恢复（a.java:9299、9305-9306）：

```java
this.tutorialSeen = false;
...
if (this.itemStackTypes[var12] == 15) { this.tutorialSeen = true; }
```

**② 邻梯标志 `f_bool_13`（a.java:5767-5806，`applyStepCellEffects` 尾部）** —— 玩家格须与楼梯锚点格四邻、非同格：

```java
int var7f = var21.stairUpMarkerX >> 5;
int var43 = var21.stairUpMarkerY >> 5;
var45 = var21.stairDownMarkerX >> 5;
var49 = var21.stairDownMarkerY >> 5;
var21.f_bool_13 = false;
if (var21.tutorialSeen) {
   if (var6 > 0) {
      if (var2 == var7f && var6 - 1 == var43) { var21.f_bool_13 = true; }
      else if (var2 == var45 && var6 - 1 == var49) { var21.f_bool_13 = true; }
   }
   if (var6 < var21.mapCellsHigh - 1) { /* var6+1 同式 */ }
   if (var2 > 0) { /* var2-1 同式 */ }
   if (var2 < var21.mapCellsWide - 1) { /* var2+1 同式，命中即 return */ }
   ...
}
```

注：四式均要求**恰好差 1 格**，玩家站到楼梯格上时四式全不成立 ⇒ `f_bool_13=false`
（此时走普通踩梯路径，见 ⑥）。`var2`/`var6` 即 `playerCellX`/`playerCellY`
（a.java:5765-5766），而 `applyStepCellEffects` 开头以 `playerCellX = playerPixelX >> 5`
算得（a.java:5545-5546）⇒ **格 = 32px**，`>>5` 是像素→格换算。

**③ 消费键 = 49 / 55（a.java:5295-5306）** —— 49/55 属 ASCII 数字键族
（同文件 50/52/54/56 = 方向键 2/4/6/8、53 = 确定，见 `docs/spec/input-operations-inventory.md:41`）：

```java
case 49:
   this.keyValue = 0;
   if (this.f_bool_13 && (var1 = this.findFloorGateEntity(true)) >= 0) {
      this.changeFloor(this.entityParam[var1] & 255, false, true);
   }
   break;
case 55:
   this.keyValue = 0;
   if (this.f_bool_13 && (var1 = this.findFloorGateEntity(false)) >= 0) {
      this.changeFloor(this.entityParam[var1] & 255, true, true);
   }
```

（`keyValue = 0` 先清键再判——历史移植 bug 已修，见 `docs/spec/type-closure.md:68`。）

**④ `findFloorGateEntity`（a.java:7115-7133）** —— 全实体扫描**第一个**合规楼梯，
**与玩家邻接的那个楼梯无关联**：

```java
private int findFloorGateEntity(boolean var1) {
   if (var1) {
      for (int var2 = 0; var2 < this.entityCount; var2++) {
         int var3 = this.entityParam[var2] >> 9;
         if (this.entityType[var2] == 7 && var3 == 0) { return var2; }
      }
   } else {
      for (int var5 = 0; var5 < this.entityCount; var5++) {
         int var6 = this.entityParam[var5] >> 9;
         if (this.entityType[var5] == 8 && var6 == 0) { return var5; }
      }
   }
   return -1;
}
```

**⑤ `changeFloor` 探索区间门（a.java:7135-7178）** —— 只有筋斗云路径带 `var3=true`：

```java
private boolean changeFloor(int var1, boolean var2, boolean var3) {
   ...
   if (var3) {
      if (var1 < this.minFloorReached) {
         this.showOverlayMessage((byte)0, this.miscTexts[0], (byte)0, (byte)0);  // 拒绝
         var5 = false; break label37;
      }
      if (var1 > this.maxFloorReached) {
         var5 = false;
         this.showOverlayMessage((byte)0, this.miscTexts[1], (byte)0, (byte)0);  // 拒绝
         break label37;
      }
   }
   var5 = true;
   ...
   if (var1 < 0) { showOverlayMessage(..., this.miscTexts[2], ...); }
   else if (var1 > this.floorCountMax) { showOverlayMessage(..., this.miscTexts[3], ...); }
   else { ...; this.m_119(this.currentFloor); this.pendingFloor = (byte)var1; this.wentDown = var2; }
```

`miscTexts[0..3]`（声明 a.java:438，条目 a.java:439-442）：

```java
"无法再下一层了，这是你达到的最底层",
"无法再上一层了，这是你达到的最高层",
"无法再下去了。",
"无法再上去了。",
```

**全文件 `changeFloor` 调用点（穷尽）**：

| 行 | 调用 | var3 | 带探索区间门 |
|---|---|---|---|
| a.java:4779 | `changeFloor(1, false, false)`（case17 道具） | false | 否 |
| a.java:5298 / a.java:5304 | 筋斗云 49 / 55 | **true** | **是（唯一）** |
| a.java:5655 / a.java:5661 | 普通踩梯 type7 / type8 | false | 否 |
| a.java:7864 / a.java:7869 | 上行符21 / 下行符22（`currentFloor±1`） | false | 否 |

**⑥ 普通上下楼（与筋斗云无关）（a.java:5638-5664）** —— 踩到梯格即换层，
**无 `tutorialSeen` 门、无探索区间门**：

```java
var1 = this.cellSlotTable[var3][var5] - 1;
var2 = this.entityType[var1];
switch (this.renderCategoryByType[var2]) {
   case 1:
      switch (var2) {
         case 7:
            var1 = this.entityParam[var1] & 255;
            this.changeFloor(var1, false, false);
            this.keyHeldCode = 0; var5 = 0; continue;
         case 8:
            var1 = this.entityParam[var1] & 255;
            this.changeFloor(var1, true, false);
            this.keyHeldCode = 0; var5 = 0; continue;
```

**⑦ HUD 邻梯箭头（a.java:2406-2414）**：

```java
if (this.f_bool_13) {
   if (var32.stairUpMarkerX >= 0)   { drawImageClipped(f_Image_arr2_00[8][19], 104, 20, ...); ... }
   if (var32.stairDownMarkerY >= 0) { drawImageClipped(f_Image_arr2_00[8][19], 104, viewBottom-17, ...); ... }
}
```

（另有 a.java:6721-6735 在地图层直接画上/下梯箭头，只看 `stairUpMarkerX>=0` /
`stairDownMarkerX>=0`，**不看 `f_bool_13`** —— 即未持筋斗云也会画。）

**⑧ desc（a.java:909）**：

```
在楼梯边，可以瞬间上下层，留神晕机。
\c00ff00[使用次数：无限]
```

### sprite 数据实证（A 级，`analysis/ex/sprite*`，复刻 a.java:9527 字节消费）

全 137 个 type7/8 楼梯的 `(floor, 目标层, param 高字节)` 分布：

- 高字节 = 0：floor 1-9、51-54
- 高字节 = 1：floor 0、10-49
- 高字节 = 2：**唯一一处** —— sprite24 的 type7 @pixel(192,32)，`param=562=0x232`，目标层 50

因 `param >> 9 == 0 ⟺ 高字节 ∈ {0,1}`，该梯被 `findFloorGateEntity` 排除：
**在 24 层按 49（上）会跳到 25 层而非 50 层**；24→50 只能普通踩梯。

> 玩家在 24 层邻接的是哪一部梯，只影响 `f_bool_13` 是否亮；真正选哪部梯由
> `findFloorGateEntity` 的「第一个」决定（见存疑点 C③）。

### 存疑点 / 假设区

- **C① `entityParam` 高字节语义未知**。语料中只有 0/1/2 三个取值，且按层段分片
  （1-9 与 51-54 = 0；0、10-49 = 1；24→50 = 2）。假设：区域/章节标记，或
  「单向/剧情梯」标记——**无代码证据，禁止据此 rename**。
- **C② `resetNewGameState`（a.java:7197 起）不重置 `tutorialSeen`**。grep 证明
  `tutorialSeen` 全文件仅 5 处（239/5772/7539/9299/9306），无其他写点 ⇒ 同一会话内
  「开新局」可能残留邻梯标志（此时新局若已在楼梯旁，49/55 会直接可用）。
  **假设级**：未跑运行时验证。
- **C③ `findFloorGateEntity` 取「全图第一个」而非「玩家邻接那个」**：多楼梯层
  可能跳到非邻接的那部梯。假设：原版行为如此（静态推理确定，实际体验未验）。
- **B 级**：`f_bool_13` 在 `applyStepCellEffects` 内重算，调用点为 a.java:3206（换层落地）、
  3605、4114、4320（步进完成）、7852（type20 瞬移后）⇒ 语义上「每步落地/换层后重算」；
  是否**每 tick**都跑需运行时确认。

---

## 2. 太公杖（type 23）：对天神力士（含巨灵神）伤害加倍

### 结论

**已证实（A）**：唯一消费点是 `effectiveAttackVsType`（a.java:9998-10008）：
`traitFlagsByType[敌人型] & 1` 非零 **且** 栈内持有 23 ⇒ 攻击力 ×2。
flag1 的敌人恰为 **49 巨力士 / 53 金刚力士 / 74 巨灵神**（a.java:6266-6267）——
与 desc「对天神力士（包括巨灵神）」一致。检查用 `findItemStackIndex(23)`（在道具栏即算持有，
与「放在道具栏中有效」吻合），**每次伤害结算都现查、无状态标记**。

### 证据锚（A 级）

| # | 锚 | 内容 |
|---|---|---|
| 1 | `a.java:9998-10008` | `effectiveAttackVsType` 本体 |
| 2 | `a.java:6266-6267` | `traitFlagsByType[49]=[53]=[74]=1; [69]=2;` |
| 3 | `a.java:954` 起（len 88） | `objectTypeNames` 索引核对 |
| 4 | `a.java:10039-10046` | `tickBattle` 实际扣血 |
| 5 | `a.java:917` | desc 原文 |
| 6 | `a.java:9992-9996` | `predictHpLossVsType`（预测共用同一函数） |

### 原文摘录

```java
// a.java:9998-10008
private int effectiveAttackVsType(int var1) {
   byte var3 = this.traitFlagsByType[var1];
   byte var2 = 1;
   if ((var3 & 1) != 0 && this.findItemStackIndex(23) >= 0) {
      var2 = 2;
   } else if ((var3 & 2) != 0 && this.findItemStackIndex(24) >= 0) {
      var2 = 2;
   }
   return this.playerAtk * var2;
}
```

```java
// a.java:6266-6267
this.traitFlagsByType[49] = this.traitFlagsByType[53] = this.traitFlagsByType[74] = 1;
this.traitFlagsByType[69] = 2;
```

`objectTypeNames`（a.java:954 起，88 条，按索引抽取核对）：

| 索引 | 名 | 索引 | 名 |
|---|---|---|---|
| 7 | 上楼梯 | 8 | 下楼梯 |
| 15 | 筋斗云 | 23 | 太公杖 |
| 24 | 捆仙绳 | 25 | 幸运币 |
| 41 | 天宫小犬 | 67 | 啸天犬 |
| **49** | **巨力士** | **53** | **金刚力士** |
| **74** | **巨灵神** | **69** | **杨戬** |
| 75 | 哪吒 | 85 | 黄金钥匙 |
| 86 | 日月无极钹 | | |

实际扣血（a.java:10039-10046，`tickBattle`）：

```java
int var1t = this.entityType[this.battleTargetEntity];
int var8;
if ((var8 = this.effectiveAttackVsType(var1t) - this.enemyDefScaled[var1t - 41]) > 0) {
   if ((this.f_int_151 & 3) == 0) {
      this.battleEnemyHp -= var8;
      this.spawnPopup((byte)(5 + this.f_int_151 % 3), var8, ...);
```

预测路径共用同一函数（`predictHpLossVsType` → `effectiveAttackVsType`），
调用点（全部 `predictHpLossVsType` / `effectiveAttackVsType`，B 级调用图）：
a.java:5288（`handleFieldInput` 内，写 `battleDigitByType` 显示伤害数字）、
a.java:5476（`interactWithCell`）、a.java:7987（`m_091`，写 `f_int_arr_22`/`f_byte_arr_20` 批量预测数组）、
a.java:8212（`tryRunScene` case34，`predictHpLossVsType(69,true)`）、
a.java:8774 与 a.java:8852（均在 `m_106`）⇒ **显示与实算同源**。

desc（a.java:917）：

```
当年姜子牙受天命封神，他的钓鱼竹竿被原始天尊附上了神力，可以役使天神力士供他差遣，
此杖又名“打神鞭”，对天神力士（包括巨灵神）威力加倍。
\cFFCC00[放在道具栏中有效]
```

### 存疑点 / 假设区

- **C**：flag1 只给了 3 个敌人 id，desc 的「天神力士」是叙事统称还是还有别的判定集合？
  **代码里没有名字集合，只有这 3 个 id** —— 假设「desc 的天神力士 ≡ {49,53,74}」。
- **B**：`else if` 结构 ⇒ 同时命中两个 flag 时优先 23（24 不再生效）；语料中 23/24
  的 flag 集不相交（{49,53,74} vs {69}），故实际不可达。
- **已证实**：`effectiveAttackVsType` 只乘攻击，不改防御/暴击；`enemyDefScaled` 在减法右侧
  未被放大 ⇒ 加倍是**净伤前的攻击力翻倍**，非最终伤害翻倍（当 `atk*2 <= def` 时
  加倍后仍不破防）。

---

## 3. 捆仙绳（type 24）：对「自恋的神仙」伤害加倍

### 结论

**已证实（A）**：与太公杖**同一函数** `effectiveAttackVsType` 的 `else if` 臂
（a.java:10003-10004）：`traitFlagsByType[敌人型] & 2` 非零 **且** 栈内持有 24 ⇒ 攻击力 ×2。
flag2 在全表**只有一处赋值**：`traitFlagsByType[69] = 2`（a.java:6267），
索引 69 = **杨戬**（`objectTypeNames`）。即「某些自恋的神仙」在代码里 = **仅杨戬一人**。

### 证据锚（A 级）

| # | 锚 | 内容 |
|---|---|---|
| 1 | `a.java:10003-10004` | `} else if ((var3 & 2) != 0 && this.findItemStackIndex(24) >= 0) { var2 = 2; }` |
| 2 | `a.java:6267` | `this.traitFlagsByType[69] = 2;`（全文件对 flag 的赋值仅 6266-6267 两行） |
| 3 | `a.java:954` 表 | 索引 69 = 杨戬 |
| 4 | `a.java:10039` | 消费点同第 2 节（`tickBattle`） |
| 5 | `a.java:918` | desc 原文 |

### 原文摘录

```java
// a.java:10001-10007（effectiveAttackVsType 内，承接第 2 节摘录）
} else if ((var3 & 2) != 0 && this.findItemStackIndex(24) >= 0) {
   var2 = 2;
}
return this.playerAtk * var2;
```

```java
// a.java:6267 —— flag2 的唯一赋值
this.traitFlagsByType[69] = 2;
```

desc（a.java:918）：

```
对某些自恋的神仙伤害加倍。
\cFFCC00[放在道具栏中有效]
```

### 存疑点 / 假设区

- **C**：「自恋的神仙」是否还包含未打 flag 的其他敌人（如哪吒 75）？
  **代码证据只支持 {69}**；假设原作者只给杨戬打了 flag。若后续剧情需要更多敌人，
  只能由 `traitFlagsByType` 新增赋值体现（当前 a.java 只有 6266-6267 两行）。
- **B**：`else if` ⇒ 23 与 24 互斥（见第 2 节）。
- **已证实**：与太公杖**共享同一个乘数变量** `var2`，两件道具不会叠成 ×4。

---

## 4. 幸运币（type 25）：打怪金钱加倍

### 结论

**已证实（A）**：**两处**击杀赏金点各自内联判断 `findItemStackIndex(25) >= 0`，
命中则 `enemyBaseGold[型-41] << 1`（即 ×2），否则原值：
① `tickBattle` 击杀（a.java:10052-10058）② `killAdjacentAt` 日月无极钹四邻杀
（a.java:7950-7954）。`gainGold` 本体（a.java:7527-7532）**不做任何加倍**；
商人（a.java:7336 `gainGold(1000,...)`）、脚本 GIN 指令（a.java:8439
`gainGold(var48,...)`）、宝箱开金**均不加倍** —— 与 desc「**打怪**得到的金钱加倍」严格一致。

### 证据锚（A 级）

| # | 锚 | 内容 |
|---|---|---|
| 1 | `a.java:10052-10058` | `tickBattle` 击杀赏金加倍 |
| 2 | `a.java:7950-7954`（else-if 起 7948） | `killAdjacentAt` 四邻杀赏金加倍 |
| 3 | `a.java:7527-7532` | `gainGold` 本体无加倍 |
| 4 | `a.java:7336` | 商人 `gainGold(1000, ...)` 不加倍 |
| 5 | `a.java:8439` | 脚本 GIN `gainGold(var48, ...)` 不加倍 |
| 6 | `a.java:245` / `a.java:1121` | `enemyBaseGold` 声明 / 初始化 |
| 7 | `a.java:919` | desc 原文 |
| 8 | `a.java:7664` | `findItemStackIndex` 定义（穷尽性依据，见 §0） |

### 原文摘录

```java
// a.java:10052-10058（tickBattle，击杀后）
if (this.findItemStackIndex(25) >= 0) {
   this.gainGold(
      this.enemyBaseGold[var1t - 41] << 1, this.entityPixelX[this.battleTargetEntity], this.entityPixelY[this.battleTargetEntity]
   );
} else {
   this.gainGold(this.enemyBaseGold[var1t - 41], this.entityPixelX[this.battleTargetEntity], this.entityPixelY[this.battleTargetEntity]);
}
```

```java
// a.java:7948-7954（killAdjacentAt，type86 日月无极钹四邻杀）
} else if (var6 < 67 && this.renderCategoryByType[var6] == 8 && this.entityState[var5] != 1) {
   this.markEntityRemoved(var5);
   if (this.findItemStackIndex(25) >= 0) {
      this.gainGold(this.enemyBaseGold[var6 - 41] << 1, 0, 0);
   } else {
      this.gainGold(this.enemyBaseGold[var6 - 41], 0, 0);
   }
}
```

（注意 popup 坐标是字面量 `0, 0` —— 四邻杀的赏金飘字落在世界原点，**原版行为，保真保留**。）

```java
// a.java:7527-7532 —— 本体不加倍
private void gainGold(int var1, int var2, int var3) {
   this.goldAmount += var1;
   if (var1 > 0) { this.spawnPopup((byte)4, var1, var2, var3); }
}
```

desc（a.java:919）：

```
打怪得到的金钱加倍。
\cFFCC00[放在道具栏中有效]
```

### 存疑点 / 假设区

- **已证实**：`<< 1` 是**无符号左移**，`enemyBaseGold` 为非负 ⇒ 无溢出/符号风险。
- **B**：`killAdjacentAt` 只对 `var6 < 67 && renderCategoryByType==8 && entityState != 1`
  的实体发赏金 ⇒ 与「打怪」定义一致（宝箱型 `t==5` 走 `tryOpenChest`，不受影响）。
- **C**：`gainGold` 的 `var1 > 0` 才飘 popup ⇒ 击杀 0 金怪时不加倍也无飘字；
  「打怪但不发金」的边角（如 `enemyBaseGold==0`）加倍后仍为 0，行为无差。

---

## 5. Rust 侧现状（交叉核对，供移植参考）

**任务书原 grep（`BindingRope|DamageReflect|AntiNarcissus|LuckyCoin|itemStackTypes==15`）
零命中，但四者其实已用别的名字实现**：

| 语义 | Java 锚 | Rust 锚 |
|---|---|---|
| 太公杖/捆仙绳 ×2 | `a.java:9998-10008` | `crates/game-core/src/combat.rs:93` `pub fn effective_attack_vs_type(player: &PlayerCombat, trait_flags: u8) -> i32`；:96/:98 用 `item_kind::TRAIT1_AMULET`(=23)、`TRAIT2_AMULET`(=24)；常量定义 :162/:164 |
| flag 表 | `a.java:6266-6267` | `crates/game-core/src/combat.rs:42` `trait_flags_for`：`49 | 53 | 74 => 1, 69 => 2`（注释已引 a.java:6266-6267） |
| 持有判定 | `findItemStackIndex` | `crates/game-core/src/combat.rs:67` `pub fn find_item(&self, item_type: u8) -> i32`（注释「findItemStackIndex 忠实移植 a.java:8485 区域」） |
| 幸运币 ×2（战斗） | `a.java:10052-10058` | 计算：`crates/game-core/src/subsystem/battle.rs:189-191` `let gold = if ctx.player.find_item(25) >= 0 { ctx.enemy_base[idx].gold << 1 } else { ... }`（注释引 a.java:10036-10062）；入账：`crates/game-core/src/scene.rs:3782` `self.player.gold += gold;`（`BattleOutcome::Exchange { ... gold, }` 字段） |
| 幸运币 ×2（四邻杀） | `a.java:7950-7954` | `crates/game-core/src/scene.rs:2303-2308`（`kill_adjacent_at`，:2282） |
| 筋斗云邻梯标志 | `a.java:5767-5806` `f_bool_13` | `crates/game-core/src/scene.rs:3342` `pub fn recompute_stair_hint(&mut self)`（注释引 a.java:5765-5805）；:3134 每 tick 调用 |
| 筋斗云置旗 | `a.java:7538-7539` `tutorialSeen` | `crates/game-core/src/effect_registry.rs:77` `pub tutorial_seen: &'a mut bool`、:190 Stack effect `*ctx.tutorial_seen = true`；接线 scene.rs:876/4212/4291；字段 scene_menu.rs:9；默认 false scene_persist.rs:118 |
| 49/55 键 | `a.java:5295-5306` | `crates/game-core/src/controls.rs:172-179` `if self.menu.stair_hint { if self.find_floor_gate_entity(true).is_some() { mid.push(Controls::key(FunctionId::CloudUp, 49)); } ... CloudDown 55 }` |
| 楼梯门查找 | `a.java:7115-7133` | `crates/game-core/src/scene.rs:758` `pub(crate) fn find_floor_gate_entity(&self, up: bool) -> Option<usize>`（:761 `(param >> 9) == 0`） |

**移植注意（B 级，措辞级缺陷）**：`crates/game-core/src/scene.rs:753-757` 有**两段重复且
行号引用错误的 doc 注释**（`a.java:7106-7118` / `a.java:7106-7122`，实际 `findFloorGateEntity`
在 **a.java:7115-7133**），其中 :755 写「`param>>9==0`（低 9 位无高字节——目标层 ≤255 的常规门）」
——**与实际位语义不符**：`param >> 9 == 0` 判的是**高字节 ∈ {0,1}**（`param < 512`），
低 9 位（目标层 + bit8）根本不参与判定。代码行为正确（:761 表达式无误），仅注释误导；
建议随本档一并更正（本任务只读，未改）。

**数据层接线（复核 2026-11-09，A 级）**：`docs/findings/reviews/2026-11-05-p12-effect-round2.md:143`
曾断言 `effect.key` 词表中 `Passive`（4 行）与 `TutorialSeenStack`（id15）**未注册** ⇒ 接线
miss=NoOp。**该结论现已过时**——本档复核实测：

- `crates/game-core/src/effect_registry.rs:115-120`：`(15, "TutorialSeenStack", EffectParams::Stack { type_id: 15, tutorial: true })`、`(22..25, "Passive", EffectParams::Stack { type_id: N, tutorial: false })`；
- `crates/game-core/src/effect_registry.rs:363`：`"TutorialSeenStack" | "Passive" => &StackEffect,`；
- `data/tables/items.json`（工作区当前态）四行 key 分别为 15=TutorialSeenStack、23/24/25=Passive。

即：**15/23/24/25 的拾取接线已通**（栈增；15 置 `tutorial_seen`），被动行为本身不走 effect
词表、而走 `combat.rs` / `scene.rs` / `subsystem/battle.rs` 的 `find_item` 活查（与 Java 一致）。

**相邻观察（B 级，超出本任务四题范围，仅登记）**：`items.json` 中 type22 下行符的
`effect.key = ChangeFloorDown`，而 `effect_registry.rs:364-366` 的 UseArm 分支只列了
`... | "ChangeFloorUp"`，**未列 `ChangeFloorDown`** ⇒ 按该词表会落到 `_ => &NoOpEffect`；
但 `crates/game-core/src/item_effect.rs:44` 另有 `(22, &[ItemAction::ChangeFloorDown])` 直通表
（:146 实现）。两条通路谁是消费面未考证 —— **留待后续，不改本档结论**。

---

## 6. 对求解器导入的含义

### 必须建模（否则求解器状态/收益算错）

| 效果 | 建模要点 | 证据 |
|---|---|---|
| **25 幸运币 → 打怪金 ×2** | 收益函数在**两处**击杀点翻倍：战斗击杀、type86 四邻杀。求解器的 gold 收益必须按「栈内是否有 25」分支。**只影响打怪**，商人/脚本/宝箱不翻倍。 | a.java:10052-10058、7950-7954 |
| **23 太公杖 → 对 {49,53,74} 攻 ×2** | 战斗伤害公式里按敌人型查 flag。注意是**攻击力翻倍后再减敌防**，不是最终伤害翻倍（`atk*2 <= def` 时仍不破防）。预测伤害与实算同源。 | a.java:9998-10008、6266、10039 |
| **24 捆仙绳 → 对 {69} 攻 ×2** | 同上，与 23 同函数、互斥（`else if`）。语料不相交，实际不会同时生效。 | a.java:10003-10004、6267 |
| **15 筋斗云 → 邻梯瞬移** | 需要建模 4 个状态分量：`tutorialSeen`（拾 15 置位）、`f_bool_13`（四邻非同格）、`findFloorGateEntity`（全图第一个 `type7/8 && param>>9==0`）、`changeFloor(var3=true)` 的**探索区间门** `[minFloorReached, maxFloorReached]`。**它是一条独立于踩梯的换层通道**，且是全游戏唯一带探索区间门的换层路径。 | a.java:7538-7539、5767-5806、7115-7133、7139-7151、5295-5306 |

### 可先忽略（对求解器目标函数无影响）

- **四者不消耗、不进 `activateItem`**：无次数资源约束（`itemUseCounts=-1`，走 else 臂）。
  求解器不必为它们建「使用次数」状态。
- **物品栏对四者只弹 desc**（a.java:3947-3954）：不产生状态变化，纯 UI 路径。
- **15 的 HUD 箭头**（a.java:2406-2414）：纯表现层。
- **23/24/25 不改防御/掉落表**：只影响 `effectiveAttackVsType` 的乘数与 `gainGold` 的入参。
- **商人 1000 金、脚本 GIN、宝箱金**：与 25 无关，按原值入模。

### 建模时的已知陷阱（原版行为，Stage A 保留）

1. **`findFloorGateEntity` 取「全图第一个」**，与玩家邻接的那部梯无关联（a.java:7115-7133）。
   多梯层瞬移目标可能与玩家预期不同——**求解器应以代码为准，不以「邻接」直觉为准**。
2. **24→50 那部梯（`param=562`，高字节=2）被 `>>9==0` 排除**：24 层按 49 只能到 25 层；
   50 层只能靠踩梯进入。sprite 语料中这是唯一被排除的楼梯。
3. **`resetNewGameState` 不重置 `tutorialSeen`**（C 级，未运行时验证）：求解器若模拟
   「同会话开新局」，需注意该标志可能残留。
4. **四邻杀的赏金飘字坐标是字面量 `(0,0)`**（a.java:7951 与 a.java:7953）：只影响 popup 位置，不影响 gold 数值。

### 尚未取证（留给后续）

- `entityParam` 高字节 0/1/2 的语义（C①）。
- `tutorialSeen` 在新局是否真的残留（C②，需运行时/读档 trace）。
- ~~`effect.key` 词表是否已注册 `Passive` / `TutorialSeenStack`~~ —— 本档已复核：**已注册**
  （见 §5，既有 review 结论过时）。遗留：`ChangeFloorDown` 是否漏出 UseArm 分支（§5 相邻观察）。
- `crates/game-core/src/scene.rs:753-757` 重复注释与措辞更正（本档只读，未改）。
