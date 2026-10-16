# 证据附件：七件道具使用效果臂（型 15/17/18/19/23/24/25）

> 角色：spec-writer 取证产物。证据源：`reference/src/deobf/a.java`（10883 行）。
> 方法：grep 定位 case 标签 → sed/awk 读上下文 → 逐字摘录，每条结论带行号指针。
> 证据分级：本文全部条目均为 **A 级**（反编译原文行号 + 逐字摘录）；推断处显式标注 B 级，不入结论。
> 取证日期：2026-10-16。行号以当日 `reference/src/deobf/a.java` 为准。

---

## 0. 结论速查表（7 行账本）

| 型 | 名称 | activateItem 使用臂 | 次数（itemUseCounts） | 实际效果 | 关键证据 |
|---|---|---|---|---|---|
| 15 | 筋斗云 | **无**（7795 fall-through `default: break`，返回 false） | -1 = 无限（944） | **被动**：拾取即置 `tutorialSeen=true`（7533），此后贴近楼梯时 `f_bool_13` 生效（5772-5806）→ 快捷键 49/55 经楼梯口瞬移上下层（5295-5305）+ HUD 楼梯标记（2406-2414） | a.java:909/7532-7533/5771-5778/5295-5305 |
| 17 | 金勺子 | 有（7809-7816） | 1 次（944） | 命中时 `m_089(11)` 移除**四邻格全部**型 11「能挖的墙」实体（7895-7919）；无墙则提示"你必须面对一堵墙使用"且**不消耗**；成功置 `gameMode=3` 并消耗 | a.java:7809-7816/7895-7919/966/911 |
| 18 | 玄明石 | 有（7817-7828） | 1 次（944） | 移除**当前层全部**型 11「能挖的墙」实体（`entityType==11` 全表扫描）+ `gameMode=3`，恒成功消耗 | a.java:7817-7828/912 |
| 19 | 千年月桂露 | 有（7829-7834） | 1 次（944） | 回血 `(playerAtk+playerDef)*74/10`（= 攻+防 的 740%）+ 提示"增加了N血量"，恒成功消耗 | a.java:7829-7834/913 |
| 23 | 太公杖 | **无**（7796 fall-through） | -1 = 无限（944） | **被动**（放在道具栏中有效）：敌人 `traitFlagsByType & 1` 时攻击力 ×2（`effectiveAttackVsType`） | a.java:7796/9992-10002/917/894 |
| 24 | 捆仙绳 | **无**（7797 fall-through） | -1 = 无限（944） | **被动**（放在道具栏中有效）：敌人 `traitFlagsByType & 2` 时攻击力 ×2；另为玉皇大帝击杀链掉落物（spawnBossEvent(5)） | a.java:7797/9992-10002/918/900/2145 |
| 25 | 幸运币 | **无**（7798 fall-through） | -1 = 无限（944） | **被动**（放在道具栏中有效）：打怪金钱 ×2（`enemyBaseGold<<1`，战斗击杀与 killAdjacentAt 两处） | a.java:7798/10046-10052/7944-7948/919/898 |

**一句话裁定**：17/18/19 是主动使用道具（成功才消耗 1 次）；15/23/24/25 的 `activateItem` 臂是**空操作**——15 拾取置位即生效，23/24/25 靠"放在道具栏中"的被动判定，四件均无限次、永不消耗。

---

## 1. 公共机制（A 级）

### 1.1 型号 ↔ 栈下标 ↔ 名称

- 栈下标映射 `itemTypeToStackIndex`（85/86 特判，其余 `type-13`）：

```
7705:    private static int itemTypeToStackIndex(int var0) {
7707:       switch (var0) {
7708:          case 85:
7709:             var1 = 20;
7711:          case 86:
7712:             var1 = 21;
7714:          default:
7715:             var1 = var0 - 13;
```

- 名称表 `objectTypeNames`（下标=型 ID；型 11 即「能挖的墙」）：

```
966:          "能挖的墙",
970:          "筋斗云",
972:          "金勺子",
973:          "玄明石",
974:          "千年月桂露",
978:          "太公杖",
979:          "捆仙绳",
980:          "幸运币",
```

（数组自 a.java:955 `孙悟空` 起，下标 = 行号 − 955：966→11、970→15、972→17、973→18、974→19、978→23、979→24、980→25。）

### 1.2 使用次数表（**有符号** byte，-1 = 无限）

```
944:       this.itemUseCounts = new byte[]{-1, -1, -1, -1, 1, 1, 1, 3, 1, 1, -1, -1, -1, 1, 1, 1, 1, 1, 1, 1, 1, 1};
```

下标 = `type-13`：15→idx2→**-1**；17→idx4→**1**；18→idx5→**1**；19→idx6→**1**；23→idx10→**-1**；24→idx11→**-1**；25→idx12→**-1**。
与描述文案逐件自洽（909/911/912/913/917/918/919 各自印有"[使用次数：无限/1次]"或"[放在道具栏中有效]"）。

### 1.3 入栈 `addItemToItemStack`（叠用累加）

```
7721:    private void addItemToItemStack(int var1) {
7722:       int var2 = this.findItemStackIndex(var1);
7723:       int var3 = itemTypeToStackIndex(var1);
7724:       if (var2 < 0) {
7725:          this.itemStackTypes[this.itemStackSize] = (byte)var1;
7726:          this.itemStackUses[this.itemStackSize] = this.itemUseCounts[var3];
7727:          this.itemStackSize++;
7728:       } else {
7729:          this.itemStackUses[var2] = (byte)(this.itemStackUses[var2] + this.itemUseCounts[var3]);
7730:       }
7732:       this.itemUseTargetType = (byte)var1;
7733:       this.showOverlayMessage((byte)1, this.itemDescriptions[var3], (byte)0, (byte)3);
```

### 1.4 使用入口与消耗规则 `useItemStack`

两个调用点：道具面板确认（overlay case -6 → `useItemStack(itemPanelCursor)`，3300-3303）与游戏中快捷使用（`useItemStack(f_int_96)`，3647-3652）。

```
7764:    private void useItemStack(int var1) {
7765:       if (this.itemStackUses[var1] > 0) {
7766:          if (this.activateItem(this.itemStackTypes[var1]) && --this.itemStackUses[var1] == 0) {
7768:             for (int var4 = var1; var4 < this.itemStackSize - 1; var4++) {
7769:                this.itemStackTypes[var4] = this.itemStackTypes[var4 + 1];
7770:                this.itemStackUses[var4] = this.itemStackUses[var4 + 1];
7771:             }
7773:             if (this.itemStackSize > 0) {
7774:                this.itemStackSize--;
7775:             }
7777:             return;
7778:          }
7779:       } else {
7780:          this.activateItem(this.itemStackTypes[var1]);
7781:       }
```

消耗语义（A 级原文 + 可判定）：
- `itemStackUses > 0`（17/18/19）：**仅当 `activateItem` 返回 true** 才 `--itemStackUses`；减到 0 时紧凑栈并 `itemStackSize--`。失败（如 17 无邻墙）**不消耗**（7766 短路）。
- `itemStackUses <= 0`（-1 即 15/23/24/25）：走 7779 else 分支无条件 `activateItem`，**永不减次、永不移出栈**。注意 byte 有符号：-1 若按 u8 读成 255 会误走计数分支（Rust 侧已在 use_item_stack 显式 `as i8`，见 §2 落点）。

### 1.5 道具面板预览分流（3937-3965）

```
3941:                                  switch (var21.itemUseTargetType) {
3942:                                     case 13:
3943:                                     case 14:
3944:                                        var21.gameMode = 3;
3945:                                        var21.activateItem(var21.itemUseTargetType);
3947:                                     case 15:
3948:                                     case 23:
3949:                                     case 24:
3950:                                     case 25:
3951:                                        var21.showOverlayMessage(
3952:                                           (byte)3, var21.itemDescriptions[itemTypeToStackIndex(var21.itemUseTargetType)], (byte)0, (byte)3
3953:                                        );
3955:                                     case 16:
3956:                                     case 17:
3957:                                     case 18:
3958:                                     case 19:
...
3963:                                        var21.showOverlayMessage(
3964:                                           (byte)3, var21.itemDescriptions[itemTypeToStackIndex(var21.itemUseTargetType)], (byte)1, (byte)2
3965:                                        );
```

15/23/24/25 与 16-22 都进 kind-3 说明/确认 overlay（参数 (0,3) vs (1,2) 的差异语义未考证，见 §4 B 级），确认键同样落到 `useItemStack`（3300-3303）。

---

## 2. 逐件取证

### 2.1 型 15 筋斗云 — 使用臂空操作；拾取置位的被动"楼梯瞬移"能力

**A 级证据**

使用臂（fall-through 到 default，返回 false，不消耗）：

```
7795:          case 15:
7796:          case 23:
7797:          case 24:
7798:          case 25:
7799:          default:
7800:             break;
```

拾取置位（唯一写点，叠加入栈）：

```
7532:          case 15:
7533:             this.tutorialSeen = true;
7534:          case 13:
...
7548:             this.addItemToItemStack(var1);
```

`tutorialSeen` 的消费点——贴梯置 `f_bool_13`（每步重算，只有 tutorialSeen 为真才可能置真）：

```
5771:       var21.f_bool_13 = false;
5772:       if (var21.tutorialSeen) {
5773:          if (var6 > 0) {
5774:             if (var2 == var7f && var6 - 1 == var43) {
5775:                var21.f_bool_13 = true;
```

`f_bool_13` 的两个消费点——楼梯口快捷键瞬移（49=上行，55=下行）与 HUD 楼梯标记：

```
5295:             case 49:
5296:                this.keyValue = 0;
5297:                if (this.f_bool_13 && (var1 = this.findFloorGateEntity(true)) >= 0) {
5298:                   this.changeFloor(this.entityParam[var1] & 255, false, true);
5301:             case 55:
5303:                if (this.f_bool_13 && (var1 = this.findFloorGateEntity(false)) >= 0) {
5304:                   this.changeFloor(this.entityParam[var1] & 255, true, true);
```

```
2406:             if (this.f_bool_13) {
2407:                if (var32.stairUpMarkerX >= 0) {
2408:                   var32.drawImageClipped(var32.f_Image_arr2_00[8][19], 104, 20, 0, 0, 16, 16);
```

道具描述（自证语义 + 无限次）：

```
909:          "在楼梯边，可以瞬间上下层，留神晕机。\n\\c00ff00[使用次数：无限]",
```

**语义总结**：不改任何数值字段；拾取（pickupItemType case 15）置 `tutorialSeen=true`（一次性标志，7533）后永久生效。效果 = 位于楼梯相邻格（`f_bool_13`）时可用快捷键 49/55 沿 `findFloorGateEntity` 的楼梯口瞬移上下层，并在 HUD 画楼梯方向标记。使用臂无操作、`itemStackUses=-1` 永不消耗。使用条件：`tutorialSeen` 已置位 + 玩家格与楼梯标记格相邻（上下左右四向，5773-5806）。

**Rust 落点**：`pickup_item_type` → `crates/game-core/src/scene.rs:5154-5157`（`if t == 15 { self.tutorial_seen = true; }`，已落地）；字段 `scene.rs:410`。使用臂空操作 = `activate_item`（scene.rs:2800）match 缺省分支。被动消费点落 `scene.rs:4382`（`find_floor_gate_entity(up)` 快捷键路径）与楼梯标记绘制；`tutorial_seen` 的贴梯判定（a.java:5771-5806 → `f_bool_13` 等价物）落 scene.rs 每步重算处（若尚未落地需补）。

### 2.2 型 17 金勺子 — 凿开四邻格全部「能挖的墙」

**A 级证据**

使用臂：

```
7809:          case 17:
7810:             if (!this.m_089((byte)11)) {
7811:                this.showOverlayMessage((byte)0, "你必须面对一堵墙使用", (byte)0, (byte)0);
7812:             } else {
7813:                this.gameMode = 3;
7814:                var2 = true;
7815:             }
7816:             break;
```

`m_089(11)` 的真实行为——**四邻格**（非朝向）找型 11 并全部移除，命中计数>0 才返回 true：

```
7895:    private boolean m_089(byte var1) {
7896:       int var2 = this.playerPixelX >> 5;
7897:       int var3 = this.playerPixelY >> 5;
7898:       int var4 = this.findBlockingEntity(var2, var3 - 1, var1);
7899:       int var5 = this.findBlockingEntity(var2, var3 + 1, var1);
7900:       int var6 = this.findBlockingEntity(var2 - 1, var3, var1);
7901:       int var7 = this.findBlockingEntity(var2 + 1, var3, var1);
7902:       var2 = 0;
7903:       if (var4 >= 0) {
7904:          this.markEntityRemoved(var4);
7905:          var2++;
...
7919:          this.markEntityRemoved(var7);
```

型 11 名称与道具描述：

```
966:          "能挖的墙",
911:          "挖洞开墙越狱的利器,挫是挫了点，但是真的很好用。\n\\c00ff00[使用次数：1次]",
```

墙的官方文案（与 18 联动自证）：

```
5443:                               this.showOverlayMessage((byte)0, "障碍物：墙\n只有金勺子、玄明石可以凿开。或者剧情打开！", (byte)0, (byte)0);
897:          "存放乌金棍的房间的门坏了，你必须用金勺子破墙而入。",
```

**语义总结**：使用后 `m_089(11)` 移除玩家上下左右四邻格中**所有**型 11「能挖的墙」实体（不是只凿一堵、也不是按朝向）；至少一堵被移除才置 `gameMode=3`、返回 true、消耗 1 次（`itemStackUses` 1→0 出栈）。四邻无墙：提示"你必须面对一堵墙使用"，返回 false，**不消耗**。quirk：提示语说"面对"，实际判定是四邻格（BUG 候选，Stage A 保真）。

**Rust 落点**：`activate_item` 的 `16 | 17` 臂 → `crates/game-core/src/scene.rs:2816-2831`（`remove_adjacent_entity_of_type(11)` @ scene.rs:2980 + 未命中提示 + 消耗语义，已落地）；消耗由 `use_item_stack`（scene.rs:3017-3041）统一处理。

### 2.3 型 18 玄明石 — 震开当前层所有墙

**A 级证据**

使用臂（全表扫描移除型 11）：

```
7817:          case 18:
7818:             int var1c = this.entityCount;
7820:             while (--var1c >= 0) {
7821:                if (this.entityType[var1c] == 11) {
7822:                   this.markEntityRemoved(var1c);
7823:                }
7824:             }
7826:             this.gameMode = 3;
7827:             var2 = true;
7828:             break;
```

道具描述 + 剧情用途 + 商人获取链（报价码 13，4000 金）：

```
912:          "可以震开当前层所有的墙\n\\c00ff00[使用次数：1次]",
838:          "你需要用“玄明石”取出37楼仓库内的所有宝物。",
885:          "我有个宝物，要收你4000金币，是个玄明石。"
7430:                      case 13:
7431:                         if (this.spendGold(4000)) {
7432:                            this.pickupItemType(18);
7433:                         } else {
7434:                            this.showOverlayMessage((byte)0, "没有足够的金钱", (byte)0, (byte)0);
```

**语义总结**：使用后遍历 `entityCount` 移除**当前层全部**型 11「能挖的墙」实体，置 `gameMode=3`，恒成功返回 true，消耗 1 次出栈。无使用条件（不需要贴墙）。获取：天宫商人报价码 13 花 4000 金（`applyMerchantOffer` 7430-7437）直接 `pickupItemType(18)` 入栈。

**Rust 落点**：`activate_item` 的 `18` 臂 → `crates/game-core/src/scene.rs:2832-2845`（全表移除型 11 + mode3 + 消耗，已落地）；商人链落 `applyMerchantOffer` 对应报价分支（pickup_item_type @ scene.rs:5148）。

### 2.4 型 19 千年月桂露 — 按攻+防 740% 回血

**A 级证据**

使用臂：

```
7829:          case 19:
7830:             int var2hp = (this.playerAtk + this.playerDef) * 74 / 10;
7831:             this.applyHpDelta(var2hp);
7832:             this.showOverlayMessage((byte)0, "增加了" + var2hp + "血量", (byte)0, (byte)0);
7833:             var2 = true;
7834:             break;
```

道具描述与剧情授予语（自证"越晚用越好"= 攻防越高等式越大）：

```
913:          "喝下后，增加相当于当前\\c00FFFF攻击力\\cFFFFFF加\\c00FFFF防御力\\cFFFFFF值740%的\\cFFCC00血量\n[月宫出品，手工酿制，不含三聚氰胺，冷藏效果更佳，使用次数：1次]",
852:          "很好，你居然找到了我，作为奖励我将给你一瓶千年月桂露，喝了它将按你的攻击力和防御力的综合增加的你生命点数，你越晚使用它效果越好。",
```

**语义总结**：使用后 `applyHpDelta((playerAtk+playerDef)*74/10)`（整数除法，74/10=7.4 倍即文案"740%"），弹出"增加了N血量"提示，恒成功返回 true，消耗 1 次出栈。无使用条件。数值随攻防成长，属"延迟使用更优"的资源决策。

**Rust 落点**：`activate_item` 的 `19` 臂 → `crates/game-core/src/scene.rs:2846-2852`（`(atk+def)*74/10` + apply_hp_delta + 提示，已落地）。注意 Java `int` 乘法溢出语义与 Rust i32 一致即可，勿改 i64。

### 2.5 型 23 太公杖 — 被动：对 trait 位 1 敌人攻击加倍

**A 级证据**

使用臂（空操作）：a.java:7796 同 §2.1 fall-through 块（7795-7800）。

被动判定（在背包即生效，与是否"使用"无关）：

```
9992:    private int effectiveAttackVsType(int var1) {
9993:       byte var3 = this.traitFlagsByType[var1];
9994:       byte var2 = 1;
9995:       if ((var3 & 1) != 0 && this.findItemStackIndex(23) >= 0) {
9996:          var2 = 2;
9997:       } else if ((var3 & 2) != 0 && this.findItemStackIndex(24) >= 0) {
9998:          var2 = 2;
9999:       }
10001:       return this.playerAtk * var2;
```

道具描述与剧情文本：

```
917:          "当年姜子牙受天命封神，他的钓鱼竹竿被原始天尊附上了神力，可以役使天神力士供他差遣，此杖又名“打神鞭”，对天神力士（包括巨灵神）威力加倍。\n\\cFFCC00[放在道具栏中有效]",
894:          "如果你持有太公杖，面对天神力士和巨灵神时你的攻击力加倍。在没有太公杖的情况下你是无法打败巨灵神的。太公杖被藏在15楼以上的墙内。",
```

**语义总结**：使用臂空操作；`itemStackUses=-1` 永不消耗。效果 = 被动：`findItemStackIndex(23)>=0`（在道具栈中）且敌人 `traitFlagsByType[t] & 1 != 0` 时 `effectiveAttackVsType` 返回 `playerAtk*2`（否则 `playerAtk`）。该值进入战斗伤害与战损预测（tickBattle / predictHpLossVsType）。文案点名适用对象为天神力士与巨灵神（型 74 在 bossTypeOrder，a.java:2198/1029）。"藏在 15 楼以上的墙内"与 17/18 凿墙机制联动（893 楼层提示）。

**Rust 落点**：`crates/game-core/src/combat.rs:66-72`（`effective_attack_vs_type`，`TRAIT1_AMULET: u8 = 23` @ combat.rs:145，已落地）；消费点 scene.rs:4674/4704。

### 2.6 型 24 捆仙绳 — 被动：对 trait 位 2 敌人攻击加倍；玉皇大帝掉落链

**A 级证据**

使用臂（空操作）：a.java:7797 同 §2.1 fall-through 块（7795-7800）。
被动判定：a.java:9997-9998（`(var3 & 2) != 0 && findItemStackIndex(24) >= 0 → var2 = 2`，见 §2.5 全文）。

道具描述与剧情文本：

```
918:          "对某些自恋的神仙伤害加倍。\n\\cFFCC00[放在道具栏中有效]",
900:          "如果要打败杨戬你需要“乌金棍”、“银缕锁甲”、“捆仙绳”或更高等级的宝物。"
1929:          "?有根捆仙绳？似乎可以克制住那个三眼小白脸，恩，搞定他，捆绑他，拿他的芭蕉扇，哦也~",
```

获取链——战斗击杀 Boss 分发（10061-10067）→ `spawnBossEvent(5)` → 掉落表第 5 行（2145）spawn 型 24 实体：

```
10061:                         int var4 = var7.bossTypeOrder.length;
10063:                         while (--var4 >= 0) {
10064:                            if (var8 == var7.bossTypeOrder[var4]) {
10065:                               var7.spawnBossEvent(var4);
10066:                               break;
2198:       this.bossTypeOrder = new byte[]{75, 74, 68, 67, 69, 71};
2145:          {24, 6, 2, 0, 32, 5, 1, 0, 32, 6, 1, 0, 32, 7, 1, 0, 27, 6, 6, 0},
8906:    private void spawnBossEvent(int var1) {
8911:       for (int var4 = 0; var4 < var2; var4 += 4) {
8912:          var3 = this.spawnEntity(var5[var4], var5[var4 + 1] << 5, var5[var4 + 2] << 5, var5[var4 + 3]);
```

`bossTypeOrder[5] = 71`，型 71 名称 = 玉皇大帝（`objectTypeNames` 下标 = 行号−955：1026→71）：

```
1025:          "玉皇大帝",
1026:          "玉皇大帝",
```

**语义总结**：使用臂空操作、永不消耗。效果 = 被动：`findItemStackIndex(24)>=0` 且敌人 `traitFlagsByType[t] & 2 != 0` 时攻击力 ×2。获取：击杀 bossTypeOrder 中型 71（玉皇大帝）→ `spawnBossEvent(5)` → 在格 (6,2) spawn 型 24（捆仙绳）拾取实体（同行附带 3 个型 32 长寿丹 + 1 个型 27 红钥匙），实体被触碰后走 `pickupItemType(24)` 入栈（7544）。

**Rust 落点**：`crates/game-core/src/combat.rs:66-72`（`TRAIT2_AMULET: u8 = 24` @ combat.rs:147，已落地）；掉落链落 boss 击杀分发（scene.rs tick_battle 击杀臂）+ `bossEventSpawns[5]` 数据表 + `spawnBossEvent` spawn 循环（a.java:8906-8914 → Rust spawn_boss_event 等价物）。

### 2.7 型 25 幸运币 — 被动：打怪金钱 ×2

**A 级证据**

使用臂（空操作）：a.java:7798 同 §2.1 fall-through 块（7795-7800）。

被动判定——战斗击杀掉落（tickBattle 击杀分支）：

```
10046:                      if (this.findItemStackIndex(25) >= 0) {
10047:                         this.gainGold(
10048:                            this.enemyBaseGold[var1t - 41] << 1, this.entityPixelX[this.battleTargetEntity], this.entityPixelY[this.battleTargetEntity]
10049:                         );
10050:                      } else {
10051:                         this.gainGold(this.enemyBaseGold[var1t - 41], this.entityPixelX[this.battleTargetEntity], this.entityPixelY[this.battleTargetEntity]);
```

被动判定——`killAdjacentAt`（道具 86 日月无极钹击杀路径，a.java:7883-7886 调用）：

```
7942:                } else if (var6 < 67 && this.renderCategoryByType[var6] == 8 && this.entityState[var5] != 1) {
7943:                   this.markEntityRemoved(var5);
7944:                   if (this.findItemStackIndex(25) >= 0) {
7945:                      this.gainGold(this.enemyBaseGold[var6 - 41] << 1, 0, 0);
7946:                   } else {
7947:                      this.gainGold(this.enemyBaseGold[var6 - 41], 0, 0);
```

道具描述与剧情文本（自证"2 倍金钱"）：

```
919:          "打怪得到的金钱加倍。\n\\cFFCC00[放在道具栏中有效]",
898:          "天宫中藏有有个“幸运金币”拥有它在打败敌人后能够获得2倍的金钱。",
```

**语义总结**：使用臂空操作、永不消耗。效果 = 被动：`findItemStackIndex(25)>=0` 时敌人掉落金 `enemyBaseGold[t-41] << 1`（×2），否则原值。共两个掉落点：正常战斗击杀（10046-10052）与 `killAdjacentAt` 击杀（7944-7948）。不改 gold 之外的字段；`gainGold` 另弹金币 popup（a.java:7521-7525）。

**Rust 落点**：`crates/game-core/src/scene.rs:2939-2944`（kill_adjacent_at 内 `find_item(25)` 判定，已落地）与 `scene.rs:4739`（战斗击杀金判定，已落地）。

---

## 3. 边界与异常路径（差分富矿）

1. **失败不消耗**（17）：四邻无型 11 → `activateItem` 返回 false → 7766 短路，`itemStackUses` 不减、不出栈。可判定：快照 `itemStackUses` + overlay 文本。
2. **无限次不计数**（15/23/24/25）：`itemStackUses=-1` 走 7779 else 分支，只调 `activateItem`（本身空操作），无任何状态变化。可判定：整栈快照逐 tick 不变。
3. **-1 有符号性**：`itemStackUses` 为 byte，比较 `> 0` 是有符号比较；按 u8 读 255 会误走计数分支（Rust `use_item_stack` 已 `as i8`，scene.rs:3020）。
4. **17 vs 18 范围差**：17 = 四邻格（m_089，可能同时凿多堵）；18 = 全层所有型 11。二者效果域不同，勿互相"优化"。
5. **m_089 的四邻对称性**：17 提示语"面对一堵墙"与实现（上下左右全查全删）不符（BUG 候选，Stage A 保真，禁止改成朝向判定）。
6. **19 的整数除法**：`(playerAtk+playerDef)*74/10` 先乘后除，勿化简为 `*7.4`。
7. **25 的双掉落点**：只有 tickBattle 与 killAdjacentAt 两处 `enemyBaseGold` 结算，均翻倍；其他 `gainGold`（开箱/剧情/卖钥匙）不翻倍。
8. **23/24 的 else-if 互斥**：9995-9998 是 if/else-if，敌人同时带 trait 位 1|2 时结果同为 ×2，不叠加为 ×4。

## 4. 遗留假设（B 级，不入结论）

- `traitFlagsByType` 位 1 = 天神力士/巨灵神类、位 2 = 杨戬类：文案（894/917/918/900）与 if/else-if 结构自洽，但位表逐敌人未核对（需遍历 `traitFlagsByType` 初始化处再销账）。
- `showOverlayMessage` 第 3/4 参数（(0,3)/(1,2) 等）的布局/按键语义未考证。
- `objectTypeNames` 中型 70/71 均为"玉皇大帝"（1025/1026），哪只是 49 楼假魔王（843 文案"假魔王"）未区分；`bossTypeOrder[5]=71` 属实（2198）。
- `crates/game-core/src/scene.rs` 现存注释引用的是**旧版** a.java 行号（如 7596/899-905），与本文现行行号（7528/7705）不一致；建议统一，不影响行为。
