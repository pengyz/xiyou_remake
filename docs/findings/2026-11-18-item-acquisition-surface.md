# 道具获取面完整枚举（含我的一处枚举错误的更正）

> 2026-11-18。触发：用户指出「相形变位/芭蕉扇/幸运金币/捆仙绳肯定可得，你的记录有误」。
> 本文订正我 2026-11-18 早先给出的「6 个道具无来源」结论，并把「道具如何进入背包」的
> 完整代码面一次性钉死，供求解器与 Stage B 共用。

## 0. 我的错误（原地更正，保留历史）

我在早先的脚本命令统计里用了 `dict(list(counter.items())[:12])`，**只看了每类命令的前 12 个**，
于是把脚本生成的道具误判成「无来源」。更正后：

| 道具 | 脚本来源（运行时脚本表 = `a.java:1124` `levelScriptLines`，68 槽 = `crates/game-core/tests/fixtures/level_scripts.txt`） |
|---|---|
| 20 相形变位 | 槽 18：`DES_1_4_4 CES_20_4_4` |
| 22 下行符 | 槽 25：`CES_22_6_6` |
| 36 / 40 装备 | 槽 53：`CES_36_11_8`、`CES_40_11_9`；槽 55：`CES_36_5_1`、`CES_40_7_1` |

⇒ 用户对「相形变位可得」的记忆**成立**，是我统计截断导致漏判。

## 1. 道具进入背包的完整代码面（A 级，穷尽）

Java 里道具只有一个入口：`addItemToItemStack(int)`（`a.java:7727`，写 `itemStackTypes`/`itemStackUses`）。
其调用点**全仓只有 3 处**：

| 行 | 上下文 | 授予 |
|---|---|---|
| `a.java:7339` | 商店 `src=0 case 2` | 13 |
| `a.java:7342` | 商店 `src=0 case 6` | 19 |
| `a.java:7554` | `pickupItemType(int)` 内部 | 参数 |

`pickupItemType(int)`（`a.java:7534`）的调用点**全仓只有 3 处**：

| 行 | 上下文 | 授予 |
|---|---|---|
| `a.java:5721` | 踩格通用拾取（`var2 = this.entityType[var1]`，实体型即道具号） | 该格的实体型 |
| `a.java:7438` | 商店 `src=1 case 13` `spendGold(4000)` | 18 |
| `a.java:8436` | 脚本 `GIN` 命令（`case 0: pickupItemType(parseScriptInt(...))`） | 脚本参数 |

商店 switch 全量（`applyMerchantOffer`，`a.java:7333` 起）：
- `src=0`（ty77）：`case 1/15/21` 送 1000 金；`case 2` → 13；`case 6` → 19。无其它 case。
- `src=1`（ty78）：`case 1..11` 钥匙（3 蓝50 / 4 黄×5 50 / 5 黄1000 / 6 红800 / 7 蓝200 / 8 黄→金100 / 9 黄+蓝1000 / 10 黄×3 200 / 11 蓝×3 2000）、`case 12` hp+2000 1000、`case 13` → 18 4000。

初始背包：新游戏 `itemStackSize = 0`（`a.java:7222`），`itemUseCounts` 是常量表（`a.java:944`）。
宝箱：`tryOpenChest` 只收集同 param 的 ty4（`a.java:7292-7316`），开箱动画 tick24 走
`removeGroupAtCellByType(x,y,4)`（`a.java:6434`，纯移除+压紧槽位）⇒ **宝箱不产道具**。
敌人死亡：只有 `gainGold`（`a.java:7945`，item25 时双倍），无掉落表。

## 2. 本 JAR 里确实没有来源的 4 个

`16 芭蕉扇`、`17 金勺子`、`24 捆仙绳`、`25 幸运币`：
- 54 层精灵普查无这些实体型（实体型直方图见 `docs/findings/2026-11-18-entity-census.md`）；
- 商店 switch 无对应 case；脚本表无 `GIN_*_{16,24,25}`、无 `CES_{16,24,25}_*`、无 `ADD_*_{16,24,25}_*`。

## 3. 但这个 JAR 是「被裁过」的 build（A 级证据）

1. `menuLabels`（`a.java:459`）定义了 18 条菜单文案，含
   `商店 / 十全大补包 / 攻防神油 / 开门天天乐 / 天庭消费券 / 印度神血油 / 跳过教程`，
   **全仓再无任何引用**（`grep -n "十全大补" a.java` 只命中这一行）⇒ 一套「付费/礼包购买」界面
   被移除，只剩文案数组。
2. JAR 内带一个**未被代码引用**的 `script` 资源（3861B）：`GAME:` 头
   `STLAYER=51 EDLAYER=51 HP=300 ATK=10 DEF=10 RKEY=0 BKEY=0 YKEY=0` + `GUTS:` 64 槽；
   与编译期字面量表逐槽比对仅 5 槽不同（槽 0/18/27/31/63）。
   读资源的两条动态路径只有 `"maplv"+n`（`a.java:9416`）、`"sprite"+n`（`a.java:9497`），
   `resourceNames`（`a.java:450`）只列 16 个图像名、不含 `script` ⇒ 该资源在本 build 是孤儿。
3. `MANIFEST.MF`：`MIDlet-Version: 1.0`、`Media-Price: 单次下载8元`、`MIDlet-Vendor: SNOWFISH`。

⇒ 结论：**「本 build 里拿不到」≠「游戏里不存在」**。台词（`a.java:1929-1930`：「……搞定他，
捆绑他，拿他的芭蕉扇……有芭蕉扇可以灭火了」）证明设计意图里 16 是由剧情给的，而这份 build
缺了发放的那段代码/菜单。

## 4. 对求解器的口径

- 模型层只认**本 build 的代码面**：16/17/24/25 无获取路径 ⇒ 求解器不得凭空给它们建获取边；
  但它们**作为机制仍然存在**（`ITEM_EFFECTS` 有 16/17 的行动语义），一旦基准数据变化就要能吃到。
- 不得由此推出「游戏不可通关」：f13 的三昧真火只是一列（x=6, y=4..6，道具 36 在 (6,3) 上方），
  从到场梯 (1,11) 到出发梯 (11,11) 的通路被 **黄门 (6,10)** 卡住，与 16 无关
  （`./target/release/solver solve-local --floor 13 --entry 1,11` ⇒ `reachable:false`，0 钥匙起点）。
