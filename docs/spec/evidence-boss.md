# Boss 取证：67/68/74/84/87 战斗臂 + 钩子 + 出层语义（task-3）

> 取证源：`reference/src/deobf/a.java`（10883 行）。方法：grep 定位 → sed 30-50 行上下文 → 逐字摘录。
> 证据分级按 [AGENTS.md](../../AGENTS.md) §3：**A** = 行号 + 原文逐字；**B** = 代码语境/调用图推断；C 级不入结论。
> boss 清单（实体类型 ID）：67 啸天犬、68 太上老君、74 巨灵神、84 嫦娥、87 菩提老祖。
> 对照件（非本批，仅钩子族佐证）：71 玉皇大帝(二阶段)、75 哪吒、69 杨戬、73 天蓬元帅。
>
> 名称映射 A 级：`objectTypeNames`（a.java:954 起，逐项索引=型号）——
> a.java:1022 `"啸天犬"`(67)、a.java:1023 `"太上老君"`(68)、a.java:1029 `"巨灵神"`(74)、
> a.java:1039 `"嫦娥"`(84)、a.java:1042 `"菩提老祖"`(87)。

## 0. 结论速览（5 行账本）

| 型 | boss | 战斗触发臂 | 击杀钩子（A 级） | 出层/出场语义（A 级） | Rust 现状 |
|---|---|---|---|---|---|
| 67 | 啸天犬 | bump 战（interactWithCell cat8，a.java:5475-5491） | `spawnBossEvent(3)` → `bossEventSpawns[3]={17,6,4,0}`：金勺子(17)@(6,4)（a.java:2143 + 10063-10067） | 战后 switch 落 default（a.java:10134-10137 无特判）；剧情侧 script5 只"绕密道"不杀（a.java:1130 + 对话49-52） | ✅ 已移植（scene.rs:130/2999/4757-4767） |
| 68 | 太上老君 | bump 战（同上） | `spawnBossEvent(2)` → `bossEventSpawns[2]={27,6,5,0,27,5,6,0,27,7,6,0,27,6,7,0}`：红钥匙(27)×4 十字格（a.java:2142） | 战后 switch 落 default（a.java:10135-10137 无特判） | ✅ 同上 |
| 74 | 巨灵神 | bump 战（同上）+ 太公杖克制（trait=1，a.java:6266） | `spawnBossEvent(1)`（12 掉落，a.java:2092-2141）**且** `loadLevelScript(21)` = 遗言 TAK_59_59（a.java:10078-10080 + 1146） | 遗言脚本 21 **无 tryRunScene 门控**（直 load）；战后 switch 落 default | ✅ 钩子已移植（scene.rs:4782 `74 => self.load_level_script(21)`） |
| 84 | 嫦娥 | **无战斗臂**（cat32，bump 只阻挡，a.java:5527-5528） | 无击杀钩子（不在 bossTypeOrder，a.java:2198；战后 switch 无 case） | 出生源 3 处（floor2 恢复钩子 a.java:9644-9646；script0/19 CES_84）；出层脚本链 = scene26（flags[26]+在位门控，a.java:8182-8186）等 26-31 族 + script0 尾 `LAY_2`（a.java:1125） | ✅ 门控/钩子已移植（scene.rs:1501-1503、928-936） |
| 87 | 菩提老祖 | **无战斗臂**（cat32，同上） | 无击杀钩子（同上） | **无实体出生源**（grep 0 命中，§4.2）；纯对话发言者（dialogueSpeakerType，a.java:1358 起）+ `lockCameraOn(87)` 显式跳过锚定（a.java:8672）；出层链 = 序章 script40-53 教学 + script67 混沌世界引子（tryRunScene case67 = `difficultyIndex > 0`，a.java:8219-8220） | ✅ case67 已移植（scene.rs:1528） |

---

## 1. 公共机制（全部 A 级）

### R-boss-1: 战斗触发 = bump 臂（interactWithCell cat8），到达臂（applyStepCellEffects）无战斗

cat8 = 敌族（`renderCategoryByType` 41..78 → 8，a.java:6281-6283）：

```java
// a.java:6281-6283
      for (int var5 = 41; var5 < 79; var5++) {
         this.renderCategoryByType[var5] = 8;
      }
```

bump 战臂（`interactWithCell`，方法头 a.java:5363 `private boolean interactWithCell(int var1, int var2) {`，
由 `tryStep` a.java:5352 `if (var5 = this.interactWithCell(var2, var3)) {` 调用——即"走进怪物格"）：

```java
// a.java:5475-5491
                  case 8:
                     var5 = this.predictHpLossVsType(var5, true);
                     var6 = false;
                     if (var5 >= 0 && var5 < this.playerHp) {
                        var5 = var4;
                        a var15 = this;
                        this.walkPhase = 5;
                        var15.entityState[var5] = 3;
                        var15.battleTargetEntity = var5;
                        var15.battleEnemyHp = var15.enemyBaseHp[var15.entityType[var5] - 41];
                        var15.battleActiveFlag = true;
                        var15.battlePredictGate = true;
                     } else {
                        this.walkStepCount = 0;
                        this.showOverlayMessage((byte)0, "你无法战胜它", (byte)0, (byte)0);
                     }
                     break;
```

语义：
- 预测损耗 `predictHpLossVsType(型, true)`（a.java:9986-9990 → `predictBattleHpLoss` a.java:10004-10024）
  返回 -1（打不动：攻 ≤ 敌防）或 ≥ playerHp ⇒ 拒战，浮层"你无法战胜它"，`var6=false` 挡移动；
- 否则开战：`walkPhase=5`、`entityState=3`（战斗中标记）、`battleTargetEntity`、
  `battleEnemyHp = enemyBaseHp[型-41]`、`battleActiveFlag=true`、`battlePredictGate=true`。

对照：`applyStepCellEffects`（a.java:5543）的 cat 分派里 cat8 空转（无战斗臂）：

```java
// a.java:5719-5728
               case 2:
               case 4:
                  this.pickupItemType(var2);
                  this.markEntityRemoved(var1);
               case 3:
               case 5:
               case 6:
               case 7:
               case 8:
            }
```

⇒ **结论：67/68/74 的战斗只走 bump（interactWithCell），不走到达（applyStepCellEffects）。**（A 级）

### R-boss-2: 击杀钩子 = tickBattle 内 bossTypeOrder 查表 spawnBossEvent(idx) + 型特判 switch

击杀判定（敌 HP 归零那一拍，a.java:10041-10052）：

```java
// a.java:10041-10054
                  } else {
                     if ((var8 = this.predictHpLossVsType(var1t, this.battlePredictGate)) > 0) {
                        this.spawnPopup((byte)2, var8, this.playerPixelX, this.playerPixelY);
                     }
                     ...
                     var8 = this.entityType[this.battleTargetEntity];
                     a var7 = this;
                     boolean var3 = false;
```

钩子主体（**50 层以上整体跳过**，a.java:10058-10092）：

```java
// a.java:10058-10092
                     if (var7.currentFloor > 50) {
                        var10001 = false;
                     } else {
                        int var4 = var7.bossTypeOrder.length;

                        while (--var4 >= 0) {
                           if (var8 == var7.bossTypeOrder[var4]) {
                              var7.spawnBossEvent(var4);
                              break;
                           }
                        }

                        switch (var8) {
                           case 73:
                              var7.scriptLineFlags[var7.currentScriptIndex] = true;
                              var7.loadLevelScript(22);
                              if (var7.currentFloor == 40) {
                                 var7.spawnBossEvent(6);
                              }
                              break;
                           case 74:
                              var7.loadLevelScript(21);
                              break;
                           case 75:
                              if (var7.tryRunScene(20, true)) {
                                 var7.loadLevelScript(20);
                                 var3 = true;
                              }
                        }

                        var10001 = var3;
                     }

                     this.battleSceneRerun = var10001;
                     this.battleFinishing = true;
```

- `bossTypeOrder`（a.java:2198）：

```java
// a.java:2198
      this.bossTypeOrder = new byte[]{75, 74, 68, 67, 69, 71};
```

  ⇒ **`spawnBossEvent` 的参数 = bossTypeOrder 的下标**，不是型号：
  75→0、74→1、**68→2、67→3**、69→4、**71→5**（对照线索"67 击杀→(3)、71 击杀→(5)"全部对上，A 级）。
- 67/68 击杀**不在**型特判 switch（只有 73/74/75），落空 ⇒ 无脚本钩子（A 级）。
- `battleSceneRerun=true` 仅 case 75 且 `tryRunScene(20,true)` 过门时置位（a.java:10082-10088）——
  效果见 R-boss-4。

### R-boss-3: spawnBossEvent(idx) = 按 bossEventSpawns[idx] 逐 4 元组刷"战利品实体"并标 DES 态

```java
// a.java:8906-8915
   private void spawnBossEvent(int var1) {
      byte[] var5;
      int var2 = (var5 = this.bossEventSpawns[var1]).length;
      int var3 = 0;

      for (int var4 = 0; var4 < var2; var4 += 4) {
         var3 = this.spawnEntity(var5[var4], var5[var4 + 1] << 5, var5[var4 + 2] << 5, var5[var4 + 3]);
         this.markEntityDesRemoved(var3);
      }
   }
```

4 元组格式 = `[型, 格x, 格y, param]`，像素坐标 = `格<<5`（32px 格）。`markEntityDesRemoved`
（a.java:6332-6347）：

```java
// a.java:6332-6347
   private void markEntityDesRemoved(int var1) {
      this.entityState[var1] = 2;
      int var2 = this.entityType[var1];
      switch (this.renderCategoryByType[var2]) {
         case 1:
            return;
         case 2:
         case 8:
         case 16:
         case 32:
            this.entitySpriteW[var1] = 27;
            this.entitySpriteH[var1] = 29;
            this.entityAnimRow[var1] = 8;
            this.entityAnimFrame[var1] = 0;
      }
   }
```

语义（B 级标注）：`entityState=2` = "脚本/事件生成"态（绘制走 27×29 浮动图元，
a.java:6548/6567 用 `f_Image_arr2_00[2][9]` 画 27×29；拾取/战斗臂只看 renderCategory 且 `entityState != 1`
即可交互，a.java:5377）。bossEventSpawns 值域（grep 统计，可复现）= 型
{16,17,24,26,27,29,30,32}，全是 cat2 拾取物/钥匙/丹药 ⇒ **钩子语义 = 击杀即掉奖励**。

`bossEventSpawns` 8 项（a.java:2041-2197）逐项（本批只关心 1/2/3/5，其余对照）：

| idx | 触发者 | 表（A 级行号） | 内容 |
|---|---|---|---|
| 0 | 75 哪吒击杀 | a.java:2042-2091 | 12 件：威力丹29×3、黄钥匙26×3、金刚丹30×3、长寿丹32×3 |
| 1 | **74 巨灵神击杀** | a.java:2092-2141 | 12 件：威力丹29×3、黄钥匙26×3、金刚丹30×3、长寿丹32×3 |
| 2 | **68 太上老君击杀** | a.java:2142 | `{27, 6, 5, 0, 27, 5, 6, 0, 27, 7, 6, 0, 27, 6, 7, 0}` 红钥匙27×4（十字：(6,5)(5,6)(7,6)(6,7)） |
| 3 | **67 啸天犬击杀** | a.java:2143 | `{17, 6, 4, 0}` 金勺子17×1 @格(6,4) |
| 4 | 69 杨戬击杀 | a.java:2144 | `{16, 5, 5, 0}` 芭蕉扇16×1 |
| 5 | **71 玉帝二阶段击杀**（对照） | a.java:2145 | `{24, 6, 2, 0, 32, 5, 1, 0, 32, 6, 1, 0, 32, 7, 1, 0, 27, 6, 6, 0}` 捆仙绳24 + 长寿丹32×3 + 红钥匙27 |
| 6 | 73 击杀且 floor==40 | a.java:2146-2195 | 12 件：长寿丹32×3、黄钥匙26×3、威力丹29×3、金刚丹30×3 |
| 7 | 69 击杀且 floor==35（post-anim） | a.java:2196 | `{32, 4, 4, 0, 32, 5, 4, 0, 32, 6, 4, 0}` 长寿丹32×3 |

### R-boss-4: 胜利动画收尾 switch（battleAnimTick > 5）——67/68/74 均落 default

```java
// a.java:10102-10113
      } else {
         if (this.battleAnimTick == 0) {
            this.markEntityRemoved(this.battleTargetEntity);
         }

         if (++this.battleAnimTick > 5) {
            this.battleAnimTick = 0;
            if (!this.battleSceneRerun && var1) {
               this.tryStep(this.facingDirection);
            }

            this.battleFinishing = false;
            int var2 = this.entityType[this.battleTargetEntity];
```

```java
// a.java:10129-10156
            case 62:
            case 63:
            case 64:
            case 65:
            case 66:
            case 67:
            case 68:
            default:
               break;
            case 69:
               if (var5.currentFloor == 35) {
                  var5.spawnBossEvent(7);
                  var5.loadLevelScript(36);
               }
               break;
            case 70:
            case 71:
               if (var5.currentFloor == 49) {
                  var5.loadLevelScript(35);
               }
               break;
            case 72:
               if (var5.currentFloor == 50) {
                  var5.loadLevelScript(65);
               }
            }

            this.battleTargetEntity = -1;
```

语义：
- 敌实体在 `battleAnimTick==0` 那拍 `markEntityRemoved`（state=1，a.java:6349-6350）；
- `battleSceneRerun=false` 时胜利动画末尾自动 `tryStep(facingDirection)`（a.java:10108-10110，
  即"杀完继续走"）；case75 的重入场景为真时**不**续走；
- 67/68/74 → default `break`：**无收尾脚本**（A 级）。74 的脚本在击杀拍就已触发（R-boss-2）。

### R-boss-5: 脚本链门控三件套（tryRunScene / loadLevelScript / scriptLineFlags）

- 触发器 = 型 83"剧情点"：bump（a.java:5465-5471）与到达（a.java:5709-5715）两臂——

```java
// a.java:5465-5471（interactWithCell，bump，var2=false）
                        case 83:
                           if (this.tryRunScene(this.entityParam[var4], false)) {
                              this.overlayActive = false;
                              this.overlayText = null;
                              var6 = this.loadLevelScript(this.entityParam[var4]);
                              var8 = 0;
                           }
```

```java
// a.java:5709-5715（applyStepCellEffects，到达，var2=true）
                     case 83:
                        if (this.tryRunScene(this.entityParam[var1], true)) {
                           this.overlayActive = false;
                           this.loadLevelScript(this.entityParam[var1]);
                           var4 = true;
                           var5 = 0;
                        }
```

  ⇒ 门控参数 `var2` = **from_walk（到达=true，bump=false）**。
- `tryRunScene`（a.java:8068-8224）：`var3 = !scriptLineFlags[var1]`（首跑门）后按 case 叠加条件。
- `loadLevelScript`（a.java:8038-8066）：**不做任何门控**，直接 `gameMode=11` +
  `currentScriptIndex=var1` 装台词；`scriptLineFlags` 在脚本收尾置位：

```java
// a.java:8597-8604
      } else {
         this.dialogPhase = 4;
         if (this.currentScriptIndex != 32) {
            this.scriptLineFlags[this.currentScriptIndex] = true;
         }

         this.lockCameraOn(0);
      }
```

  或 `GUT` 跳转时置位（a.java:8310-8313）：

```java
// a.java:8310-8313
         } else if (var3.equals("GUT")) {
            var4 = this.parseScriptInt(var1, var2 + 1, " ");
            this.scriptLineFlags[this.currentScriptIndex] = true;
            this.loadLevelScript(var4);
```

- `LAY_x` = 出层/换层脚本指令（a.java:8355-8361）：

```java
// a.java:8355-8361
            if (var3.equals("LAY")) {
               this.m_119(this.currentFloor);
               this.pendingFloor = (byte)this.parseScriptInt(var1, var2 + 1, " ");
               this.scriptCursor = this.lastDelimPos + 1;
               this.floorTransitioning = true;
               this.overlayText = null;
               this.dialogPhase = 5;
```

  语义：存当前层状态（m_119）→ 置 `pendingFloor` → 开层转场（wipe），转场完成时
  `currentFloor = pendingFloor; loadFloorData(...)`（a.java:3187-3188）。**"出层"= LAY 触发的层切换**。

---

## 2. 67 啸天犬

### 2.1 战斗臂与数值（A 级）

- cat8（a.java:6281-6283）→ bump 战（R-boss-1）。无敌 trait（`traitFlagsByType[67]` 未在
  a.java:6266-6267 赋值 ⇒ 0）⇒ `effectiveAttackVsType` 恒 ×1（a.java:9992-10002）。
- 数值表（`enemyXxx[型-41]`，`enemyBaseHp` a.java:1044-1080 / `enemyBaseAtk` a.java:1081-1117 /
  `enemyBaseDef` a.java:1118-1120 / `enemyBaseGold` a.java:1121-1123）：
  idx26（=型67）：HP **1200**（a.java:1071 `1200,`）、Atk **180**、Def **20**、Gold **100**。
- 绘制特例：`dispatchSpriteSize` case 67 载图包 9（a.java:6303-6305）；多段拼装绘制
  a.java:6580-6603（`case 67:` 逐段 `drawImageClipped`）。非战斗语义，仅记录。

### 2.2 击杀钩子（A 级）→ spawnBossEvent(3)

`bossTypeOrder[3] == 67`（a.java:2198）⇒ 击杀拍 `spawnBossEvent(3)`（a.java:10063-10067）；
`bossEventSpawns[3]`（a.java:2143）：

```java
// a.java:2143
         {17, 6, 4, 0},
```

⇒ 刷 **金勺子(型17)** @格(6,4)，param 0，标 DES 态（R-boss-3）。
型特判 switch 无 case 67（a.java:10070-10086）；收尾 switch case 67 落 default（a.java:10134-10137）。

语义总结：**击杀啸天犬 = 金勺子(17) 的正常游玩来源**（bossEventSpawns 表内唯一 17 型条目 =
idx3；来源认定同 [type-closure.md](type-closure.md) §1.2 型 17 行；关卡二进制是否另有 17 型出生点
属 C 级待查）。金勺子用途 = 凿型11 墙，a.java:5443 `"障碍物：墙\n只有金勺子、玄明石可以凿开。或者剧情打开！"`）。
B 级呼应剧情：对话 64（a.java:1802 `"有，我这有把刚挖地道的破勺，你拿去挡住先！"`，牛魔王赠勺）。

### 2.3 出层/剧情语义（A 级）

- 剧情上 67 **可以不打**：对话 49-52（`dialogueSpeakerType` a.java:1244=72/1245=0/1246=72/1247=0；
  文本 a.java:1817 `"贤弟，这只啸天犬在这里挡道太危险了，跟哥哥绕密道吧。"`、
  a.java:1818 `"一只看门的宠物，能凶到哪里？"`、a.java:1819 `"不是，自从调到天庭守路，已经很多天没有吃荤腥了……"`、
  a.java:1820 `"了解，绕道，走~！"`）属 script5（a.java:1130 `"TAK_49_52 MOV_72_9_1_7_1 DES_72_7_1 "`）。
- script5 门控 = tryRunScene case 5 → 直通组（a.java:8092 `case 5:` 落 a.java:8121-8122 `default: break;`
  无附加条件）⇒ 仅 `!scriptLineFlags[5]` 首跑门；bump/到达皆可触发（型83 param5）。
- "密道"出层 = script0 尾段（a.java:1125）`... DES_70_5_8 LAY_2 ROS_1_4_7 ROS_2_0 ROS_2_6 RES_0 GUT_1 `
  的 **`LAY_2`**（出层到 2 层，R-boss-5）。

### 2.4 Rust 落点

- 已移植：`BOSS_TYPE_ORDER`/`BOSS_EVENT_SPAWNS`（scene.rs:130-135，注释含 a.java:2198/2041-2104）、
  `spawn_boss_event`（scene.rs:2999-3015）、击杀钩子（scene.rs:4757-4787，`rposition` 查表等价
  Java 的倒序扫）、收尾 switch（scene.rs:4826-4865）。
- 缺口：无（67 战+钩子全在共享臂内）。绘制多段拼装在渲染侧 todo!()
  （scene.rs:5603 `todo!("paintEntityLayer case 8 的 67 多段拼装")`）——非本任务面。

---

## 3. 68 太上老君 / 74 巨灵神

### 3.1 68 太上老君（A 级）

- 战斗臂同 R-boss-1。数值 idx27：HP **4500**（a.java:1072）、Atk **560**、Def **310**、Gold **1000**。
- 击杀钩子：`bossTypeOrder[2] == 68`（a.java:2198）⇒ `spawnBossEvent(2)`；表 a.java:2142：

```java
// a.java:2142
         {27, 6, 5, 0, 27, 5, 6, 0, 27, 7, 6, 0, 27, 6, 7, 0},
```

  ⇒ **红钥匙(27)×4** @格(6,5)(5,6)(7,6)(6,7)（以 boss 尸位为中心的十字）。
- 型特判 switch 无 case 68（a.java:10070-10086）；收尾 switch case 68 落 default（a.java:10135-10137）
  ⇒ **无脚本钩子、无出层链**。
- 语义总结：68 = 纯数值 boss（本批数值最高一档），击杀仅掉 4 红钥匙。剧情提示层（B 级）：
  a.java:831 `"太上老君就在25楼。以你现在的状态去攻击他简直就是自杀。……"`（教程文本，非脚本门控）。

### 3.2 74 巨灵神（A 级）

- 战斗臂同 R-boss-1 + **克制 trait**：

```java
// a.java:6266
      this.traitFlagsByType[49] = this.traitFlagsByType[53] = this.traitFlagsByType[74] = 1;
```

```java
// a.java:9992-10002
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

  ⇒ 持**太公杖(23)** 时对 74（及 49 巨力士、53 金刚力士）攻击 ×2；否则 ×1。
  B 级呼应：a.java:894 `"如果你持有太公杖，面对天神力士和巨灵神时你的攻击力加倍。在没有太公杖的情况下你是无法打败巨灵神的。……"`。
- 数值 idx33：HP **444**（a.java:1078）、Atk **199**、Def **66**、Gold **144**。
- 击杀钩子（双臂，A 级）：
  ① `bossTypeOrder[1] == 74` ⇒ `spawnBossEvent(1)`（12 件掉落，a.java:2092-2141：
  威力丹29@(4,4)(4,5)(5,4)、黄钥匙26@(7,4)(8,4)(8,5)、金刚丹30@(4,7)(4,8)(5,8)、
  长寿丹32@(8,8)(7,8)(8,7)；对照 idx0〔75 哪吒，a.java:2042-2091〕为
  29@(1,3)(2,3)(3,3)/26@(1,4)(2,4)(3,4)/30@(9,3)(10,3)(11,3)/32@(9,4)(10,4)(11,4)）；
  ② 型特判：

```java
// a.java:10078-10080
                           case 74:
                              var7.loadLevelScript(21);
                              break;
```

  `script21`（a.java:1146）= `"TAK_59_59 "` —— 遗言单页：文本 a.java:1827
  `"啊~~~天王，巨玲儿不能陪伴你了……"`（speaker 74，a.java:1254）。
- 出层条件：**`loadLevelScript(21)` 不经 tryRunScene** ⇒ 无首跑门、无 scriptLineFlags 门控，
  每次击杀 74 都会重放遗言（flags[21] 只在脚本收尾置位 a.java:8600，不回挡 loadLevelScript）。
  收尾 switch case 74 落 default（无第二段脚本）。
- 剧情上下文（B 级）：巨灵神登场戏 = script6（a.java:1131 `"TAK_53_58 "`，speaker 74 交替，
  a.java:1248-1253；文本 a.java:1821 `"我乃托塔天王麾下先锋官，巨玲神是也！你欺我孩儿……"` …）。

### 3.3 Rust 落点（68/74）

- 已移植：钩子双臂 scene.rs:4757-4786（`74 => self.load_level_script(21)`，scene.rs:4782）；
  trait 克制在 combat 侧 `effective_attack_vs_type`（含 `find_item(23)` 逻辑，与 a.java:9992-10002 同构）。
- 待补核对点：`load_level_script(21)` 的重复触发语义（每次击杀重放）需用例固定
  （无门控 = 行为合同的一部分，禁止"顺手加 flags 门"）。

---

## 4. 84 嫦娥 / 87 菩提老祖（出层链普查）

### 4.0 "出层"语义裁定（本文件口径）

`docs/spec/type-closure.md` 对 82/84/87 记"出层待普查"。本次取证把该词拆成两个可判定面：

1. **出场（spawn）面**：实体 84/87 从哪里生成（决定"它们在哪层出现"）；
2. **出层（LAY）面**：脚本链里 `LAY_x` 指令把玩家切出当前层（a.java:8355-8361，R-boss-5）。

### 4.1 84 嫦娥

**战斗臂：无。** cat32（A 级）：

```java
// a.java:6293-6294
      this.renderCategoryByType[84] = 32;
      this.renderCategoryByType[87] = 32;
```

bump 臂 case 32 只挡路（A 级）：

```java
// a.java:5527-5528
                  case 32:
                     var6 = false;
```

且 cat8 战臂/数值表按 `型-41` 索引（a.java:5484、9988-9989），84-41=43 越界（表长 35，
`enemyBaseHp` n=35，a.java:1044-1080）⇒ **84/74 之外的 84/87 在数值上不可能参战**（A 级表长证据）。
击杀钩子 switch（a.java:10070-10086）与收尾 switch（a.java:10115-10154）均无 case 84/87 ⇒ 无击杀钩子。

**出场源（3 处，A 级）：**

```java
// a.java:9642-9647（loadFloorData 恢复路径，floor==2 特例）
         switch (this.currentFloor) {
            case 2:
               if (!this.scriptLineFlags[26] && this.findBlockingEntity(11, 4, 84) < 0) {
                  this.spawnEntity(84, 352, 128, 0);
               }
               break;
```

- floor2 恢复钩子：`!flags[26]` 且 (11,4) 无活 84 ⇒ 刷 84 @像素(352,128)=格(11,4)，param 0
  （= 2 层天牢的被囚嫦娥；flags[26] 置位后不再刷）。
- script0（a.java:1125）头段 `CES_84_6_11 ` ⇒ 格(6,11) 刷 84（CES = spawnEntity + markEntityDesRemoved，
  a.java:8395-8396）。
- script19（a.java:1144）头段 `CES_84_7_7 MOV_84_7_7_6_8 ... DES_84_10_1 ...` ⇒ 格(7,7) 刷 84
  并随对白走位（MOV 五参实体路径）后于 (10,1) 移除。

**出层脚本链（tryRunScene 门控，A 级）：**

| scene | 脚本行（a.java:行） | tryRunScene 门控（a.java） | 内容 |
|---|---|---|---|
| 0 | 1125 `"CES_84_6_11 MOV_0_5_11 TAK_8_9 ... LAY_2 ... RES_0 GUT_1 "` | case 0：首跑时置 `flags[19]`（8076-8080），无 var2 约束 | 嫦娥登场戏（对话 8-21，speaker 84/0/70 交替，a.java:1196-1216）→ `DES_70_5_8` → **`LAY_2` 出层到 2 层** → `RES_0` → `GUT_1` 接 script1 |
| 19 | 1144 | case 19 → `var3 &= var2`（8128-8146 组，到达触发） | 嫦娥走位戏（对话 0-7） |
| 26 | 1151 `"TAK_129_132 DES_84_11_4 GIN_1_1000 "` | **case 26**（8182-8186）：`findBlockingEntity(11, 4, 84) < 0 ⇒ false`（嫦娥必须在位） | 2 层营救戏：对话 129-132（文本 a.java:1897 `"姑娘，你怎么会被关在这里！"` … a.java:1900 `"唉……你又何苦……"`，speaker 0/84）→ `DES_84_11_4` 除嫦娥 → `GIN_1_1000` +1000 金 |
| 27 | 1152 `"TAK_113_117 TAK_133_135 DES_84_6_11 GIN_0_13 "` | case 27（8187-8189）：`var3 &= var2 & scriptLineFlags[0]`（**到达触发且 script0 已跑**） | 对话后 `DES_84_6_11` 除嫦娥 → `GIN_0_13` 给火眼金睛(13) |
| 28 | 1153 `"TAK_136_143 DES_84_1_11 TAK_144_144 GIN_0_19 "` | case 28 → 直通组（8092-8122） | `DES_84_1_11` → `GIN_0_19` 给千年月桂露(19) |
| 29 | 1154 `"TAK_145_146 DES_84_9_8 GIN_1_1000 "` | case 29 → 直通组 | `DES_84_9_8` → `GIN_1_1000` +1000 金 |
| 30 | 1155 `"MOV_84_6_3_4_3 MOV_84_4_3_8_3 MOV_84_8_3_6_3 TAK_107_107 "` | case 30 → `var3 &= var2`（8128-8146 组） | 嫦娥三段走位戏 |
| 31 | 1156 `"TAK_108_108 DES_10_6_6 MOV_0_6_5 TAK_109_112 TAK_147_148 DES_84_6_3 TAK_149_149 "` | case 31（8190-8193）：`var3 & var2` 且 `findItemStackIndex(16) >= 0`（持芭蕉扇） | 终幕：除 84@(6,3) 等 |

（`GIN` 语义 A 级：a.java:8424-8437 `case 0: this.pickupItemType(var48)` / `case 1: this.gainGold(...)`。）

**flags[26] 的双重角色（A 级）**：scene26 首跑门（8182-8186 的 `!scriptLineFlags[26]`）+
floor2 恢复钩子的补刷开关（9644）——置位途径 = 脚本收尾（a.java:8600）。

### 4.2 87 菩提老祖

**战斗臂：无**（同 4.1，cat32 + 数值表越界）。**击杀钩子：无**（同上）。

**实体出生源：语料无此分支 = 死码/数据侧候选（B 级，grep 0 命中证明）：**

```bash
$ grep -n "spawnEntity(87\|CES_87" reference/src/deobf/a.java   # 0 命中
$ grep -n "\b87\b" reference/src/deobf/a.java | grep -v 对话表
2248: this.logoSegmentHeights = new byte[]{87, 18, 9};   # logo，无关
2665/3815: case 87:  # gameMode 大 switch 的 default 簇，无关
6294: this.renderCategoryByType[87] = 32;
8672: if (var1 != 0 && var1 != 87) {
9478: var17.entityTypeImage[87] = var17.f_Image_arr2_00[13][19];
```

⇒ a.java 侧 87 **没有任何 spawn**（脚本 CES 无、bossEventSpawns 值域无〔{16,17,24,26,27,29,30,32}〕、
无层恢复钩子）。仅存的三类用途：
1. 渲染注册：`renderCategoryByType[87]=32`（6294）、`entityTypeImage[87]=f_Image_arr2_00[13][19]`
   （9478，= sptenemy2[19] 图）；
2. **对话发言者**：`dialogueSpeakerType` 大段 87（a.java:1358-1423、1450-1463 等），含
   序章教学（对话 163 起：a.java:1931 `"欢迎你来到天宫世界，我是你的师傅菩提老祖。"`，
   speaker a.java:1358）与混沌世界引子（对话 264-268：a.java:2032-2036
   `"我的徒儿啊，你已经陷入混沌世界。……"` … `"去吧……劫难在所难免。"`，speaker a.java:1459-1463）；
3. 相机锚定**显式跳过**（A 级）：

```java
// a.java:8672
      if (var1 != 0 && var1 != 87) {
```

  ⇒ `lockCameraOn(87)` 不寻实体、不移相机（= 87 出场时镜头留在原处）。
  这解释了 87 无出生源仍能发言的机制（B 级结论，A 级代码支撑）。

**出层脚本链（A 级）：**

| scene | 脚本行（a.java:行） | tryRunScene 门控（a.java） | 内容 |
|---|---|---|---|
| 40 | 1165 `"TAK_255_259 TAK_165_165 SEE_3_10_166_166_1 "` | case 40 → 直通组（8092-8122） | 序章教学入口（对话 255-259/165，speaker 87） |
| 41-45,48-50,52,53 | 1166-1178 | `var3 &= var2`（8128-8146 组，**仅到达触发**） | 教学镜头族（SEE） |
| 66 | 1191 `"TAK_261_263 SMS_0 "` | case 66：`var3 = false`（8216-8217）**恒不触发** | 保险/短信残迹（死码候选，同 type-closure §2 SMS 行） |
| 67 | 1192 `"TAK_264_268 "` | **case 67**（8219-8220）：`var3 &= this.difficultyIndex > 0` | 混沌世界引子（对话 264-268，speaker 87 全段） |

**case 67 门控的可达性（B 级，赋值集审计）**：`difficultyIndex` 全文赋值仅 3 处——
a.java:2233 `this.difficultyIndex = 0;`（初始化）、a.java:7217 `this.difficultyIndex = 0;`
（resetNewGameState）、a.java:9258 `this.difficultyIndex = this.f_DataInputStream_00.readByte();`
（读档，而存档侧 a.java:9190 写的就是当前值）。⇒ 正常游玩恒 0 ⇒ **scene67 的混沌引子当前不可达**
（除非改存档）；`difficultyMultipliers`（a.java:2234 `{1,30,60,80,100}`）随其缩放敌人
（a.java:9313）。**待终裁**：按死码候选流程（docs/spec/dead-code-rulings.md 格式）裁 scene67。

### 4.3 Rust 落点（84/87）

- 已移植：tryRunScene case 26/27/67（scene.rs:1501-1503 / 1504 / 1528）；
  floor2 补刷钩子（scene.rs:928-936 与 1365-1369 双路径 + 测试
  `floor2_restore_respawns_84_only_while_script26_pending` scene.rs:6700-6706）；
  DSL 臂 LAY/GIN/RES/GUT/CES/DES（script.rs:333-441 / scene.rs:1793-2001）。
- 待办：scene67 死码终裁（4.2）；87 无出生源的裁定写入 dead-code-rulings 或 type-closure 出层列销账。

---

## 5. 对照组（钩子族语义佐证，非本批）

| 型 | 击杀钩子（A 级） | 行号 |
|---|---|---|
| 71 玉帝二阶段 | `bossTypeOrder[5]=71` ⇒ `spawnBossEvent(5)`（24 捆仙绳 + 32×3 + 27）；收尾 switch floor49 → `loadLevelScript(35)` | a.java:2198、2145、10144-10148 |
| 73 天蓬元帅 | `flags[currentScriptIndex]=true` + `loadLevelScript(22)`；floor40 加 `spawnBossEvent(6)` | a.java:10071-10077 |
| 75 哪吒 | `bossTypeOrder[0]=75` ⇒ `spawnBossEvent(0)`；`tryRunScene(20,true)` 过门 → `loadLevelScript(20)` + `battleSceneRerun=true`（唯一重入臂） | a.java:10081-10085 |
| 69 杨戬 | `bossTypeOrder[4]=69` ⇒ `spawnBossEvent(4)`（16 芭蕉扇）；收尾 switch floor35 → `spawnBossEvent(7)` + `loadLevelScript(36)` | a.java:2144、10138-10143 |
| 70 玉帝一阶段 | 收尾 switch floor49 → `loadLevelScript(35)` | a.java:10144-10148 |
| 72 牛魔王 | 收尾 switch floor50 → `loadLevelScript(65)` | a.java:10150-10153 |

钩子族语义总结：**bossTypeOrder→spawnBossEvent(idx) 是"掉落臂"（值域=道具/钥匙）；
型特判 switch 是"剧本臂"（loadLevelScript 直载或 tryRunScene 门控后载）；两臂独立、可叠加
（74 = 掉落+剧本双臂）**。50 层以上（混沌世界）全部钩子跳过（a.java:10058）。

## 6. 遗留假设（C 级，不入结论）

1. `entityState=2` 的命名（DES 态）与 `markEntityDesRemoved` 的原始语义——本文件只描述其
   可观察效果（state=2、27×29/row8 重置、可交互），"DES=剧情生成物"为 B 级语境推断。
2. 87 在关卡二进制数据中是否有 spawn（loadFloorData 通用臂 a.java:9590 可刷任意型）——
   a.java 静态不可见，需 L1 资源普查补证。
3. scene67 死码判定（4.2）在"作弊存档"路径下可解除——终裁时按 dead-code-rulings 口径注明边界。

## 7. 取证命令备忘（可复现）

```bash
grep -n "spawnBossEvent\|case 67\|case 68\|case 74\|case 84\|case 87" reference/src/deobf/a.java
grep -n "bossTypeOrder\|bossEventSpawns\|tryRunScene\|loadLevelScript\|scriptLineFlags" reference/src/deobf/a.java
grep -n "spawnEntity(87\|CES_87" reference/src/deobf/a.java   # 0 命中 = 87 无出生源
sed -n '10027,10160p' reference/src/deobf/a.java               # tickBattle 击杀钩子全貌
sed -n '2041,2198p' reference/src/deobf/a.java                 # bossEventSpawns + bossTypeOrder
```

---

## 补遗（2026-10-16 lead 审校裁决）

- **scene67 死码候选**（difficultyIndex 正常游玩恒 0 ⇒ 混沌引子 TAK_264_268 不可达）：终裁移交 task-6 走 `python3 tools/dead_code_ruling.py` 字节码 CFG 不动点证明后，按流程登记 ☠️ 或补用例。**不在此口头终裁**（AGENTS.md：死码要有证明）。
- **型 87 出生源空缺**：grep `spawnEntity(87|CES_87` = 0 命中（A 级证明已存档于正文取证命令备忘）——账本 87 行销账时写"纯对话发言者，无实体出生源（grep 0 命中存证）"。
- 行号抽查 3/3 真（a.java:5475/2143/10078 逐字吻合）。
