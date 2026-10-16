# 夹具场景设计书（task-6 执行蓝图）

> 角色：spec-writer 设计产物（**纯设计，不含代码**）。依据四份证据文档：
> [evidence-interact-types.md](evidence-interact-types.md)（型 2/6/82/11）、
> [evidence-items.md](evidence-items.md)（道具 15/17/18/19/23/24/25）、
> [evidence-boss.md](evidence-boss.md)（boss 67/68/74/84/87）、
> [evidence-todos.md](evidence-todos.md)（T-1 MVS/未知 op、T-2/T-3 渲染）。
> 体例基准：`crates/game-core/tests/fixtures/preset-floor40-mov.{preset,txt,frames.txt}`
> 与其消费端 `crates/game-core/tests/mov_route_scene.rs` 头注释（再生命令在该头注释）。
> 取证日期：2026-10-16。层数据事实由 sprite 字节级解析得出（复现命令见 §1.5）。

---

## 0. 场景总览（11 个场景，按优先级排序）

| # | 场景 id | 覆盖目标 | 目标层 | 夹具 stem | 销账（证据文档行） |
|---|---|---|---|---|---|
| 1 | FX-INT-6A | 型 6 首碰显形三元组 + 幂等 + 地形豁免 | 5 | `preset-f5-reveal` | interact-types **R-INT-6** 全行 + 补遗#2 |
| 2 | FX-DOOR-2A | 型 2 开门（有钥匙）+ popup 时机 | 40 | `preset-f40-door2` | interact-types **R-INT-2** 有钥匙半 + 补遗#1 |
| 3 | FX-DOOR-2B | 型 2 无钥匙阻挡（负例） | 40 | `preset-f40-door2-nokey` | interact-types **R-INT-2** 无钥匙半 |
| 4 | FX-ITEM-17A | 金勺子凿墙成功臂 + 出栈 | 29 | `preset-f29-item17` | items 速查表 **17 行** + §2.2 + §3.1/3.4/3.5 |
| 5 | FX-ITEM-17B | 金勺子失败不消耗（负例） | 1 | `preset-f1-item17-fail` | items §3.1（失败不消耗）+ §2.2 失败分支 |
| 6 | FX-ITEM-18A | 玄明石全清墙 + 出栈 | 37 | `preset-f37-item18` | items 速查表 **18 行** + §2.3 + §3.4 |
| 7 | FX-ITEM-19A | 月桂露回血精确值/取整 + 出栈 | 1 | `preset-f1-item19` | items 速查表 **19 行** + §2.4 + §3.6 |
| 8 | FX-BOSS-67A | 啸天犬击杀钩子 spawnBossEvent(3) → 金勺子 | 15 | `preset-f15-boss67` | boss §0 **67 行** + §2.2 + R-boss-2/3 |
| 9 | FX-BOSS-74A | 巨灵神双钩子 + 遗言重放（无门控） | 20 | `preset-f20-boss74` | boss §0 **74 行** + §3.2（含"待补核对点"） |
| 10 | FX-MVS-A | MVS 型 82 配对互传（**条件场景**） | 合成 | `mvs_pair_scene`（Rust 单测） | interact-types **R-INT-82** MVS 半 + todos **R-TODO-1a** |
| 11 | FX-SCRIPT-UNKNOWN-A | 真未知 op 静默停摆（**条件场景**） | 合成 | `script_unknown_stall`（Rust 单测） | todos **R-TODO-1b** |

扩展候选（§3，不在本批优先级）：FX-ITEM-25A（幸运币金翻倍）、FX-BOSS-74B（f53 >50 层无钩子负例）、
FX-E2E-67-SPOON-WALL（杀 67→拾勺→凿墙整合链）、FX-RENDER-67（T-2 拼装渲染 golden）。

---

## 1. 体例与公共约束

### 1.1 夹具三件套（沿用 preset-floor40-mov 体例）

| 文件 | 内容 | 生成方式 |
|---|---|---|
| `<stem>.preset` | RecordStore dump：`MOT_IF@1=` / `MOT_L0@1=` / `SKY_WAR@1=` hex 行 | 从 `reference/oracle/scenarios/` 既有 preset 拷贝改字段（见各场景"预置"） |
| `<stem>.txt` | `# ticks:` / `# preset:` / `# cover:` 头 + `<tick> <keyCode> press\|release` 输入行 | 本设计书给草图，task-6 定 tick |
| `<stem>.frames.txt` | 头注释（源 trace sha256 + 再生命令指针）+ 逐 tick `NNNN <FRAME-sha>` | Java A 变体 trace 抽取（A==B==C 逐字节一致后） |

`# cover:` 头必须写：场景目标、证据指针（a.java 行号）、逐阶段节拍说明——照抄 preset-floor40-mov.txt 的叙事密度。

### 1.2 输入码表（方向行 A 级实证；其余 B 级）

> 【2026-10-16 更正】方向键位原表**上/下写反**（原 B 级依 facing 编号推断）。A 级实证：
> **-1=上 / -2=下 / -3=左 / -4=右**（同义键码 50/56/52/54 = 键盘 2/8/4/6，keyHeldCode 分派
> `case -1/50 facing 1`=上）。佐证：preset-floor40-mov 真夹具北撞即用 `-1`。原表主张保留如下：
> "（B 级）-2/-1/-4/-3 = 上/下/左/右（a.java keyHeldCode 分派 `case -2/56 facing 0 …`）"——
> facing 编号 ≠ 屏幕方向，系推断失误。本设计书各场景草图的方向用键已按实证互换。

| 码 | 语义 | 依据 |
|---|---|---|
| `-1 / -2 / -3 / -4` | 上/下/左/右（keyHeldCode，走格；50/56/52/54 同义） | A 级实证（floor40-mov 北撞 `-1` + 键码 2/8/4/6 对照） |
| `48` | 对话推进（'0' 键） | preset-floor40-mov cover"对话推进键 48" |
| `-5` | 确定/探索（'5' 键等价） | 游戏文案"确定键/5:探索地图"（a.java:442 附近） |
| `-6` | 左软键：开道具面板 / overlay 确认 | a.java:3937-3938 面板 `case -6: case -5:`、a.java:3300-3303 overlay 确认 `useItemStack` |
| `-7` | 右软键：返回/菜单 | a.java:3296-3298 overlay `case -7 → gameMode=10` |

道具使用的标准键序（FX-ITEM-* 全部适用）：`-6`（开面板 gameMode=10）→ `-3/-4`（面板光标移到目标道具）→
`-5`（选中 → kind-3 说明 overlay，a.java:3941-3965）→ `-6`（overlay 确认 → `useItemStack`，a.java:3300-3303）。
快捷路径（游戏中直接使用 f_int_96 选中项，a.java:3647-3652）作可选变体。

### 1.3 断言点两层约定

1. **帧窗断言**（差分主判据）：Rust 全帧 sha == Java 参照帧 sha（逐 tick，frames.txt 全量）；
   设计书另标**关键帧窗**（如"T120-T140 = 说明 overlay 帧、T141 起 = 出栈后 HUD 帧"）供 task-6 定位失败。
2. **状态检查点**（trace 断言）：在指定 tick 断言状态快照键——如 `redKeyCount`、`itemStackSize`、
   `itemStackUses[]`、`entityState`、`playerHp`、`goldAmount`、`dialogPhase`、`currentScriptIndex`。
   FLD 编号以 oracle dump 实测为准（对照 mov_route_scene.rs 头注释已登记：082 walkPhase /
   243 dialogPhase / 257 f_int_127 / 258 routePhase / 259 f_byte_19 / 263 battleActiveFlag / 342 walkStepCount）；
   本设计书只写键名，**task-6 生成夹具时补登记 FLD 号**（未登记 = 断言不可执行）。

### 1.4 硬约束：合成脚本不可做 Java 帧差分（影响 FX-MVS-A / FX-SCRIPT-UNKNOWN-A）

Java 的 `levelScriptLines`（a.java:1124-1193，68 条）是 `<init>` 硬编码常量、无外部写点
（evidence-todos T-1/DC-2 的字节码复核结论）。**MVS 与未知 op 在 68 条脚本中 0 次出现**
⇒ 无法通过 preset+输入把 Java 侧驱进这两个臂 ⇒ 此二场景**不能**用 frames.txt 差分体例，
只能做 **Rust 合成状态单测**（注入实体/脚本行字符串 → 断言状态迁移）。设计仍给出断言点与销账行，
但体例标注"单测"而非"夹具"。

### 1.5 层数据事实表（A 级，字节级解析 sprite{n}，n = currentFloor）

复现命令（与 evidence-todos 附录 B 同口径；`EXTRA` 表 = 各 case 尾参字节数）：

```bash
python3 - <<'EOF'
import struct
EXTRA = {4:1,5:1,81:1,6:0,12:0,7:2,8:2,9:1,76:2,82:2,77:1,78:1,83:1,57:1,59:1,70:1,71:1,72:1,73:1}
def parse(p):
    d=open(p,'rb').read(); n=struct.unpack_from('<I',d,0)[0]; pos=4; out=[]
    for _ in range(n):
        t=d[pos]; x,y=struct.unpack_from('<HH',d,pos+1); pos+=5; pos+=EXTRA.get(t,0)
        out.append((t,x>>5,y>>5))
    assert pos==len(d); return out
for i in (5,20,29,37,15,20,40,0,1):
    print(i, sorted(set(parse(f'assets/raw/sprite{i}'))))
EOF
```

**编号约定（已实证）**：`sprite{n} ↔ currentFloor = n`。证据：`preset-floor2-mov.preset`
MOT_L0 字节 1 = 0x02（currentFloor=2）且 sprite2 含该夹具 cover 点名的 72@(3,7)、11 墙(2,8)、
83@(3,7)；`preset-floor40-mov.preset` MOT_L0 字节 1 = 0x28（=40）且 sprite40 含 cover 点名的
2 红门 (6,8)、83@(6,7)、73@(6,2)。

| 层 | 关键实体（格坐标 = 像素>>5） | 用途 |
|---|---|---|
| 5 | 6@(9,11) | FX-INT-6A |
| 40 | 2 红门 (6,8)、83@(6,7) | FX-DOOR-2A/2B（可复用 preset-floor40-mov 的 MOT_L0） |
| 29 | 11 墙 (8,3) 单堵 | FX-ITEM-17A |
| 37 | 11 墙 ×8：(2,4)(2,8)(4,6)(6,4)(6,8)(8,6)(10,5)(10,8) | FX-ITEM-18A（全清断言 8 个快照；呼应 a.java:838"37楼仓库"文案，B 级） |
| 1 | 无敌 41×4/42/43×3/44/45/46；无型 11 | FX-ITEM-17B（失败路径）、FX-ITEM-19A |
| 15 | **67@(6,5)**、11 墙 (8,1)、49/50/51/52/53/43 敌群 | FX-BOSS-67A |
| 20 | **74@(6,6)**、2 红门 (6,9)、83@(6,6)、5 门卫 (6,6) | FX-BOSS-74A |
| 0 | 25 幸运币 (6,5) | FX-ITEM-25A（扩展） |
| 19 | 23 太公杖 (6,3) | FX-BOSS-74A 变体（trait ×2） |
| 53 | 74@(7,4)、75@(5,4)、71@(6,3)，且 currentFloor>50 | FX-BOSS-74B（扩展负例） |

### 1.6 预置（MOT_L0 可改字段，A 级布局 = a.java:9190-9226 写序）

`difficulty(byte) · currentFloor(byte) · minFloor · maxFloor · facing · weapon · armor ·
playerCellX(short) · playerCellY(short) · hp(int) · atk(int) · def(int) ·
yellowKey(short) · blueKey(short) · redKey(short) · gold(int) · alchemyCount(short) ·
inventoryEntryCount(short) + 条目对 + scriptLineFlags(bool×N) + itemStackSize(byte) +
itemStackTypes[] (byte×N) + itemStackUses[] (byte×N)`。

道具栈预置直接写 `itemStackTypes/Uses`（如 `[17]` = size=1, types=[17], uses=[1]；
`[25]` = size=1, types=[25], uses=[-1=0xFF]）。**注意 uses 为有符号 byte**（evidence-items §1.4）。

---

## 2. 场景逐件设计

### FX-INT-6A：型 6 首碰显形三元组（优先级 1）

- **目标**：R-INT-6 全语义——首 bump 显形三元组、二次 bump 幂等、var7++ 恒执行、地形豁免放行。
- **目标层/预置**：floor5，玩家 (9,12)（型 6 在 (9,11)，北邻一格），hp/atk/def 默认即可。
- **输入脚本草图**：
  ```
  T010 -1 press / T012 -1 release     # 北迈步 bump (9,11) → 首碰显形
  T060 -1 press / T062 -1 release     # 再 bump (9,11) → 幂等（仅 stair_count++）
  T110 -1 press / T112 -1 release     # 放行走入 (9,11) → 落格 f_bool_07（bob）
  T160 -2 press / T162 -2 release     # 离开（南返）
  ```
- **断言点**：
  - 状态：T020 快照 (9,11) 实体 `{visible: 0→1, solid(state): 0→2, anim_idx: →9}`（写序断言：三字段同拍全变，anim 恒 9）；
  - T070 二次 bump 后三字段**不变**（幂等）；
  - T120 落格后 `player_bob_applied`（Java `f_bool_07`，a.java:5650-5652）= true；
  - 帧窗：T013-T016 显形动画帧（state=2 过渡）、T113+ 玩家 bob 帧。
- **目标夹具文件名**：`crates/game-core/tests/fixtures/preset-f5-reveal.{preset,txt,frames.txt}`。
- **销账**：evidence-interact-types.md **R-INT-6** 整行（语义总结 3 条 + 可判定性）+ 补遗#2
  （scene.rs:4515 拆臂缺口）；连带 coverage-matrix 对应 ARM_GAP（若 task-5 未销）。

### FX-DOOR-2A：型 2 开门（有钥匙）（优先级 2）

- **目标**：R-INT-2 有钥匙臂 + 补遗#1 popup 时机（popup 在 consumeKeyForDoor **内部**、redKeyCount-- 之前）。
- **目标层/预置**：floor40，玩家 (6,9)、redKey=1（**逐字沿用 preset-floor40-mov.preset 的 MOT_L0**，
  仅缩短输入；sprite40 的 (6,8) 红门原生存在）。
- **输入脚本草图**：
  ```
  T010 -1 press / T012 -1 release     # 北 bump (6,8) 红门 → 开门
  T060 -1 press / T062 -1 release     # 走入 (6,8)
  ```
- **断言点**：
  - 状态：T020 `redKeyCount` 1→0；(6,8) 门实体 `entityState` 0→1 + `entitySpriteW/H` 45/56 + `entityAnimRow` 6（markEntityRemoved 换开门动画，a.java:6355-6361）；popup 事件（kind=1 icon=1）记录序：**popup 先于计数减一**（trace 序断言，补遗#1 的对拍点）；
  - `allowed` 放行 ⇒ T060 走入成功（walkPhase=1，玩家格 (6,8)）；
  - 帧窗：T013-T030 开门动画帧（45×56 行 6），T031 起门帧空出。
- **目标夹具文件名**：`crates/game-core/tests/fixtures/preset-f40-door2.{preset,txt,frames.txt}`。
- **销账**：evidence-interact-types.md **R-INT-2** 有钥匙半（语义总结"有钥匙效果"3 步顺序）+ **补遗#1**（popup 时机）。

### FX-DOOR-2B：型 2 无钥匙阻挡（负例）（优先级 2）

- **目标**：无钥匙臂：walkStepCount=0、"你没有红钥匙"、allowed=false、门与钥匙均不变。
- **目标层/预置**：floor40 同上，**redKey=0**（MOT_L0 改 1 字段）。
- **输入脚本草图**：同 FX-DOOR-2A 第 1 拍 bump + 后续两次重复 bump（断言持续阻挡）。
- **断言点**：
  - 状态：T020 `redKeyCount` 恒 0、门 `entityState` 恒 0、`walkStepCount`=0、overlay 文案 id = "你没有红钥匙"；
  - `allowed=false` ⇒ 玩家格恒 (6,9)；
  - 帧窗：T013-T050 overlay 文案帧，无开门动画帧。
- **目标夹具文件名**：`crates/game-core/tests/fixtures/preset-f40-door2-nokey.{preset,txt,frames.txt}`。
- **销账**：evidence-interact-types.md **R-INT-2** 无钥匙半（"无钥匙效果"3 步）。

### FX-ITEM-17A：金勺子凿墙成功臂（优先级 3，道具主笔）

- **目标**：evidence-items §2.2——`m_089(11)` 移除四邻型 11、成功置 gameMode=3、消耗 1 次出栈。
- **目标层/预置**：floor29，玩家 (8,4)（墙 (8,3) 南邻），道具栈 `[17×1]`（types=[17] uses=[1]）。
- **输入脚本草图**：
  ```
  T010 -6 press / T012 -6 release     # 开道具面板（gameMode=10）
  T040 -5 press / T042 -5 release     # 选中 17 → kind-3 说明 overlay（a.java:3955-3965）
  T080 -6 press / T082 -6 release     # overlay 确认 → useItemStack（a.java:3300-3303）
  T140 -1 press / T142 -1 release     # 断言后走入原墙格 (8,3)（墙已删应放行）
  ```
- **断言点**：
  - 状态：T090 (8,3) 墙 `entityState` 0→1（markEntityRemoved）；`itemStackUses[0]` 1→0 且 `itemStackSize` 1→0（出栈压紧，a.java:7768-7775）；`gameMode` 回 3；
  - T150 玩家格 = (8,3)（凿墙放行的端到端证明）；
  - 帧窗：T043-T080 说明 overlay 帧、T083-T090 无墙帧、T150+ 原墙格可走帧。
- **目标夹具文件名**：`crates/game-core/tests/fixtures/preset-f29-item17.{preset,txt,frames.txt}`。
- **销账**：evidence-items 速查表 **17 行**、§2.2 语义总结、§3.1（成功才消耗）、§3.4（17=四邻 vs 18=全层）、
  §3.5（提示语"面对"与四邻实现的 quirk——**断言按四邻实现写，quirk 不修**）。

### FX-ITEM-17B：金勺子失败不消耗（负例）（优先级 3）

- **目标**：失败分支——提示文案、`itemStackUses` 不减、不出栈、gameMode 不变 3。
- **目标层/预置**：floor1（全层无型 11，A 级：sprite1 类型直方无 11），道具栈 `[17×1]`。
- **输入脚本草图**：同 FX-ITEM-17A 键序（面板 → 选中 → 确认），末尾加一次重复使用（二次确认仍不消耗）。
- **断言点**：
  - 状态：T090/T150 `itemStackUses[0]` 恒 1、`itemStackSize` 恒 1；overlay 文案 = "你必须面对一堵墙使用"；
  - 帧窗：overlay 文案帧窗两段（两次使用）。
- **目标夹具文件名**：`crates/game-core/tests/fixtures/preset-f1-item17-fail.{preset,txt,frames.txt}`。
- **销账**：evidence-items **§3.1**（失败不消耗）+ §2.2 失败分支（7810-7811 臂）。

### FX-ITEM-18A：玄明石全清墙（优先级 3）

- **目标**：§2.3——全表扫描移除**当前层全部**型 11、恒成功消耗出栈。
- **目标层/预置**：floor37（8 堵墙，见 §1.5），玩家 (8,9) 开阔处，道具栈 `[18×1]`。
- **输入脚本草图**：同道具键序（-6 → -5 → -6）。
- **断言点**：
  - 状态：T090 **8 个墙实体快照全部** `entityState` 0→1（一个不剩；数量断言 = 8）；
    `itemStackUses[0]` 1→0 + `itemStackSize` 1→0；gameMode→3；
  - 帧窗：T083 起 8 墙同时消失的单帧跳变（与 17A 单墙帧变不同——帧窗断言"同一 tick 全消失"）。
- **目标夹具文件名**：`crates/game-core/tests/fixtures/preset-f37-item18.{preset,txt,frames.txt}`。
- **销账**：evidence-items 速查表 **18 行**、§2.3、§3.4（与 17 的范围差）。顺带覆盖商人语义之外的唯一使用臂。

### FX-ITEM-19A：千年月桂露回血精确值（优先级 3）

- **目标**：§2.4——`hp += (atk+def)*74/10` 整数取整、提示文案、消耗出栈。
- **目标层/预置**：floor1，道具栈 `[19×1]`，**atk=105、def=203、hp=500**（取非 10 倍数钉取整语义：
  (105+203)*74/10 = 308*74/10 = 22792/10 = **2279**，若 porter 误写浮点或先除会得 2279.2/2280）。
- **输入脚本草图**：同道具键序（-6 → -5 → -6）。
- **断言点**：
  - 状态：T090 `playerHp` 500→2779（=500+2279，精确值断言）；overlay 文案 = "增加了2279血量"；
    `itemStackSize` 1→0；
  - 帧窗：T043-T080 说明 overlay、T083-T100 提示 overlay（"增加了N血量"）。
- **目标夹具文件名**：`crates/game-core/tests/fixtures/preset-f1-item19.{preset,txt,frames.txt}`。
- **销账**：evidence-items 速查表 **19 行**、§2.4、**§3.6**（整数除法先乘后除）。

### FX-BOSS-67A：啸天犬击杀钩子 → spawnBossEvent(3) 金勺子（优先级 4）

- **目标**：evidence-boss §2.2——击杀 67 → `spawnBossEvent(3)` 刷金勺子(17)@(6,4)（DES 态）；
  战后 switch 落 default（无脚本）。
- **目标层/预置**：floor15，玩家 (6,6)（boss 67@(6,5) 北邻），hp/atk/def=1000（一合合击杀：
  67 数值 HP1200/Atk180/Def20，攻 1000 ⇒ 伤害 980/合，2 合内死；def 1000 ⇒ 无伤）。
  **设计注记**：(6,5) 同格叠有型 5 门卫与型 67（sprite15 字节级实证）——bump 交互按
  interactWithCell 槽倒序扫描（a.java:5377 起）先碰槽序后者；若门卫臂先触发（阻挡/对话），
  本场景改为先处理门卫再 bump（或预置把玩家挪到 (5,5) 侧向 bump）。**task-6 落地时先跑一次探路再定键序**。
- **输入脚本草图**：
  ```
  T010 -1 press / T012 -1 release     # bump (6,5) 开战（R-boss-1）
  T020-T200 …（tickBattle 节拍，按实测；确定键推进攻击）
  T210 -5 press（拾取 (6,4) 金勺子，落格 pickupItemType(17)，a.java:7548）
  ```
- **断言点**：
  - 状态：开战拍 `battleTargetEntity` 指向 67、`walkPhase`=5、`entityState`=3（R-boss-1 五元组）；
  - 击杀拍：boss 实体移除 + **新实体 type=17 @ (6,4) 且 DES 态**（markEntityDesRemoved，R-boss-3）；
    `goldAmount` += 100（enemyBaseGold[26]=100，evidence-boss §2.1）；
  - 收尾 switch 落 default（断言无 `currentScriptIndex` 变化、无 dialogPhase 进入 11）；
  - T220 拾取后 `itemStackTypes[0]=17`、uses=[1]；
  - 帧窗：开战动画帧窗（walkPhase=5）、死亡 6 拍动画窗、掉落物出现帧、拾取提示帧。
- **目标夹具文件名**：`crates/game-core/tests/fixtures/preset-f15-boss67.{preset,txt,frames.txt}`。
- **销账**：evidence-boss §0 **67 行**、§2.2、R-boss-2（查表 spawnBossEvent(idx)）、R-boss-3（DES 态掉落）。

### FX-BOSS-74A：巨灵神双钩子 + 遗言重放（优先级 4）

- **目标**：evidence-boss §3.2——击杀 74 → ① `spawnBossEvent(1)` 12 件掉落；② `loadLevelScript(21)`
  遗言（TAK_59_59，无 tryRunScene 门控）；**每次击杀都重放遗言**（"待补核对点"钉成断言）。
- **目标层/预置**：floor20，玩家 (6,7)（boss 74@(6,6) 北邻），hp/atk/def=1000（74 数值 HP444/Atk199/Def66
  ⇒ 一合合击杀）。
  **设计注记**：(6,6) 同格叠有 74、83 剧情点、5 门卫（sprite20 字节级实证）——83 剧情点（script6
  登场戏 TAK_53_58）会先于战斗臂触发（槽序）；设计采用"先被 script6 对话推进完（48 键）再进战斗"的
  键序变体，或预置 `scriptLineFlags[6]=true` 跳过登场戏（更短，推荐）。
- **输入脚本草图**（flags[6]=true 变体）：
  ```
  T010 -1 press / T012 -1 release     # bump (6,6) 开战
  …战斗节拍…（击杀）
  T220-T400 48 × N                     # 推进遗言 dialog（script21 = TAK_59_59）
  …第二轮：再次击杀 74（预置再刷一只或 respawn）→ 再次遗言 → 证明无门控重放
  ```
- **断言点**：
  - 击杀拍 1：`spawnBossEvent(1)` 12 件掉落快照（29@(4,4)(4,5)(5,4)、26@(7,4)(8,4)(8,5)、
    30@(4,7)(4,8)(5,8)、32@(8,8)(7,8)(8,7)，evidence-boss §3.2 引 a.java:2092-2141）；
  - 击杀拍 2（同拍/次拍）：`currentScriptIndex`=21、dialogPhase 进入台词、speaker=74；
  - **重放断言**：第二只 74 击杀后 `currentScriptIndex` 再次 =21（**无 flags[21] 门控拦截**——
    行为合同，禁止 porter 加门，evidence-boss §3.2"待补核对点"）；
  - 帧窗：死亡动画、12 掉落物出现帧、遗言对话帧（speaker 74 头像帧）两轮各一段。
- **目标夹具文件名**：`crates/game-core/tests/fixtures/preset-f20-boss74.{preset,txt,frames.txt}`。
- **销账**：evidence-boss §0 **74 行**、§3.2 全行（含"待补核对点：重复触发语义"）、R-boss-2 双臂、R-boss-5
  （loadLevelScript 无门控语义）。

### FX-MVS-A：MVS 型 82 配对互传（条件场景——仅当 porter 实现 MVS 臂）

- **前置（必须先裁定）**：MVS 现有死码裁决 **DC-2**（docs/spec/dead-code-rulings.md DC-2 节 +
  coverage-matrix m_107 行：语料 68 脚本 0 调用、型 82 出层 0 个）。
  **若维持裁决 ⇒ 本场景冻结不启用**；若 porter 实现 MVS 臂（或裁决被推翻）⇒ 启用。
- **体例**：**Rust 合成状态单测**（§1.4：Java 侧无法被驱进 MVS ⇒ 无 frames.txt 差分）。
- **合成场景**：注入一对型 82（同 `param&0xff`）于格 (2,2)/(5,5)，各格一只 cat8 怪（型 41，同 param），
  调 executeScriptInstruction("MVS")（或走含 MVS 的合成 dialogScriptLine）。
- **断言点**（状态迁移，对齐 evidence-interact-types R-INT-82 语义总结）：
  1. 两怪**双向互传**：原 (2,2) 怪出现在 (5,5)、原 (5,5) 怪出现在 (2,2)（m_107 复制-删除语义：
     目标格 spawnEntity(同型同 param) + 旧格全同型移除——quirk：旧格同型多只一并消失，保真断言）；
  2. 两型 82 点位 `entityRemoved`=true（removeGroupAtCellByType 消耗，一次性）；
  3. 怪物总数守恒（单向 m_107 是复制+删旧 ⇒ 总数不变）；
  4. `scriptCursor` 恰推进 1（a.java:8591 无参指令游标语义）；
  5. 反例：仅一只型 82（不成对）⇒ 无任何状态变化（配对槽判空逻辑）。
- **目标文件名**：`crates/game-core/tests/mvs_pair_scene.rs`（单测，非 fixtures/ 三件套）。
- **销账**：evidence-interact-types **R-INT-82** MVS 半（语义总结"传怪动作"4 条）、
  evidence-todos **R-TODO-1a**（MVS 臂行为合同）；若走死码终裁路线则改销 dead-code-rulings DC-2 复核行。

### FX-SCRIPT-UNKNOWN-A：真未知 op 静默停摆（条件场景）

- **体例**：Rust 合成单测（§1.4 同理）。
- **合成场景**：合成脚本行 `"XYZ_1 "`（3 字符标签外）+ 已知后续指令。
- **断言点**（R-TODO-1b 语义总结）：逐 tick `script_cursor`/`dialog_phase` **不变**（停摆）、
  `scriptLineFlags` 不置位、行内后续指令永不执行；对照 op 缺席时的默认行为**禁止**按 4 字节跳过。
- **目标文件名**：`crates/game-core/tests/script_unknown_stall.rs`。
- **销账**：evidence-todos **R-TODO-1b** 全行。

---

## 3. 扩展候选（本批之外，设计先行）

| 场景 id | 目标 | 要点 | 销账 |
|---|---|---|---|
| FX-ITEM-25A | 幸运币金钱×2 被动 | floor1 有 41-46 敌群；预置栈 `[25]`（uses=-1）→ bump 战杀一只 41 → 断言 `gold` 增量 = `enemyBaseGold[0]<<1` 且栈不变（-1 永不消耗）。敌 BaseGold 以 constants 表为准 | items 速查表 **25 行** + §2.7 + §3.2/§3.7 |
| FX-BOSS-74B | currentFloor>50 跳过钩子（负例） | floor53（74@(7,4)，且 53>50）击杀 ⇒ **无** spawnBossEvent、**无**遗言（R-boss-2"50 层以上整体跳过"分支钉死） | boss R-boss-2 >50 分支 + §0 74 行 Rust 现状列 |
| FX-E2E-67-SPOON-WALL | 整合链 | floor15：杀 67 → 拾 (6,4) 金勺子 → 走到 (8,1) 墙 → 使用 17 凿墙。长输入脚本，验三段衔接 | boss §2.2 + items §2.2 联动 |
| FX-RENDER-67 | T-2 多段拼装渲染 | floor15 boss 67 在视野内逐帧（frame-sha 覆盖 paint 拼装臂 scene.rs:5603）；67 state=3 抖动帧与 animRow 3 三帧（帧偏移 0/1/2）全覆盖 | todos **T-2** 全行 |
| FX-INT-11-F23 | 型 11 文案 23 层静默 | floor23 bump 墙 ⇒ 无 overlay 但仍阻挡（R-INT-11）；可并入 FX-ITEM-17A 前置阶段 | interact-types **R-INT-11** |

---

## 4. 生成与验收流程（task-6 执行序建议）

1. **预检**：`python3 gates/cli.py status`（照 AGENTS.md §5）。
2. **探路**：先跑目标层确定键序（尤其 FX-BOSS-67A/74A 的同格重叠实体槽序），写定 `.txt` 输入。
3. **Java 参照帧生成**（无头，照 mov_route_scene.rs 头注释再生命令，stem 换新夹具）：
   A/B/C 三变体并跑 → trace sha256 三方一致 + A 重跑一致 → 按 TICK 抽 `FRAME sha=` 写 `<stem>.frames.txt`。
4. **Rust 对拍接入**：仿 `mov_route_scene.rs` 的 harness（parse_preset/parse_floor_save → GameScene
   逐 tick 喂输入 → 全帧 sha 比对）+ 状态检查点断言（FLD 号登记）。
5. **销账**：每场景过闸后，按 §0 表到对应证据文档刷账（原地标注日期销账，不删历史——AGENTS.md §3.6）。
6. **顺序建议**：FX-INT-6A → FX-DOOR-2A/2B → FX-ITEM-17B → FX-ITEM-19A → FX-ITEM-17A → FX-ITEM-18A
   → FX-BOSS-67A → FX-BOSS-74A →（条件）FX-MVS-A/FX-SCRIPT-UNKNOWN-A。
   理由：先钉小状态机（显形/门），再钉道具消耗协议，最后钉战斗钩子长脚本。

## 5. 设计注记与遗留假设（B 级，不入结论）

- **同格重叠实体槽序**（FX-BOSS-67A/74A）：sprite15 的 (6,5) 叠 5/67、sprite20 的 (6,6) 叠 5/74/83——
  interactWithCell 倒序扫槽，先触发哪只由 cellSlotTable 插入序决定（出层顺序）；探路实测定键序后
  写入 `# cover:` 头。
- **键码表**（§1.2）依据现有夹具用法与 a.java 分派推断（B 级），-5/-6/-7 的软键语义以探路实测校准。
- **FLD 号**：状态检查点键名已定，FLD 编号需 task-6 生成 trace 时登记；未登记的断言不可执行。
- **FX-ITEM-19A 取整钉法**：若 Java `applyHpDelta` 内部对 hp 上限有钳制（未考），断言值需按探路修正；
  先用低血量预置避开上限。
- **MVS/未知 op 无差分体例**（§1.4）是硬约束不是省事：Java 脚本源封闭（DC-2 字节码复核），
  任何"合成脚本行喂 Java"的夹具设想都不可执行。
- **popup 时机**（补遗#1）：FX-DOOR-2A 的事件序断言需要 trace 记录 popup 事件序——若现行 trace
  不记 popup 序，降级为"同拍同帧即可接受"并在账上标注放宽原因（禁止无据放宽）。
