# boss 残项夹具设计书（task-13：68/84 处置，task-6 执行蓝图·续）

> 角色：spec-writer 设计产物（**纯设计，不含代码**）。补 [fixture-plan.md](fixture-plan.md)
> （task-9 设计书）的 boss 残项缺口——其 §0 十一场景只覆盖 67/74，68/84 遗留"待决策"。
> 证据基座：[evidence-boss.md](evidence-boss.md)（task-3，A 级行号+逐字）、
> [type-closure.md](type-closure.md) §1.4/§1.5 两行残项、层数据 sprite 字节级解析
> （fixture-plan §1.5 同口径，本文件 §1.3 补 83 参数位）。
> 体例基准：`crates/game-core/tests/fixtures/preset-floor40-mov.{preset,txt,frames.txt}`
> 与其消费端 `crates/game-core/tests/mov_route_scene.rs` 头注释；逐场景格式 = 场景 id/
> 预置/输入草图/断言点/销账行（fixture-plan §2 同构）。取证日期：2026-10-16。
>
> 【探针锁定】设计前已探明夹具格式能力边界（§1.4 能力探针 + §3 终裁判定）：
> MOT_L0 记录含 56 层实体 blob（第 21 段），Rust 解析与应用链双端在位
> （crates/game-core/src/save.rs 布局表行 21 + parse_floor_save；scene.rs:613 装载 + :905/:1343 恢复）
> ⇒ **跨层实体预置在格式能力内**，本设计书全部 10 场景均可夹具化，无终裁替代项。

---

## 0. 场景总览（10 个场景，按优先级排序）

| # | 场景 id | 覆盖目标 | 目标层 | 夹具 stem | 销账（证据文档行） |
|---|---|---|---|---|---|
| 1 | FX-BOSS-68A | 68 bump 战 + 击杀钩子 spawnBossEvent(2) 红钥匙×4 十字 | 25 | `preset-f25-boss68` | boss §0 **68 行** + §3.1 + R-boss-1/2/3；type-closure §1.4 **68 行** |
| 2 | FX-BOSS-68B | 68 战斗预测拒绝负例（atk ≤ 310 打不动） | 25 | `preset-f25-boss68-refuse` | boss §3.1（"你无法战胜它"臂，a.java:5487-5490） |
| 3 | FX-CHAN-84F2A | scene26 营救：f2 补刷钩子 + 同格触发 + GIN_1_1000 | 2 | `preset-f2-chan84` | boss §0 **84 行** + §4.1 scene26 行 + 出生源①；type-closure §1.5 **84 行** |
| 4 | FX-CHAN-84F2B | scene26 门控负例：flags[26] 已置 ⇒ 钩子不刷 + 触发不跑 | 2 | `preset-f2-chan84-off` | boss §4.1（flags[26] 双重角色）+ §4.1 scene26 门控 |
| 5 | FX-LAY-2A | script0 出层链最小窗：CES_84/70 → **LAY_2 出层** → RES_0 → GUT_1 交接 | 3 | `preset-f3-lay2` | boss §4.1 scene0 行（LAY_2）+ §1.5 LAY 语义；type-closure §2 **LAY 行** |
| 6 | FX-CHAN-84F3A | scene27 主窗：flags[0] 门控 + GIN_0_13（火眼金睛入包） | 3 | `preset-f3-chan84` | boss §4.1 scene27 行（门控半 + GIN） |
| 7 | FX-CHAN-84F3B | scene27 变体：blob 预置 84@(6,11) ⇒ DES_84 真删臂 | 3 | `preset-f3-chan84-blob` | boss §4.1 scene27 行（DES 半）+ §4.1 出生源②（CES 派生） |
| 8 | FX-CHAN-84F16A | scene28：native 84 同格触发 + GIN_0_19（月桂露入包） | 16 | `preset-f16-chan84` | boss §4.1 scene28 行 |
| 9 | FX-CHAN-84F36A | scene29：native 84 同格触发 + GIN_1_1000 | 36 | `preset-f36-chan84` | boss §4.1 scene29 行 |
| 10 | FX-CHAN-84F26A | scene30/31：from_walk 双门控（flags[0]、持 16 芭蕉扇）+ MOV_84 走位 | 26 | `preset-f26-chan84` | boss §4.1 scene30/31 行 + 门控 a.java:8190-8193 |

第二波（§3.2 扩展候选，不在本批）：FX-BOSS-68C（>50 层无钩子负例，同 FX-BOSS-74B 族）、
FX-CHAN-84F1A（scene19 到达臂）、FX-E2E-84F2-RELOAD（场景 26 后重入 f2 不再补刷的跨夹具链）。

---

## 1. 体例与公共约束

### 1.1 体例引用（不重复展开）

夹具三件套、输入码表、断言点两层约定、硬约束——**全部沿用 fixture-plan.md §1.1-§1.4**：
`.preset`（`MOT_IF@1=` / `MOT_L0@1=` / `SKY_WAR@1=` hex 行）+ `.txt`（`# ticks:`/`# preset:`/
`# cover:` 头 + 输入行）+ `.frames.txt`（Java A 变体 trace 帧 sha）。方向键 **-1=上 / -2=下 /
-3=左 / -4=右**（fixture-plan §1.2 A 级实证）；对话推进 **48**；道具键序 `-6 → -5 → -6`。
断言点写键名，task-6 生成时补 FLD 号（fixture-plan §1.3 约定）。

### 1.2 68 战斗节拍数学表（A 级公式，预置即定预算）

战臂（a.java:5475-5491）：`predictHpLossVsType(68, true)` 过门 → `battleEnemyHp =
enemyBaseHp[27] = 4500`（a.java:5484，**不吃难度缩放**）。每拍伤害（a.java:10033-10036，
`(f_int_151 & 3) == 0` 每 4 tick 一击）：

```
dmg = effectiveAttackVsType(68) - enemyDefScaled[27]   # difficulty=0 ⇒ DefScaled = 310
反击 = enemyAtkScaled[27] - playerDef = 560 - def      # 仅当该击后 battleEnemyHp > 0
```

**68 无 trait**（traitFlagsByType 仅 49/53/74=1、69=2，a.java:6266-6267）⇒ 太公杖/捆仙绳不生效
（B 级对照：67/68/74 中仅 74 有克制）。击杀拍（a.java:10041-10052）：伤害弹窗 + 金弹窗 +
`gainGold(1000)`（持 25 幸运币 ×2，a.java:10046-10051）→ 钩子（R-boss-2）→
`battleSceneRerun=false`（case 68 落 default）→ 收尾 switch 亦 default（a.java:10135-10137）。

预置推荐（钉死预算）：**atk=1000 / def=1000 / hp=1000** ⇒ dmg=690、反击=0、命中数
= ceil(4500/690) = **7 击**（4 tick 节拍 ≈ T28 内击杀）+ 收尾动画 6 tick + 复步。
负例 68B：**atk=310** ⇒ dmg=0 ⇒ `predictBattleHpLoss` 返 -1（a.java:10007 `if (var0 > var4)` 不成立）
⇒ 拒战。

### 1.3 层数据事实表（A 级，sprite 字节解析；83 参数位=EXTRA 第 1 字节）

复现命令 = fixture-plan §1.5 逐字（`EXTRA` 表 83:1 参数位）。本批涉及层（格坐标 = 像素>>5）：

| 层 | 关键实体 | 用途 |
|---|---|---|
| 25 | **68@(6,6)**；红门 2×6（(6,10)/(1,6-8)/(11,6-8)）；楼梯 7@(1,11)/8@(11,11)；丹/钥匙带 | FX-BOSS-68A/B（玩家 (6,7) 南邻，格空） |
| 2 | **83@(11,4) param=26**（scene26 触发器，与补刷 84 同格）；83@(3,7) param1、(1,8) param2、(11,10) param33；72@(3,7)；78@(11,7)；楼梯 7@(1,11)/8@(1,1) | FX-CHAN-84F2A/B（玩家 (11,5)） |
| 3 | **83@(2,11) param=0**（scene0 出层链触发）；**83@(5,11) param=27**（scene27）；83 无 84 native；77@(11,4)；楼梯 7@(11,11)/8@(1,11) | FX-LAY-2A（玩家 (2,10)）；FX-CHAN-84F3A/B（玩家 (5,10)） |
| 16 | **84@(1,11) + 83@(1,11) param=28 同格**；黄门 1×8 环带；敌 49-55 群 | FX-CHAN-84F16A（玩家 (1,10)，格空） |
| 36 | **84@(9,8) + 83@(9,8) param=29 同格**；敌 53/55-59 群；墙 11@(9,7) | FX-CHAN-84F36A（玩家 (9,9) 或 (8,8)，task-6 按可走性定） |
| 26 | **84@(6,3)**；**83@(6,11) param=30**；**83@(6,7) param=31（与红门 2@(6,7) 同格）**；三昧真火 10×33 场（含 (6,6)）；红门 2 纵列 x=6 y=8/9/10 | FX-CHAN-84F26A（玩家 (6,12)；需 redKey=1 开 (6,7) 门 + 道具栈含 16） |

**同格触发语义**（84 与 83 同格时的处理序，A 级）：`interactWithCell` 槽表**倒序**遍历
（a.java:5374 `while (--var8 >= 0)`）——后生成者先处理。f2 补刷场景：83(关卡数据) 先入槽、
84(钩子) 后入槽 ⇒ bump 时 84 先落 case 32 挡路（`var6=false`，a.java:5527-5528），
83 后落 case 83 触发（a.java:5465-5471）⇒ **一次 bump 同时完成"挡路+触发"**；
f16/f36 的 84 与 83 同为关卡数据同帧入槽（sprite 内 84 在前、83 在后 ⇒ 倒序 83 先触发）。

### 1.4 能力探针：MOT_L0 第 21 段（56 层实体 blob）= 跨层预置的格式能力证明

- 布局（A 级）：save.rs 字段序表行 21 `56×(short len + byte[len]) 楼层触发状态 f_byte_arr2_05`；
  Java 对偶 = saveFloorState 写（a.java:9214-9236）/ loadFloorSave 读（a.java:9304-9311）。
- 解析（A 级）：`parse_floor_save`（save.rs:109 起）逐块解析进 `FloorSave.floor_states`。
- 应用（A 级）：`load_floor_from_save` 装载 `scene.saved_floor_states = save.floor_states`
  （scene.rs:613）；进层时 `if let Some(data) = saved_floor_states[floor]` 走恢复臂
  （scene.rs:905/1343），**与 sprite 原生解析互斥**（Java 同构 if/else，a.java:9480/9626）。
- blob 字节配方（A 级，恢复侧解码 a.java:9626-9640 + restoreFloorState a.java:9679-9681）：

```
[count:u16 BE] + count × [type:u8][x_pixel:u16 BE][y_pixel:u16 BE][visible:u8][param:u16 BE]
（x_pixel = 格<<5；恢复侧 spawnEntity(type, x, y, param) + entityVisible=visible + entityRemoved=false）
```

**结论**：预置某层实体（如 scene27 需要的 84@(6,11)）= 在 MOT_L0 第 21 段写该层 blob；
因恢复臂与原生解析互斥，blob 须枚举**该层全实体**（sprite 原生清单 + 追加），配方见 §4.2。
⇒ 本设计书无需终裁替代路径（§3.1）。

---

## 2. 场景逐件设计

### FX-BOSS-68A：太上老君 bump 战 + 击杀钩子 spawnBossEvent(2)（优先级 1）

- **目标**：R-boss-1 战臂（预测过门→开战）、R-boss-2 钩子（bossTypeOrder[2]=68 ⇒
  spawnBossEvent(2)）、R-boss-3（DES 态战利品）全链；战后 case 68 双 switch 均 default
  （无脚本、无重入）。
- **目标层/预置**：floor25，玩家 (6,7)（68@(6,6) 南邻，格空），**atk=1000 def=1000 hp=1000**
  （§1.2 预算），keys/gold 0。
- **输入脚本草图**：
  ```
  T010 -1 press / T012 -1 release     # 北 bump (6,6) → 预测过门 → 开战（walkPhase=5）
  （无输入 ~40 tick：战斗自动进行，7 击 ×4 tick 节拍 + 击杀 + 收尾动画 6 tick）
  T080 -2 press / T082 -2 release     # 走入 (6,7) 拾红钥匙（战后复步若已进 (6,6) 则改走 (6,5)）
  T120 -3 press / T122 -3 release     # 走向 (5,6) 拾第二把（抽查十字另两格 (7,6)(6,7) 在快照断言）
  ```
- **断言点**：
  - 状态（开战拍）：`battleActiveFlag` 0→1、`battleTargetEntity`=68 实体槽、
    `battleEnemyHp`=4500（**断言基数 = enemyBaseHp 而非 scaled**，钉 a.java:5484）；
  - 状态（每击拍）：`battleEnemyHp` 逐击 -690 序列 {4500→3810→3120→2430→1740→1050→360→-330}，
    伤害弹窗值 690、节拍严格每 4 tick 一击（f_int_151 cadence）；`playerHp` 恒 1000（反击 0）；
  - 状态（击杀拍）：**4 实体快照** type=27 出现于 (6,5)(5,6)(7,6)(6,7)，`entityState`=2
    （markEntityDesRemoved，a.java:6332-6347）+ 精灵 27×29/animRow 8；金弹窗 + `goldAmount`
    0→1000（a.java:10051，无 25 幸运币分支）；
  - 状态（收尾）：68 实体 `entityState` 0→1（battleAnimTick==0，a.java:10102-10104）；
    `battleTargetEntity`=-1；**`currentScriptIndex` 不变、`gameMode` 不变**（case 68 双
    switch default——与 74 的 loadLevelScript(21) 对照）；`battleSceneRerun` 恒 false；
  - 状态（拾取拍）：`redKeyCount` 0→1→2（cat2 拾取臂，a.java:5719-5722）；
  - 帧窗：T013-T020 开战跳变（星光/伤害数字帧）、击杀拍单帧 4 钥匙同现 + 金弹窗、
    T083+ 拾取后钥匙消失帧。
- **目标夹具文件名**：`crates/game-core/tests/fixtures/preset-f25-boss68.{preset,txt,frames.txt}`。
- **销账**：evidence-boss.md §0 表 **68 行**（战臂+钩子+无出层链三栏）、§3.1 全节、
  R-boss-1/2/3 关联段；type-closure.md §1.4 表 **68 行** ⬜→✅（"boss 战+钩子——delta"销讫）。

### FX-BOSS-68B：战斗预测拒绝负例（优先级 2）

- **目标**：a.java:5487-5490 拒战臂——`predictBattleHpLoss` 返 -1（攻 ≤ 防）⇒
  "你无法战胜它"、`walkStepCount`=0、不进战斗、实体不动。
- **目标层/预置**：floor25 同 68A，**atk=310**（= DefScaled，`var0 > var4` 不成立 ⇒ -1）、
  def=1000 hp=1000。
- **输入脚本草图**：T010/T060/T110 三次北 bump（重复撞，断言持续拒绝）。
- **断言点**：
  - 状态：`battleActiveFlag` 恒 0、`battleEnemyHp` 无从赋值（恒 0/初值）、
    `walkStepCount`=0（a.java:5488）、overlay 文案 = "你无法战胜它"；
  - 68 实体三字段恒不变（entityState/anim 不动）；玩家格恒 (6,7)；
  - 帧窗：T013-T150 每拍 bump 后的 overlay 文案帧段 ×3，无战斗帧。
- **目标夹具文件名**：`crates/game-core/tests/fixtures/preset-f25-boss68-refuse.{preset,txt,frames.txt}`。
- **销账**：evidence-boss.md §3.1 拒战半（a.java:5487-5490 逐字段）；连带 R-boss-1
  "拒绝路径"一句（预测臂负例）。

> 【2026-10-29】已落地：`crates/game-oracle/tests/fixtures/fx-boss68b.*`，用例 `fx_boss68b_refuse_frames_match`（BUG-046）。落地脚本与上方草图的差异：
> 第二次撞击前用 -5 关框（kind 0 的 -1/-2 是翻页键，不关框），共两次北撞（T300 / T420）。
> 落地路径取代上方"目标夹具文件名"（`crates/game-core/tests/fixtures/preset-f25-boss68-refuse.*`）：帧夹具统一放 `crates/game-oracle/tests/fixtures/`，按 `fx-*` 命名。

### FX-CHAN-84F2A：scene26 营救链（优先级 1，84 主窗）

- **目标**：三合一——f2 补刷钩子（a.java:9642-9647）、同格 bump 触发（case 83 + case 32
  同拍）、scene26 链（TAK_129_132 + DES_84_11_4 + GIN_1_1000 + flags[26] 收尾）。
- **目标层/预置**：floor2，玩家 (11,5)，flags 全 0（默认），gold=0，其余默认。
- **输入脚本草图**：
  ```
  （T001-T005 载入拍：断言补刷钩子已生 84@(11,4)）
  T010 -1 press / T012 -1 release     # 北 bump (11,4)：84 挡路 + 83 触发（同拍双臂）
  （T020+ 对话 4 页：129/130/131/132，48 键逐页推进）
  T020 48 press / T022 48 release
  T050 48 press / T052 48 release
  T080 48 press / T082 48 release
  T110 48 press / T112 48 release     # 末页 → DES_84 + GIN + 脚本收尾 flags[26]=1
  ```
- **断言点**：
  - 状态（载入拍）：84 实体存在于 (11,4)（补刷钩子 `!flags[26] && findBlockingEntity(11,4,84)<0`
    ⇒ spawnEntity(84,352,128,0)）——**载入钩子本身的可执行断言**；
  - 状态（bump 拍）：`gameMode`=11、`currentScriptIndex`=26、`dialogPhase` 推进；
    玩家格恒 (11,5)（84 挡路 ⇒ var6=false）；
  - 状态（收尾）：`goldAmount` 0→1000（GIN_1，a.java:8424-8434 case 1）；84@(11,4)
    `entityState` 0→1（DES_84_11_4，a.java:8314-8336）；`scriptLineFlags[26]` 0→1
    （收尾置位 a.java:8598-8601）；`gameMode` 回 3；
  - 帧窗：T001-T005 补刷 84 登场帧、T013-T110 对话 overlay 帧 4 段、T113+ 84 消失 + 金数跳变帧。
- **目标夹具文件名**：`crates/game-core/tests/fixtures/preset-f2-chan84.{preset,txt,frames.txt}`。
- **销账**：evidence-boss.md §0 表 **84 行**（出生源①+scene26 链+门控）、§4.1 scene26 行；
  type-closure.md §1.5 表 **84 行** ⬜→（首窗）；GIN 臂同步补 type-closure §2 **GIN 行**语料
  （scenes 26-29 为其主语料）。

### FX-CHAN-84F2B：scene26 门控负例（优先级 2）

- **目标**：flags[26] 双重角色负半——已置 ⇒ 补刷钩子不生 84、case 26 门控
  `findBlockingEntity(11,4,84)<0 ⇒ var3=false` ⇒ 触发不跑（无对话、无 GIN）。
- **目标层/预置**：floor2 同 84F2A，**scriptLineFlags[26]=1**（MOT_L0 旗标段第 26 字节=0x01）。
- **输入脚本草图**：T001-T005 载入断言 + T010/T060 两次北 bump。
- **断言点**：
  - 状态（载入拍）：(11,4) **无 84**（钩子跳过）；83@(11,4) 仍在；
  - 状态（bump 拍）：`gameMode` 恒 3、`currentScriptIndex` 不变（无 loadLevelScript）、
    `goldAmount` 恒 0、`dialogPhase` 恒 0；
  - 帧窗：无对话帧段（对照 84F2A 的 4 段）。
- **目标夹具文件名**：`crates/game-core/tests/fixtures/preset-f2-chan84-off.{preset,txt,frames.txt}`。
- **销账**：evidence-boss.md §4.1 flags[26] 双重角色负半；§4.1 scene26 门控行。

### FX-LAY-2A：script0 出层链最小窗（优先级 1，LAY 出层主窗）

- **目标**：scene0 全链的**最小断言窗**——CES_84/70 生成 → 对话 8-21（14 页）+ 玩家 MOV 自走
  → DES_70 → **LAY_2 出层**（3→2 层转场）→ ROS_1_4_7 落点 → **RES_0 数值重置**（difficulty=0 分支）
  → **GUT_1 交接**（flags[0]=1 + currentScriptIndex=1）。窗止于 script1 首页 TAK_22——
  script1/2 的 MOV 链已有 `preset-floor2-mov` 覆盖（fixture-plan §2 mov 行），不重复。
- **目标层/预置**：floor3，玩家 (2,10)（触发器 (2,11) 北邻），difficulty=0（RES_0 分支），
  atk=999/def=999/hp=999/gold=777/keys 各 1（**预置非默认值以钉 RES_0 重置断言**）。
- **输入脚本草图**：
  ```
  T010 -2 press / T012 -2 release     # 南步入 (2,11) → case 83(param0) 到达触发（case0 无 var2 约束）
  （T020-T200：CES_84_6_11 + MOV_0_5_11 自走 + TAK_8_9 2 页）
  T020 48 press / T022 48 release     # 逐页推进（每 TAK 页一拍 48；MOV 拍自走不占输入）
  ...（对话 8-21 共 14 页按页展开，每页一拍；节拍由 task-6 按实测定）
  （T210-T230：DES_70_5_8 → LAY_2 → 转场 wipe ~10 tick → currentFloor 3→2）
  T240 48 press / T242 48 release     # script1 TAK_22 首页推进（交接完成断言）
  ```
- **断言点**：
  - 状态（触发拍）：`currentScriptIndex`=0、84 出现 (6,11)（CES，a.java:8395-8396 标 DES 态）、
    70 出现 (5,8)；`scriptLineFlags[19]` 被置 1（case 0 附带臂，a.java:8076-8080——**旁证断言**）；
  - 状态（LAY 拍）：`pendingFloor`=2、`floorTransitioning`=true、`dialogPhase`=5
    （a.java:8355-8361）；wipe 期间黑幕帧段；转场完成拍 `currentFloor` 3→2
    （a.java:3187-3188）+ f2 数据载入（83@(11,4) 等出现）；
  - 状态（ROS/RES 拍）：玩家落 (4,7)（ROS_1_4_7）；**RES_0 全字段**：`playerHp`→400、
    `playerAtk`→10、`playerDef`→10、`goldAmount`→4、三钥匙→0、`itemStackSize`→0、
    `minFloorReached`→1、`maxFloorReached`→3、`alchemyUpgradeCount`→0（a.java:8470-8484）；
  - 状态（GUT 交接拍）：`scriptLineFlags[0]`=1（GUT 置位，a.java:8310-8313）、
    `currentScriptIndex`=1、对话 22 页帧出现；
  - 帧窗：触发后 CES 双实体登场帧、14 页对话帧段、wipe 转场帧段、RES 后 HUD 数字跳变帧
    （999→400/999→10/777→4）、script1 首页帧。
- **目标夹具文件名**：`crates/game-core/tests/fixtures/preset-f3-lay2.{preset,txt,frames.txt}`。
- **销账**：evidence-boss.md §4.1 **scene0 行**（LAY_2 出层）+ §1.5 LAY 语义；type-closure.md
  §2 **LAY 行** ⬜→✅（语料 script0 唯一 LAY，A 级：68 条脚本 `LAY_` 仅 1 处）+ **RES 行**
  （同链语料）+ **GUT 行**补语料。**注意**：本窗不含 scene27 的 DES_84@6,11（84 由本链
  CES 生成但链尾已 LAY 出层离 f3）——该臂归 FX-CHAN-84F3B（§2）。

### FX-CHAN-84F3A：scene27 主窗（flags[0] 门控 + GIN_0_13）（优先级 2）

- **目标**：case 27 门控（`var3 &= var2 & scriptLineFlags[0]`，a.java:8187-8189，**到达触发**）
  + 链 TAK_113_117 + TAK_133_135（9 页）+ GIN_0_13（火眼金睛入包）。无 84 时 DES_84_6_11
  为安全 no-op（a.java:8333-8335 `if findBlockingEntity>=0 才删`）——负断言一并钉。
- **目标层/预置**：floor3，玩家 (5,10)，**scriptLineFlags[0]=1**（其余 0），道具栈空。
- **输入脚本草图**：
  ```
  T010 -2 press / T012 -2 release     # 南步入 (5,11) → 到达触发（bump 不触发：case27 要 var2）
  （T020-T200：对话 9 页（113-117、133-135）逐页 48 推进）
  T020 48 press / ... / T200 48 press # 末页 → DES_84_6_11(no-op) + GIN_0_13
  ```
- **断言点**：
  - 负半门控（对照窗，同夹具前置段可选）：T005 若先**北 bump** (5,11) ⇒ **不触发**
    （case27 需 var2=true）——`currentScriptIndex` 不变；随后南步入才触发；
  - 状态（收尾）：`itemStackTypes` 增 13（GIN_0，a.java:8424-8431 case 0）、
    `itemStackSize` 0→1；`scriptLineFlags[27]`=1；**实体计数不变**（DES no-op 负断言）；
  - 帧窗：9 页对话帧段、收尾道具栏 HUD 数字跳变帧。
- **目标夹具文件名**：`crates/game-core/tests/fixtures/preset-f3-chan84.{preset,txt,frames.txt}`。
- **销账**：evidence-boss.md §4.1 **scene27 行**门控半 + GIN 半。

### FX-CHAN-84F3B：scene27 DES_84 真删臂（blob 预置变体）（优先级 2）

- **目标**：scene27 的 DES_84_6_11 **真删**臂（84 在位）——补 84F3A 的 no-op 负断言对照。
- **目标层/预置**：floor3 同 84F3A，**另加 MOT_L0 第 21 段 floor_states[3] blob** =
  sprite3 全 38 实体序列化 + 追加 `84@(6,11)（visible=1, param=0）`（§4.2 配方）。
  blob 与原生解析互斥（§1.4）⇒ 84F3A 的 native 断言基线须以 blob 版全清单为准重算。
- **输入脚本草图**：同 84F3A。
- **断言点**：
  - 状态（载入拍）：84@(6,11) 在位（blob 恢复臂，scene.rs:905/1343）；
  - 状态（收尾）：84@(6,11) `entityState` 0→1（DES 真删，a.java:8333-8335）；
    `itemStackTypes` 增 13；`scriptLineFlags[27]`=1；
  - 帧窗：载入拍 84 登场帧 + 收尾 84 消失帧（与 84F3A 的"全程无 84"帧基线区分）。
- **目标夹具文件名**：`crates/game-core/tests/fixtures/preset-f3-chan84-blob.{preset,txt,frames.txt}`。
- **销账**：evidence-boss.md §4.1 scene27 行 DES 半 + 出生源②（CES_84_6_11 派生态的持久性）。

### FX-CHAN-84F16A：scene28（native 84 同格触发 + GIN_0_19）（优先级 3）

- **目标**：直通门控族（case 28 无附加条件，a.java:8092-8122 组）+ native 84 同格触发序
  （§1.3）+ 链 TAK_136_143 + DES_84_1_11 + TAK_144 + GIN_0_19（千年月桂露入包）。
- **目标层/预置**：floor16，玩家 (1,10)（(1,11) 北邻，格空），flags 全 0，道具栈空。
- **输入脚本草图**：
  ```
  T010 -1 press / T012 -1 release     # 北 bump (1,11)：83 先触发（倒序）+ 84 挡路
  （T020-T200：对话 9 页（136-143、144）逐页 48 推进 → DES_84_1_11 + GIN_0_19）
  ```
- **断言点**：
  - 状态（bump 拍）：`gameMode`=11、`currentScriptIndex`=28；玩家格恒 (1,10)；
  - 状态（收尾）：84@(1,11) `entityState` 0→1；`itemStackTypes` 增 19；`scriptLineFlags[28]`=1；
  - 帧窗：同格"触发+挡路"同拍帧（对话 overlay 与玩家未移动同帧）、84 消失帧、道具栏跳变帧。
- **目标夹具文件名**：`crates/game-core/tests/fixtures/preset-f16-chan84.{preset,txt,frames.txt}`。
- **销账**：evidence-boss.md §4.1 **scene28 行**；同格触发序（§1.3）顺带钉进帧窗。

### FX-CHAN-84F36A：scene29（native 84 同格触发 + GIN_1_1000）（优先级 3）

- **目标**：scene29 短链（TAK_145_146 2 页 + DES_84_9_8 + GIN_1_1000）——84 族中最短窗，
  作直通门控族第二样本（覆盖直通族"无 var2 约束"：bump/step 双触发源都可，主窗用 bump）。
- **目标层/预置**：floor36，玩家 (9,9)（(9,8) 南邻；若 (9,9) 被 53/55-59 占则改 (8,8)，task-6 按
  sprite36 可走性定——敌人阻挡格需先清或预置绕位），flags 全 0，gold=0。
- **输入脚本草图**：T010 北 bump (9,8) → 对话 2 页（145/146）→ 收尾。
- **断言点**：
  - 状态（收尾）：84@(9,8) `entityState` 0→1；`goldAmount` 0→1000；`scriptLineFlags[29]`=1；
  - 帧窗：2 页对话帧段 + 金跳变帧。
- **目标夹具文件名**：`crates/game-core/tests/fixtures/preset-f36-chan84.{preset,txt,frames.txt}`。
- **销账**：evidence-boss.md §4.1 **scene29 行**。

### FX-CHAN-84F26A：scene30/31 双门控 + MOV_84 走位（优先级 4，窗最长）

- **目标**：case 30（`var3 &= var2` 到达族，a.java:8128-8146 组）与 case 31
  （`var3 & var2 & findItemStackIndex(16)>=0`，a.java:8190-8193，**持芭蕉扇**）双门控
  + scene30 的 MOV_84 三段实体路径（m_106 五参臂）+ scene31 的 DES_10/DES_84 双删。
- **目标层/预置**：floor26，玩家 (6,12)，**scriptLineFlags[0]=1**（scene31 门控要 flags[0]？——
  复核：case 31 = `(var5 = var3 & var2) & findItemStackIndex(16) >= 0`，**无 flags[0] 要求**
  （flags[0] 是 case 27 的）；故预置只需 **道具栈 [16×1]**、**redKey=1**（开 (6,7) 红门）。
- **输入脚本草图**：
  ```
  T010 -2 press / T012 -2 release     # 南步入 (6,11) → case 30 到达触发 → scene30：
                                      #   MOV_84_6_3_4_3 → MOV_84_4_3_8_3 → MOV_84_8_3_6_3
                                      #   （84 走位三段）+ TAK_107 1 页
  T200 48 press                       # 推完 TAK_107
  T260 -1 press / T262 -1 release     # 北 bump (6,7) 红门 → 开门（消耗 redKey）
  T300 -1 press / T302 -1 release     # 北步入 (6,7) → case 31 到达触发 → scene31：
                                      #   TAK_108 + DES_10_6_6 + MOV_0_6_5 + TAK_109_112
                                      #   + TAK_147_148 + DES_84_6_3 + TAK_149
  （T310-T600：对话 6 段逐页 48 推进 → 双删收尾）
  ```
- **断言点**：
  - 状态（scene30）：`currentScriptIndex`=30；84@(6,3) 像素位移三段
    (6,3)→(4,3)→(8,3)→(6,3)（m_106 路径缓动，逐段快照）；`scriptLineFlags[30]`=1；
  - 状态（开门拍）：`redKeyCount` 1→0；(6,7) 门 `entityState` 0→1；
  - 状态（scene31 收尾）：10@(6,6) `entityState` 0→1（DES_10_6_6）；84@(6,3) 0→1
    （DES_84_6_3）；玩家落 (6,5)（MOV_0_6_5）；`scriptLineFlags[31]`=1；
  - 负半（可选拍）：预置无 16 时同输入 ⇒ scene31 不触发、门照开（门控隔离断言）；
  - 帧窗：84 走位帧段（m_106 缓动）、开门帧、双删单帧、长对话 6 段。
- **目标夹具文件名**：`crates/game-core/tests/fixtures/preset-f26-chan84.{preset,txt,frames.txt}`。
- **销账**：evidence-boss.md §4.1 **scene30/31 行** + case 31 门控行（a.java:8190-8193）。

---

## 3. 能力探针结论与终裁判定

### 3.1 全链可夹具化判定（终裁替代方案：**无**）

任务第 3 项的触发条件（"跨多层预置超夹具格式能力"）经探针**不成立**：

| 待验环节 | 探针结论（A 级） | 影响场景 |
|---|---|---|
| 跨层实体预置 | MOT_L0 第 21 段 56 blob：save.rs 布局表 + parse_floor_save 全解析 + scene.rs:613/:905 装载恢复链在位 | 84F3B（唯一受益者） |
| blob 与原生互斥 | Java if/else（a.java:9480/9626）+ Rust 同构 ⇒ blob 须全清单（§4.2 配方可构造） | 84F3B |
| 长链（14 页对话 + 转场 + 交接） | 输入格式无长度上限（序章 T1-9800 先例） | FX-LAY-2A |
| 数值重置断言 | RES_0 全字段可由 MOT_L0 预置非默认值钉死 | FX-LAY-2A |
| 战斗预算可钉 | §1.2 数学表：atk/def 预置直定击数与节拍 | 68A/B |

**故本批 10 场景全部走夹具三件套体例，无"单测替代"项**（fixture-plan §1.4 的 MVS/未知 op
硬约束不波及本批——scenes 0/26-31 全部在 68 条真脚本语料内，可由 preset+输入驱进）。

### 3.2 何时才需要终裁替代（预案，本批不触发）

- 若 task-6 实现期发现 `parse_preset`/装载链**不透传**第 21 段（探针已证透传，此处仅预案）：
  84F3B 降级为 Rust 合成单测（fixture-plan §1.4 体例），销账行改挂 evidence-boss §4.1
  DES 半 + 注明"夹具格式限制"；
- 若 scene31 门控负半（无 16 不触发）与开门臂耦合导致输入不可分：负半拆出独立小窗
  （preset-f26-chan84-no16）。

### 3.3 扩展候选（不在本批）

| 候选 | 内容 | 备注 |
|---|---|---|
| FX-BOSS-68C | f53（currentFloor>50）击杀 68 无钩子负例 | 同 FX-BOSS-74B 族（a.java:10058 总闸） |
| FX-CHAN-84F1A | scene19 到达臂（f1 83@(6,9) param19；CES_84_7_7 + MOV 走位链） | 与 30 同族 m_106 长链 |
| FX-E2E-84F2-RELOAD | scene26 后重入 f2 断言不再补刷 | 跨夹具状态链，宜作双夹具序贯或单测 |
| FX-BOSS-68-DIGIT | 68 战斗数字条 battleDigitByType 渲染窗 | 渲染侧，随 T-2/T-3 族走 |

---

## 4. 预置字节配方

### 4.1 MOT_L0 字段序（A 级 = a.java:9190-9226 写序；save.rs 布局表同构）

沿用 fixture-plan §1.6 逐字。本批涉及的预置差异点：

| 场景 | 相对基线的字段差异 |
|---|---|
| 68A | currentFloor=0x19(25)、playerCell=(6,7)、hp/atk/def=1000 |
| 68B | 同 68A 但 atk=310（0x0136） |
| 84F2A | currentFloor=0x02、playerCell=(11,5)、flags 全 0 |
| 84F2B | 同 84F2A 但 script_flags[26]=0x01 |
| FX-LAY-2A | currentFloor=0x03、playerCell=(2,10)、hp/atk/def=999、gold=777、三钥匙=1 |
| 84F3A | currentFloor=0x03、playerCell=(5,10)、script_flags[0]=0x01 |
| 84F3B | 同 84F3A + **floor_states[3] blob（§4.2）** |
| 84F16A | currentFloor=0x10、playerCell=(1,10) |
| 84F36A | currentFloor=0x24、playerCell=(9,9) |
| 84F26A | currentFloor=0x1a、playerCell=(6,12)、redKey=1、itemStack=[16×1] |

### 4.2 floor_states[3] blob 构造配方（84F3B 用；§1.4 字节格式）

```python
# 输入 = sprite3 解析清单（§1.3 复现命令产出）+ 追加 84@(6,11)
import struct
def blob(ents):                      # ents = [(type, cell_x, cell_y, param, visible)]
    out = struct.pack('>H', len(ents))
    for t, cx, cy, p, v in ents:
        out += struct.pack('>BHHBH', t, cx << 5, cy << 5, v, p)
    return out
# 例：sprite3 全清单（含 param 位：83@(2,11) p=0、83@(5,11) p=27、7/8 楼梯 param 0x0400/0x0200 等）
#     + [(84, 6, 11, 0, 1)] 追加
# 写入 MOT_L0 第 21 段：第 3 块（index 3）len>0，其余 55 块 len=0（-1/0 均按读侧 len>0 判）
```

**校验点**（task-6 落地时）：blob 块数恒 56；恢复侧 `entityVisible[i]` 按序号回填
（a.java:9638 quirk：按 spawn 序而非实体槽——保真照搬）；生成后跑一轮
`parse_floor_save` 往返 = 状态等价。

---

## 5. 销账对照表（本设计书 → 台账行）

| 台账 | 行/节 | 销账场景 | 转色 |
|---|---|---|---|
| evidence-boss.md §0 | 68 行 | 68A + 68B | "战臂+钩子语义待查" → ✅ 已设计 |
| evidence-boss.md §0 | 84 行 | 84F2A/B + FX-LAY-2A + 84F3A/B + 84F16A/36A/26A | "出层待普查" → ✅ 已设计（出层=scene0 LAY_2 + 26-31 族） |
| evidence-boss.md §3.1 | 全节 | 68A/B | 数值/钩子/无脚本对照断言落窗 |
| evidence-boss.md §4.1 | scene0/26/27/28/29/30/31 行 | 对应场景 | 逐行挂窗 |
| type-closure.md §1.4 | 68 行 ⬜ | 68A/B | → ✅（生成夹具后） |
| type-closure.md §1.5 | 84 行 ⬜ | 84F2A 起首窗 | → 🎨/✅ 按实跑 |
| type-closure.md §2 | LAY 行 ⬜ | FX-LAY-2A | → ✅ |
| type-closure.md §2 | GIN 行 ⬜ | 84F2A/3A/16A/36A | 语料=scripts26-29，主窗销 |
| type-closure.md §2 | RES 行 ⬜ | FX-LAY-2A | → ✅（同链） |

> 判定口径沿用 type-closure §0：✅=已交互/已战斗（帧窗断言落地）；仅设计未落地时保持 ⬜/🎨。
> 本文件为设计书——**销账动作在 task-6 生成夹具并首跑绿之后执行**（含 fixture-plan §0
> 尾注的连带 ARM_GAP 销账口径）。
