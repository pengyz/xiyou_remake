# 确定性求解器实施计划（solver-implementation-plan）

> 2026-11-09 · 分支 `modern` · 状态：**计划（未开工）**
> 需求来源：用户 2026-11-09 裁定（派工单 m00001 转述）——求解器是 P3 编辑器/mod 的**前置安全网**；
> 架构风格参照：`docs/superpowers/specs/2026-11-03-dddsa-architecture-design.md` §4/§5；
> 纪律依据：`AGENTS.md`（证据 A/B/C、门禁表、逐提交报审）。
> 行号说明：本文所有代码行号均为 2026-11-09 本会话 `grep -n`/`sed` 实测复核（复核清单见 §8），
> 非转抄自任何简报；引用时请以 `文件:行` 现场复核为准（本项目行锚漂移是惯犯）。

---

## 0. 全局约束与评审焦点

**Global Constraints（违反任一条即打回）**

- [ ] 本计划只授权**新增 `crates/game-solver`** 与 `docs/findings/` 报告、测试夹具；不改 `original/`、`reference/`、`data/tables/`（保真表冻结）、`gates/`。
- [ ] 求解器 lib 纯函数零 I/O；`game-solver` → `game-data`/`game-core` 单向依赖，**不得被 game-core/game-data 引用**（无环）。
- [ ] 求解语义复用真游戏公式（`predict_battle_hp_loss` 等），**禁止求解器内重写一套战斗算术**；与现解释器/真引擎对拍，禁自证。
- [ ] 不发明行为：语义拿不准的标 **【待考证】** 并给考证路径（见 §5）。
- [ ] 每批判据 = 可复制命令 + 预期输出；每批 = 报 lead → **独立对抗复核子 agent** → 绿后推送（用户 m00963 裁定）。
- [ ] 金样 `python3 tools/state_baseline.py --check` 与 `python3 gates/cli.py check` 在每批结束时必须绿；求解器新增测试**不得**使 ratchet 回退。
- [ ] 计划≠已完成：本文全篇为待执行项，checkbox 未勾 = 未做。

**Review Focus（对抗复核看这里）**

1. 行号是否与现场一致（抽查 §2/§4 各 3 处）。
2. 批次判据是否可复制运行、预期输出是否具体。
3. 三查询定义是否忠实于用户裁定（最大化血量 / 最少杀怪 / 禁终极装备可达）。
4. 是否把"计划"写成了"已完成"。

---

## 1. 背景与目标

### 1.1 用户裁定（不可改）

用户 2026-11-09 原话（paraphrase+关键引语，转述自派工单 m00001）：

> "在开始修改前，我们需要一个确定性求解器，**保证修改不影响可通关性，同时能评估游戏难度**；游戏本身是一个数学求解问题……有些逻辑是写在脚本里的，如何量化，然后用求解器做一轮求解，包括**剩余血量最大，剩余怪物最多（最少杀怪），不拿终极装备能否通关**？这是后续 mod 修改的基建。"

→ 求解器 = P3 编辑器/mod 的**前置安全网**；**验收 = 三查询各出一轮解**。

### 1.2 三查询定义（验收口径）

设：初始状态 `S0 = (hp=300, atk=10, def=10, floor=51, 钥匙=0, 金币=0, 背包=∅)`（依据 §2.5）；
通关判定 = 脚本行 65 的 `END_0` 被执行（`op_end → GameMode::Ending`，§2.6）。
一次解 = 从 S0 到通关的**合法动作序列**（走格/撞敌/开门/踩门触发脚本/换层），全部转移经 §3 的真游戏公式计算。

| # | 查询 | 目标函数 | 约束 |
|---|------|---------|------|
| Q1 | 剩余血量最大 | **最大化**通关时 `player.hp` | 存在通关路线 |
| Q2 | 最少杀怪（=剩余怪物最多） | **最小化**通关路径上击杀的怪物数（等价最大化存活怪物） | 存在通关路线 |
| Q3 | 不拿终极装备能否通关 | 判定型：**可达性 0/1** | 通关路线上**不装备**终极装备集（集合口径见 §2.7，批0 裁定） |

口径细则（写死，防跑偏）：

- Q1/Q2 的"解"必须是**完整通关解**，不是局部最优；两者允许不同路线（Q1 可多杀强怪换金买药，Q2 可绕路避战）。
- Q2 "杀怪"= 使敌方 HP 归零的战斗；**Refused 战斗（打不过）不计入杀怪**（它根本没发生）。
- Q3 的"终极装备"= 候选 {36 金箍棒, 40 紫金龙鳞甲}（tier 最高 bonus220，§2.7）；**是否并入 79/80 次级档属批0 裁定项**，裁定后写死为常量。
- 三查询均在**有界求解**口径下报告（§1.3），解须标注 `optimal | bounded-best | feasible` 三级置信。

### 1.3 明确不承诺

- **不承诺全局最优**：完整状态空间 NP-难（钥匙/金币/装备/怪物/脚本旗标交叉），只做**有界求解**：分层分解 + 每层资源规划 + 层间关键道具 DAG，界内报最优、界外报可行解并注明界。
- 不做实时/帧级模拟：求解器是**回合抽象模型**（移动=格、战斗=闭式），与真引擎的等价由批5 oracle 回放保证。
- 不替代差分门禁：求解器绿 ≠ 游戏对；它只回答"还能通关吗、难在哪"。

---

## 2. 现状盘点（全部行号 2026-11-09 实测）

### 2.1 关卡几何（maplv）

| 事实 | 证据 |
|------|------|
| 55 个地图文件 `assets/raw/maplv0..54` | 目录实测 |
| 头 4 字节 = u16 LE ×2，**raw = 真实×2，读取时 `>>1`** | `crates/game-data/src/maplv.rs:51`（注释锚 a.java:7179）、`:69-70` `let width = (raw_width >> 1)` |
| `parse_partial` :65 / `parse`（consumed==len 零余字节）:93 / `tile_at` :117 | `maplv.rs` |
| **实测 maplv0 头 `1a00 1a00` = raw 26×26 → 真实格 13×13**（1356B = 4+13×13×4×2 验算自洽） | `xxd -l 4 assets/raw/maplv0` + 字节数验算 |
| ⚠️ 转写简报"26×26 典型"是 raw 值误当真实尺寸；**真实典型 = 13×13**（oracle `walk_replay_floor1.rs` 亦按 13×13） | 同上 |
| L1 golden：55/55 解析零尾字节 | `crates/game-data/tests/golden.rs:49 fn maplv_all_55_parse_zero_trailing` |

### 2.2 对象布局（sprite）→ 实体

| 事实 | 证据 |
|------|------|
| 55 个 `assets/raw/sprite0..54`；头 u16 LE×2（count_lo + count_hi<<16）；记录 = type(1B)+x(u16LE)+y(u16LE)+extra(0/1/2B) | `crates/game-data/src/sprite.rs:107 parse_partial`、`:133 parse`、`:54 extra_len`（2B={7,8,76,82}、1B={4,5,9,57,59,70,71,72,73,77,78,81,83}） |
| **全 55 层 census（python 按上述格式 parse，零错位）：总记录 1703、敌人 502（type41..75 全 35 型出现）、45 层有敌人**；type12 门=24、type13=1；楼梯 type7(上)=54、type8(下)=51；**type81 封印门=5、type82 MVS 对=0、type83 场景门=52、type72 ROUTE=4** | 本会话 census 脚本输出 |
| type83 param=脚本号：0..16,18,19,24..34,38,40..50,52,53,56,64,66,67（24/34 各 3 个）；**缺 17/20-23/35-37/39/51/54/55/57-63/65** | 同上 |
| type72 ROUTE 4 条：floor2(96,224) / floor15(288,32) / floor29(96,64) / **floor50(192,160,ex=1)** | 同上 |
| 生成参数/可见性：`spawn_param_and_visibility(type_code, extra, doors) -> (i16, bool)`，**match 臂覆盖 type 4..83 + `_ => 0`**（转写简报"4..87"不准） | `crates/game-core/src/sprite_spawn.rs:23`；DoorTables{door12,door13} `:18` |
| 实体类别常量 | `crates/game-core/src/enums.rs:164` `pub mod entity_kind`：STAIR_UP=7 `:166` / STAIR_DOWN=8 `:168` / ROUTE=72 `:170` / NPC_BODHI=87 `:172` / SEAL_GATE=81 `:174` / MVS_PAIR=82 `:177` |
| 生产装载链：`vec![None; 56]` + `enumerate().skip(1).take(54)` 只装 maplv1..54/sprite1..54（**槽 0、55 为 None**），每记录经 spawn 参数化后 `*slot = Some(FloorAssets{maplv, records})` | `shell/common/src/handoff.rs:97,98,111,117` |
| 场景装载 `scene.load_floor_data(maplv, px, py, records, floor)` | `crates/game-core/src/scene.rs:612` |

### 2.3 属性表（data/tables）

| 表 | 行数 | 关键字段 | 证据 |
|----|------|---------|------|
| `enemies.json` | 35（id 41..75） | hp/atk/def/gold/behavior.trait_flags | 文件实测；trait_flags 稀疏（49/53/74=1、69=2，表内 evidence 注） |
| `items.json` | 32（id 13..40,79,80,85,86） | 见 §2.7 | 文件实测 |
| `shop_goods.json` | 17 | P1/P2 已聚合成记录 | 文件实测 |

### 2.4 战斗闭式（求解器"与真游戏同算"的复用面）

| 函数 | 语义 | 证据 |
|------|------|------|
| `effective_attack_vs_type(player, trait_flags) -> i32` | trait_flags&1 → TRAIT1_AMULET(23) 命中 ×2；&2 → TRAIT2_AMULET(24) ×2 | `crates/game-core/src/combat.rs:93`（分支 `:96-98`）；trait 源 `combat.rs:42 trait_flags_for` |
| `predict_battle_hp_loss(atk, def, enemy_hp, enemy_atk, enemy_def, armor_gate) -> i32` | hits=⌈enemy_hp/(atk−def)⌉；loss=(enemy_atk−def)×(hits−1)；`!armor_gate && enemy_atk>def` 再加首击差；**atk≤def ⇒ -1** | `combat.rs:106`（hits 上整 :115-120、多击 :121-124、首击 :126-128） |
| `predict_engagement(player, enemies, enemy_base, t, no_counter) -> i32` | idx=t−41；trait_flags 取 enemy_base；atk 走 effective_attack_vs_type | `crates/game-core/src/subsystem/battle.rs:85` |
| `resolve_bump(...) -> BattleOutcome`；**Refused = loss<0 或 loss≥hp** | `if !(loss >= 0 && loss < ctx.player.hp) return Refused` | `battle.rs:79`（trait）、`:110`（impl）、`:122`（判据）、`Refused` 变体 `:33` |
| BumpEngage=撞格无先手(armor_gate)、RouteEngage=途中敌先手 | 调用点 `scene.rs:2045 / 3592 / 3739` | 实测 |
| 钥匙消耗：`consume_key_for_door(player, door) -> (bool, i32)`（黄26/红27/蓝28 各 −1，无钥 (false,0)） | | `combat.rs:170`（体 :175-188） |
| PlayerCombat 字段（**逐层 checkpoint 的字段源**）：hp/atk/def/yellow_keys/blue_keys/red_keys/gold/item_types/item_uses/item_stack_size | | `combat.rs:51` |
| 掉落金：battle 胜利 +gold（幸运币 item25 则 gold<<1） | | `crates/game-core/src/battle.rs:189-192` |

### 2.5 初始状态（转写简报错误点之一）

- **`assets/raw/script` 的 `GAME:` 段（:1-13：STLAYER=51/EDLAYER=51/HP=300/ATK=10/DEF=10/RKEY=BKEY=YKEY=0/HOT=0/SMS=0/WHOSYOURDADDY=0/END）无任何 Rust 代码读取（A 级）**：全仓 grep `STLAYER|EDLAYER|WHOSYOURDADDY` 零命中（仅 `golden.rs:226-247` 词法测试读 GUTS 段）；考证记录 `docs/findings/script-dsl-semantics.md` §2（a.java getResourceAsStream 仅 4 处都不是 script 文件；且 GAME: HP=300 与字段初值 this.J=498 不一致，a.java:482-485）。
- **实际新游戏初始**：`crates/game-core/src/intro.rs:361` `pub fn new_game_stats() -> (i32, i32, i32, i32) { (300, 10, 10, 51) }`（hp/atk/def/floor=51；文档注锚 a.java:3482-3489 步骤8）。**该函数当前无调用方 = 接线点【待考证】**（考证路径：grep 调用点为零 → 查 a.java 步骤8 的实际调用处 → 批0 决定求解器直接取常量还是等接线）。
- 本计划 Q1-Q3 采用 `S0 = (300,10,10,51, keys=0, gold=0, ∅)`，与 new_game_stats 一致。

### 2.6 脚本 DSL 与"通关终点"（转写简报错误点之二）

**双真相源（A 级）**：

- **运行时真相源 = `crates/game-core/tests/fixtures/level_scripts.txt`**（68 槽 index=层号 this.n[0..67]，空槽 39/58/59/60；3609 字节 / 68 行实测）：`crates/game-core/src/scene.rs:252-253` `DialogData::from_fixtures` 用 `include_str!("../tests/fixtures/level_scripts.txt")` 载入；`scene_persist.rs:219` `dialog_data: DialogData::from_fixtures()`。
- `assets/raw/script`（79 行 CRLF）**与运行时数组 index 对齐 diff 实测 5 槽不一致：[0,18,27,31,63]**（层0 文件多 `DES_84_6_11`；层18 fixture 多 `DES_1_4_4`；层27 `DES_84_11_4` vs `DES_84_6_11`；层31 文件多 `DES_10_6_7`；层63 `DES_-59_0` vs `DES_-66_0`）。
  → **求解器输入必须用 fixture 源，不用 assets/raw/script**（`assets/raw/script` 仅作 GUTS 语料参考）。

**语料规模**：GUTS 段 **64 编号行、297 条指令**（与 `script_ops.rs:318-324` 注释"297 条"一致）；opcode 分布 TAK 81 / CES 48 / MOV 47 / DES 41 / ROS 30 / SEE 30 / GUT 7 / GIN 4 / SWD 3 / LAY 1 / RES 1 / ADD 1 / GLV 1 / END 1 / SMS 1 / **MVS 0**。

**词法（复用，禁重写）**：`crates/game-data/src/script.rs` — `OPCODES: [&str;16]` `:25`（CES,MOV,TAK,DES,ROS,SEE,GUT,GIN,ADD,GLV,RES,SWD,MVS,LAY,END,SMS）、`is_known_opcode` `:31`、`split_instr_token` `:53`、`lex_instr_stream` `:93`、`parse_guts_line` `:111`。

**解释器与触发链**：

| 事实 | 证据 |
|------|------|
| `pub enum ScriptOp` **23 变体**（转写简报"32"错）：Tak/Lay/Gin/Res/Add/See/Gut/Mov/DesAll/DesAt/Swd/Ces/Glv/End/Sms/RosMove/RosEquip/RosHp/RosFace/RosAtk/RosDef/RosNoop/Mvs | `crates/game-core/src/script_ops.rs:6` |
| **`parse_op` :401 与 `execute_op` :553 均 `pub(super)`（crate 私有）⇒ game-solver 不可直调，须自建转移函数表** | `script_ops.rs:401,553` |
| 关键 op 语义（A 级，带 a.java 锚）：`op_end` :269 → `transition_to(GameMode::Ending)`（**通关终点**，a.java:8445-8463）；`op_gut` :167 → `self.load_level_script(target)`；`op_swd` :226 → 扫表 type==81 解封（visible=true, type_=4, state=2, anim_row=8）；`op_gin` :78 kind0=物品/kind1=金币；`op_res` :101（仅 easy difficultyIndex==0 生效）重置钥匙/道具/金币=4/攻=防=10/hp=400；`op_ros_equip` :289 `try_equip(tier,true)`；`op_ros_move` :282 瞬移玩家；**`op_glv` :264 与 `op_sms` :279 体为 `{}` 空操作** | `script_ops.rs` 逐行实测 |
| **MVS 双重死码**：注释原文"语料 297 条指令 0 条 MVS + 全 55 层 sprite 型 82=0…语料不可达，为忠实而保"——本会话 census 独立复证（MVS=0、type82=0） | `script_ops.rs:318-324` + census |
| 双解释器：GameScene 集成路径 `scene_persist.rs:501 load_level_script(&mut self, idx: i32)`（pub）→ `scene_script.rs:14 execute_script_instruction`（parse_op→execute_op，未知 op 静默停摆 a.java:8594-8595）；另一 `ScriptEngine`（`crates/game-core/src/script.rs:154`，测试用）仅服务 script_golden/oracle 测试 | 实测 |
| **战斗胜利→脚本**：型69→floor==35 时 load(36)；型70\|71→floor==49 时 load(35)；**型72→floor==50 时 `load_level_script(65)`**（`scene.rs:3891-3894`）；型73/74/75→load(22/21/20)（`scene.rs:3804-3820`） | `scene.rs` 实测 |
| 脚本行 65 = `TAK_127_128 END_0`（`assets/raw/script:77`）⇒ **通关 = floor50 击败型72 → END** | 实测 |
| 场景门：type83 踩格/出生 → `try_run_scene(param, from_walk)`（`scene.rs:926` 定义、`:4068` 踩格、`:3496` step 内），过门才 `load_level_script`；case 按号特判（case15 活型5 挡等） | `scene.rs:926,3496,4068` |
| 楼梯换层：type7 `change_floor(param&0xff,false,false)`（`scene.rs:4079`）、type8 同理（`:4085`） | 实测 |
| `change_floor(target, down, strict)`（`scene_persist.rs:349`）：**target<0 或 >55 直接拒绝**（floorCountMax=55，a.java:808）；save/swap/reload 链 :383/:410/:444 | `scene_persist.rs:349,362,366` |

**层号语义辨析（防混淆）**：GUTS 行号 = **脚本行 index（0..67）**，与**楼层号（1..54）是两套编号**——两者仅通过 type83 门（param=脚本号）与战斗胜利触发（floor==50→65 等）关联。求解器数据结构里必须分字段命名（`script_idx` vs `floor`），禁混用。

### 2.7 物品与"终极装备"口径

`items.json` 全 32 行 effect key 实测（摘要）：

- **武器**：33桃木棒(t1,10) / 34玄铁棒(t2,30) / 35真银棒(t3,70) / 79乌金棍(t4,120) / **36金箍棒(t5,220)**；
- **护甲**：37道袍(t7,10) / 38铁甲(t8,30) / 39锁子甲(t9,70) / 80银缕锁甲(t10,120) / **40紫金龙鳞甲(t11,220)**；
- **钥匙**：26黄/27红/28蓝、85黄金钥匙(RemoveAllType)；**属性**：29威力丹(atk)/30金刚丹(def)；**血**：31回春丸(×50)/32长寿丹(×200)；**治疗**：19千年月桂露；
- **清障**：16芭蕉扇(RemoveAdjacent)/17金勺子/18玄明石(RemoveAllType)；**面板**：13火眼金睛/14生死簿；**QoL**：15筋斗云(TutorialSeenStack)/20相形变位/21上行符/22下行符/23太公杖/24捆仙绳；**被动**：25幸运币(击杀金×2)/86日月无极钹(KillAdjacent4)。

→ **Q3 终极装备集候选 = {36 金箍棒, 40 紫金龙鳞甲}**（tier 最高、bonus220）；79/80 为次级（bonus120）。**是否并入 79/80 = 批0 裁定项**，裁定后写死常量并在 §7 验收表登记。

### 2.8 可复用的求解基建

| 组件 | 用途 | 证据 |
|------|------|------|
| `findPath` 忠实移植（best-first cost=曼哈顿和，扩展上限 `++table_count < 100`，**可通行 = terrain ‖ 格有实体槽**） | 层内移动可行性 | `crates/game-core/src/pathfind.rs:1-16`、Walkability `:22-27` |
| `change_floor` / `find_floor_gate` / `save_floor_state` / `reload_floor_entities` | 层间转移与状态保存 | `scene_persist.rs:349 / 435 / 383 / 444` |
| **金样 74 场景** = 已知通关样例对拍锚 | 批5 验证 | `data/status/baselines/state-dump-golden.json`（scenarios=74 实测）；`python3 tools/state_baseline.py --check`（`tools/state_baseline.py:6`） |
| game-oracle 无头回放先例 | 批5 oracle 回放 | `crates/game-oracle/tests/walk_replay_floor1.rs`（真实 floor1 maplv+sprite，Rust walk vs Java 逐采样像素）、`boot_replay.rs`（逐 tick FRAME sha）、`preset_cases.rs`（GameScene 构造）、`l2_prologue.rs`（脚本驱动） |
| `tools/route_gen.py`（**手写路线走步填充，非规划器**：BFS 填航点+三重校验） | 与求解器互补：求解器产航点 → route_gen 转 oracle 输入脚本 | `tools/route_gen.py:1-30` |
| 门禁命令集 | 每批判据 | `python3 gates/cli.py status/check/reference/trace/all/...`（--help 实测） |

### 2.9 门禁现状（2026-11-09 复跑）

`python3 gates/cli.py status` → **PASS=13 FAIL=0 SKIP=1 TOLERATED=0**：rust-tests 全绿；test-ratchet total=1022(基线978) 无回退（备注：新增二进制待 ratchet-update `{'shop_goods_defs(test)': 6}`）；table-content-hash/table-evidence 全绿；asset-truth-manifest entries=138；reference-regression refs=707 shim=69 missing=0；oracle 6/6（确定性/等价/运行/stderr/C等价 PASS）；diff-trace ok；**SKIP=visual-diff（P4 后启用，已显式打印）**。

---

## 3. 架构设计

### 3.1 crate 位置与依赖方向

```
crates/game-solver/            ← 本计划唯一新增 crate（workspace glob "crates/*" 自动入列）
├── Cargo.toml                 # deps: game-data, game-core（单向）
├── src/
│   ├── lib.rs                 # 纯函数库：禁 fs/net/std::process（单测断言无 I/O 依赖）
│   ├── model.rs               # SolverState / 抽象层 / checkpoint 五元组+背包
│   ├── ingest.rs              # 批0：输入构造（maplv/sprite/fixture 脚本/三表 → SolverInput）
│   ├── script_abs.rs          # 批1：23 枚举 → 转移函数表 + A/B/C 分类
│   ├── solve_local.rs         # 批2：单层求解 MVP
│   ├── solve_global.rs        # 批3：全局分层求解 + 状态抽象
│   └── query.rs               # 批4：Q1/Q2/Q3 三查询
└── src/bin/solver.rs          # bin：唯一允许 I/O 的入口（读 assets/data →调 lib → 输出 JSON）
```

依赖方向（无环保证）：

```
game-solver ──▶ game-data ──▶ (纯解析)
     │
     └────────▶ game-core ──▶ game-platform (trait)
game-core / game-data ──X──▶ game-solver   （禁止反向引用；CI/单测可查：cargo tree -i game-solver 仅 solver 自身）
```

- **纯度边界**：`lib.rs` 下所有模块是纯函数/纯数据结构——输入 `SolverInput`（由 bin 或测试构造），输出 `Solution`；**禁 `std::fs`/网络/时钟**（批0 单测加一条 `cargo build --release` + grep 依赖树断言）。
- **bin 负责读 assets**：`src/bin/solver.rs` 读 `assets/raw/maplv*`、`assets/raw/sprite*`、`crates/game-core/tests/fixtures/level_scripts.txt`（**fixture 源，见 §2.6**）、`data/tables/*.json`，构造 `SolverInput` 后调 lib。
- **不改 game-core**：`parse_op/execute_op` 私有没关系——批1 由 solver 自建转移函数表，用**对拍**（与现解释器/真引擎）而非直调保证同语义（§4 批1）。若对拍暴露必须直调的需求，另立提案改 game-core 暴露 `pub` 门面，**不混入本计划批次**。

### 3.2 求解形态（论证 lead 倾向：采纳，附修正）

lead 倾向"分层求解（单层资源规划 + 层间关键道具 DAG）+ 状态抽象（hp 离散化/背包 canonical），不承诺全局最优"。本计划**采纳**，理由：

1. 楼梯/换层把状态天然切成 54 层子问题（`change_floor` 保存/恢复实体与玩家状态，`scene_persist.rs:383/444`），层内图小（13×13 典型，§2.1），单层可穷举或 A*；
2. 层间只有少量耦合量：hp、三色钥匙、金币、装备、关键道具（脚本 GIN/ADD 产出）、脚本旗标（script_line_flags）——DAG 规模 = 54 节点 + 边上的资源约束；
3. 与"评估难度"目标匹配：每层的杀怪代价/钥匙需求本身就是难度指标。

**修正**（相对 lead 简报）：

- 求解器输入源 = **fixture 脚本**而非 assets/raw/script（§2.6 双真相源，5 槽不一致）；
- ScriptOp 是 **23 枚举**非 32；maplv 真实格 **13×13** 非 26×26；type_code 生成臂 **4..83** 非 4..87；初始状态 = `new_game_stats` 非 GAME: 段；
- 单层求解须处理**场景门/战斗胜利触发的脚本**（type83 ×52、型69..72 触发），它们会 DES/ADD/MOV 改层内实体——层不是静态图。

### 3.3 状态抽象（唯一算法难点，批3 主攻）

```rust
// 概念形状（批3 落地时以此为准，字段名不许漂）
struct SolverState {
    floor: u8,                 // 1..54（与 script_idx 严格分字段，§2.6）
    hp: DiscreteHp,            // 离散化：批3 裁定桶宽（建议按敌人一击伤害粒度）
    atk: i32, def: i32,        // 取值域小（10..~装备后），可精确
    keys: (u8, u8, u8),        // 黄/红/蓝，上界=层内门数，canonical
    gold: DiscreteGold,        // 离散化（商店价目最小刻度）
    inventory: CanonicalBag,   // item_type 集合 + uses，排序去重 canonical
    flags: ScriptFlags,        // script_line_flags 位图（try_run_scene gate 用）
    kills: u32,                // Q2 目标函数累积
    monsters_alive: u32,       // Q2 等价口径（全图存活数）
}
```

- **canonical**：背包排序、钥匙/金币离散桶 → 状态去重的前提；同一抽象态合并（保留 Q1/Q2 各自的 Pareto 前沿：hp 最大 ↔ kills 最小是双目标，批3 用分层字典序或 Pareto 集，口径在批3 报审时定死）。
- **保真红线**：hp 损失一律调 `predict_battle_hp_loss`（§2.4），禁止 solver 内自算 hits/loss。

---

## 4. 批次切片

> 执行纪律（用户 m00963 裁定，写死）：**每批 = TDD 实现 → 成员 commit 绿 → 报 lead → 独立对抗复核子 agent → 红回派/绿裁定 → 推送**。
> 每批开始前跑 `python3 gates/cli.py status` 看清 SKIP/恒红项（AGENTS §5）；每批结束跑 `python3 gates/cli.py check`。
> 估时为区间（乐观–悲观），按 1 人 + 对抗复核往返计。

### 批0 · 导入器 = 求解器输入构造 + 快照测试 【估时 1–2d】

**内容**

- [ ] 建 `crates/game-solver` 骨架（Cargo.toml deps: game-data, game-core；lib 纯函数 + bin）。
- [ ] `ingest.rs`：maplv（复用 `MapLv::parse`）+ sprite（复用 `SpriteTable::parse` + `spawn_param_and_visibility`）+ fixture 脚本（68 槽，**用 `include_str!` 指向 `game-core/tests/fixtures/level_scripts.txt`**）+ enemies/items/shop_goods 三表 → `SolverInput`。
- [ ] 初始状态 `S0 = (300,10,10,51,∅)` 取自 `intro.rs:361 new_game_stats`（接线点【待考证】按 §2.5 路径考证，结论写入 commit message）。
- [ ] **Q3 终极装备集裁定**：{36,40} vs 并入 {79,80}——由 lead/用户确认后写死常量。
- [ ] 快照测试：`SolverInput` 摘要（55 层尺寸/实体数/502 敌人/68 脚本槽/三表行数）与 §2 census 数字对拍。
- [ ] 层号↔地图资源映射【待考证】（GUTS 层号至 67 但 maplv 只 1..54）：考证路径 = a.java loadLevelScript(:7997-8012) + GUT_n/LAY 全量交叉引用（`docs/findings/script-dsl-semantics.md:152` 遗留7 同族）；结论入 `docs/findings/`。

**判据**

```bash
cargo test -p game-solver                 # 预期：全部通过（快照数字与 §2 census 一致）
cargo run --release -p game-solver --bin solver -- ingest --summary
# 预期输出（节选）：floors=55 enemies=502 scripts=68 enemies_table=35 items=32 shops=17
cargo tree -p game-solver -i game-solver  # 预期：无 game-core→game-solver 反向边
python3 gates/cli.py check                # 预期：任一 FAIL=0（PASS 汇总行绿）
```

**依赖**：无（首批）。**TDD**：先写 census 快照测试（红）→ 实现 ingest（绿）。

### 批1 · 脚本抽象解释层 = 23 枚举 → 转移函数表 + A/B/C 分类 【估时 2–3d】

**内容**

- [ ] `script_abs.rs`：把 23 个 `ScriptOp` 变体映射为 `fn(&mut SolverState, &SolverInput) -> ()` 转移函数表（**自建，不依赖 crate 私有的 parse_op/execute_op**；词法复用 `game-data::script` 的 `lex_instr_stream`/`parse_guts_line`）。
- [ ] **A/B/C 分类**（修正版，按 op 实测语义）：
  - **A 状态变换（进转移表）**：GIN(给金/物)、ADD(增实体)、DES(删实体)、CES(改类型)、MOV(移实体)、ROS_2..6(强制装备/HP/攻/防/朝向)、ROS_1(瞬移)、LAY(换层)、SWD(封印门解除)、RES(easy 重置)、GUT(脚本跳转)；
  - **B 纯展示（求解器剥除，不进状态）**：TAK、SEE、SMS、**GLV（体为空 `{}`，op_glv:264）**、END（END 单独处理=终止符，不算状态变换）；
  - **C 玩家干预/边界（求解器按约束处理）**：MVS（**死码**：语料 0 条 + sprite type82=0，表内留 no-op 分支并注释证据）、END(终止)。
  - ⚠️ 转写简报把 GLV 归 A、把 END 归 B——**按实测语义修正**（GLV 空体、END 是通关判定），修正理由写入单测注释。
- [ ] 分类单测：每 opcode 至少 1 条 fixture 语料驱动的转移断言（A 类改状态、B 类状态不变、C 类按约束）。
- [ ] **与现解释器对拍**（禁自证）：构造同一 fixture 行 → (a) 经 solver 转移函数表、(b) 经 game-core 现有解释路径（`ScriptEngine`（`script.rs:154`，pub）或 GameScene `load_level_script`（`scene_persist.rs:501`，pub））→ 断言可观测状态一致（hp/gold/keys/实体集合）。

**判据**

```bash
cargo test -p game-solver --lib script_abs   # 预期：A/B/C 分类单测全绿（含 23 变体逐一覆盖）
cargo test -p game-solver --lib 对拍          # 预期：fixture 64 行逐行对拍 0 差异
python3 gates/cli.py check                    # 预期：FAIL=0
```

**依赖**：批0。**TDD**：先写"GLV/SMS 状态不变""RES easy 重置"等红测试 → 实现转移表。

### 批2 · 单层求解 MVP = 一层跑通可达 + 最优 hp + 最小杀怪 【估时 2–3d】

**内容**

- [ ] `solve_local.rs`：选**一层**（建议 floor1：oracle 有 walk_replay_floor1 真地形先例，§2.8）做全量求解：可达性（复用 findPath 语义/或 solver 内 BFS，可通行=terrain‖实体槽，`pathfind.rs:22-27`）+ 战斗闭式（`predict_battle_hp_loss`）+ 开门（`consume_key_for_door`）+ 层内脚本触发（type83/型69..75 胜利触发的 A 类转移）。
- [ ] 三个出口都出解：`reachable: bool`、`max_hp: Option<i32>`（带路线）、`min_kills: Option<u32>`（带路线）。
- [ ] 路线输出为航点序列（兼容 `tools/route_gen.py` 的 `x,y` / `bump 型@x,y` 语法，§2.8——为批5 铺路）。

**判据**

```bash
cargo run --release -p game-solver --bin solver -- solve-local --floor 1
# 预期输出（形状）：{"floor":1,"reachable":true,"max_hp":<N>,"min_kills":<M>,"route":["10,11",...]}
cargo test -p game-solver solve_local          # 预期：可达性与手工构造的小图样例一致；hp/kills 单调性断言绿
python3 gates/cli.py check                     # 预期：FAIL=0
```

**依赖**：批1。**TDD**：先用手工 3×3 夹具写红测试（1 强怪挡路：能杀则 reachable、杀不过则 Refused 不计杀）→ 实现。

### 批3 · 全局分层求解 + 状态抽象 【估时 3–5d】

**内容**

- [ ] `solve_global.rs`：层间 DAG（换层边 = 楼梯 param 指向 + LAY/GUT + 型72 触发），层间传播 `SolverState`（§3.3），checkpoint 字段对齐 `PlayerCombat`（`combat.rs:51`）。
- [ ] 状态抽象落地：hp/gold 离散桶宽裁定（报审时定死并写注释）、背包 canonical、script_line_flags 位图；状态去重 + 双目标（Q1 hp 最大 / Q2 kills 最小）Pareto 或字典序（口径批3 定死）。
- [ ] 有界求解口径：界（扩展数/时间）可配，解标注 `optimal|bounded-best|feasible`。
- [ ] 确定性：同输入同解——固定遍历序（层号→格→动作字典序），无 HashMap 迭代序依赖。

**判据**

```bash
cargo test -p game-solver solve_global     # 预期：分层一致性测试绿（逐层结果拼接 == 全局解的 hp/kills）
cargo run --release -p game-solver --bin solver -- solve-global --bound 1e6 --out /tmp/sol1.json
cargo run --release -p game-solver --bin solver -- solve-global --bound 1e6 --out /tmp/sol2.json
diff /tmp/sol1.json /tmp/sol2.json         # 预期：无差异（确定性复跑）
python3 gates/cli.py check                 # 预期：FAIL=0
```

**依赖**：批2。**TDD**：两层玩具图（钥匙门跨层）先红 → 实现 DAG 传播。

### 批4 · 三查询跑轮 + 结果入 docs/findings 【估时 1–2d】

**内容**

- [ ] `query.rs`：Q1/Q2/Q3 各跑一轮全图求解。
- [ ] 产出 `docs/findings/2026-11-XX-solver-queries.md`：三查询解（路线摘要 + 数值 + 置信级 `optimal|bounded-best|feasible`）+ 难度评估（每层杀怪代价/钥匙瓶颈/终极装备依赖度）。
- [ ] findings 必须按 AGENTS 区分 **已证实**（求解器算出的）与 **假设**（界内未证最优等）。

**判据**

```bash
cargo run --release -p game-solver --bin solver -- query --all --out docs/findings/solver-queries.json
# 预期输出：Q1 max_hp=<N>(<置信级>) / Q2 min_kills=<M>(<置信级>) / Q3 reachable=<true|false>
python3 -c "import json;d=json.load(open('docs/findings/solver-queries.json'));assert{'q1','q2','q3'}<=set(d)"
python3 gates/cli.py check                 # 预期：FAIL=0
```

**依赖**批3，**但 Q1/Q2/Q3 的真验证在批5**——批4 的 findings 结论先标"待批5 oracle 回放确认"。

### 批5 · 验证闭环 = oracle 回放 + 金样对拍 + 确定性 【估时 1–2d】

**内容**

- [ ] 解 → oracle 无头回放：把批4 路线喂给 game-oracle（参照 `walk_replay_floor1.rs` / `route_gen.py` 语法），断言路线在**真引擎**走通（到达 Ending）。
- [ ] 金样对拍：`python3 tools/state_baseline.py --check` —— 74 场景逐字节等价（重构前后主判据，`tools/state_baseline.py:6`）。
- [ ] 确定性复跑：三查询连续跑 2 次 `diff` 无差异（批3 已做，批5 在全量数据上复验）。
- [ ] 回放通过后，把批4 findings 的"待确认"标记翻成"已证实"。

**判据**

```bash
cargo test -p game-oracle solver_replay     # 预期：解路线在真引擎回放 0 断言失败
python3 tools/state_baseline.py --check     # 预期：[GREEN] 金样逐字节等价：74 场景 sha 全同
python3 gates/cli.py all                    # 预期：FAIL=0（visual-diff 仍 SKIP 且显式打印）
```

**依赖**：批4。

**批次依赖链**：批0 → 批1 → 批2 → 批3 → 批4 → 批5（串行；批0 的两项【待考证】考证可与批1 并行）。

---

## 5. 风险与难点

| # | 风险 | 影响 | 对策/考证路径 |
|---|------|------|--------------|
| R1 | **状态爆炸**（hp×金×钥匙×装备×旗标×层） | 批3 超时/内存 | 离散桶 + canonical 去重 + 有界求解（界可配）；报审时给界与耗时曲线；不承诺全局最优（§1.3） |
| R2 | **隐藏墙双通道破坏决策**：可通行 = terrain ‖ 实体槽（`pathfind.rs:22-27`），实体被脚本 DES/ADD/CES/MOV 动态改（A 类），且 sprite 有 hide 分支（spawn 可见性）——静态图会误判可达 | 可达性假阳/假阴 | 求解器层内图**每步按当前实体集重建**，不缓存静态图；与 findPath 对拍（批2） |
| R3 | **剧情强制 ROS_2/DES 可达性约束**：脚本强制换装/清状态/删实体（op_ros_*、op_des_*、op_res），可能使"最优路线"被剧情否决 | 解不可执行 | A 类转移全量进状态机（批1）；批5 oracle 回放兜底——回放不过=解作废 |
| R4 | **行为键 trait_flags 已入公式**：`effective_attack_vs_type` 的护身符 ×2 分支（`combat.rs:96-98`）依赖 trait_flags 稀疏表（49/53/74=1、69=2） | 漏算=hp 低估 | 战斗一律调 `predict_battle_hp_loss`+`effective_attack_vs_type`（§2.4），禁自算；单测覆盖 4 个 trait 敌 |
| R5 | **双真相源**：assets/raw/script vs fixture 5 槽不一致（§2.6） | 用错源=解不可回放 | ingest **只读 fixture**（批0 判据钉死）；assets/raw/script 仅语料参考 |
| R6 | **层号 vs 脚本 index 两套编号**（§2.6） | 换层/触发错位 | 数据结构分字段 `floor`/`script_idx`，类型层面禁混（批0 review 焦点） |
| R7 | **ScriptOp parse/execute 私有**（`script_ops.rs:401,553`） | 求解器不能直调现解释器 | 批1 自建转移表 + 与现解释器对拍（禁自证）；若必须直调，另立提案改 game-core（不混入本批） |
| R8 | **MVS/type82 死码、GLV/SMS 空体** | 分类误判 | 按实测语义归类（批1 分类表），死码留 no-op 分支+证据注释 |
| R9 | **GUTS 层号 >54、maplv 只 1..54**（§2.6） | 层映射断裂 | 批0 【待考证】：a.java loadLevelScript(:7997-8012) + GUT_n/LAY 交叉引用；结论入 findings |
| R10 | **new_game_stats 无调用方**（§2.5） | 初始状态接线不确定 | 批0 考证 a.java 步骤8 实际调用处；S0 常量先按 (300,10,10,51) 写死并注证据 |

**不发明行为**：凡表中未列的语义（如商店购买的精确时序、难度 difficulty_index ×16 机制 `scene.rs:978/1031/4199`、a.java:5172-5184）一律标【待考证】，考证路径 = 对应 a.java 锚 + 运行时插桩，不进结论。

---

## 6. 与 P3 编辑器的关系（回归门）

P3 编辑器（`docs/superpowers/specs/2026-11-03-dddsa-architecture-design.md` §4：本地 Python 服务、Rust 唯一真值、零改码演练）落地后，**任何 mod/编辑器改动 → 求解器回归门**：

```
改数据（items/enemies/maplv/sprite/script）
   → cargo run -p solver -- query --all   # 三查询重跑
   → 与基线 JSON diff：
        Q3 reachable 必须不变（false→true 允许，true→false = 破坏可通关性，打回）
        Q1 max_hp / Q2 min_kills 越界（变难超过阈值）→ 提示"难度不越界"复核
   → 同时 gates check + state_baseline --check 必须绿
```

- 基线 JSON 入库（建议 `data/status/baselines/solver-queries.json`，随 P3 批次更新，更新须写 commit 说明——同 ratchet 纪律）。
- 编辑器验收（spec §4 的零改码演练）与求解器正交：演练保证"没乱改"，求解器保证"改了还能玩、难度可控"。
- **求解器是 P3 的前置**：本计划批0–批5 全绿之前，P3 的"改数据"通道不得宣称可安全试错。

---

## 7. 验收总表

| # | 验收项 | 判据（可复制命令） | 预期输出 |
|---|--------|--------------------|---------|
| A1 | Q1 剩余血量最大出一轮解 | `cargo run --release -p game-solver --bin solver -- query q1` | `Q1 max_hp=<N> confidence=<optimal\|bounded-best\|feasible> route_len=<L>` |
| A2 | Q2 最少杀怪出一轮解 | `... -- query q2` | `Q2 min_kills=<M> confidence=<…>` |
| A3 | Q3 禁终极装备可达性 | `... -- query q3 --ultimate36,40`（集合=批0 裁定常量） | `Q3 reachable=<true\|false>` |
| A4 | 三查询结果落盘 | `... -- query --all --out docs/findings/solver-queries.json` + §4 批4 的 python assert | assert 通过，json 含 q1/q2/q3 |
| A5 | **确定性复跑** | 连跑 2 次 `--query --all` 到不同文件后 `diff` | 无差异 |
| A6 | **oracle 回放**（解在真引擎走通） | `cargo test -p game-oracle solver_replay` | 全绿，到达 Ending |
| A7 | **金样对拍** | `python3 tools/state_baseline.py --check` | `[GREEN] 金样逐字节等价：74 场景 sha 全同` |
| A8 | 门禁总闸 | `python3 gates/cli.py check` | FAIL=0（visual-diff SKIP 显式打印，属已知项） |
| A9 | 纯度/无环 | `cargo tree -p game-solver -i game-solver` + lib 无 fs/net 断言测试 | 无反向依赖 |
| A10 | 分类对拍（批1） | `cargo test -p game-solver --lib` | 23 变体覆盖 + fixture 64 行对拍 0 差异 |

**样例输出格式**（A1–A3 统一 JSON，写死防漂）：

```json
{"query":"q1","max_hp":847,"confidence":"bounded-best","bound":1000000,
 "route":["10,11","10,12","bump 41@11,12","door 11,13", ...],
 "kills":12,"monsters_alive":490,"floors":[51,50,49,...]}
```

---

## 8. 本文实测行号复核清单（防"照抄未核"）

**已实测复核（2026-11-09，grep/sed/xxd/python 现场验证）**：
maplv.rs:51,65,69-70,93,117 · sprite.rs:54,107,133 · sprite_spawn.rs:18,23 · enums.rs:164,166,168,170,172,174,177 · handoff.rs:97,98,111,117 · scene.rs:252-253,612,926,3496,3804-3820,3891-3894,4068,4079,4085 · scene_persist.rs:349,362,366,383,410,435,444,501 · combat.rs:42,51,93,106,170 · subsystem/battle.rs:33,79,85,110,122 · script_ops.rs:6,167,226,264,269,279,282,289,318-324,401,553 · game-data/script.rs:25,31,53,93,111 · game-core/script.rs:154,193 · intro.rs:361 · pathfind.rs:1-16,22-27 · golden.rs:49,226-247 · state_baseline.py:6 · route_gen.py:1-30 · Cargo.toml(workspace):12-13 · walk_replay_floor1.rs 头注 · tables 三表行数 · gates status/check 子命令 · xxd maplv0 头 · census 脚本输出（1703/502/45/type83×52/type72×4）· fixture 68 行 · assets/raw/script:77 · script-dsl-semantics.md §2。

**未核/待批0 现场复核（引用前必须重新 grep）**：
- a.java 行锚（7179/808/8445-8463/8594-8595/9869-9928/3482-3489/482-485/9497 等）——均系从 Rust 注释/doc 转引，**未打开 reference/ 现场核**；
- `scene.rs:2045/3592/3739`（RouteEngage 调用点，转引）、`scene.rs:978/1031/4199`（difficulty_index，转引）、`battle.rs:189-192`（掉落金，转引）；
- `script-dsl-semantics.md:152` 遗留问题清单（引用其编号，未复读该行）；
- `docs/superpowers/specs/2026-11-03-dddsa-architecture-design.md` §4/§5（本会话早前读过，行号未记——引用只写 §号不写行号）。

---

## 9. 执行工作流（逐批，写死）

每批循环（AGENTS §9 + m00963）：

1. **TDD**：先写红测试（判据命令对应），再实现至绿。
2. **成员 commit 绿**：`python3 gates/cli.py check` FAIL=0 → 按 `type(scope): subject` 提交（scope=solver；触及台账才需 Evidence 字段）。
3. **报 lead**：贴 commit hash + 判据命令实跑输出。
4. **独立对抗复核**：lead 派子 agent 复核（红回派 / 绿裁定）。
5. **绿后推送**（本计划起草阶段不推送，留 lead 复核）。

批0 起点命令：

```bash
cd /home/peng/workspace/xiyou_remake && git status -sb   # 确认 modern 分支、工作区干净
python3 gates/cli.py status                              # 看清 SKIP（visual-diff）与 ratchet 备注
```
