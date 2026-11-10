# 求解器批2「单层求解 MVP」设计提案

> 2026-11-10。**用户 2026-11-10 裁定「按计划切片走，先出提案再复核」**：本件即求解器实施计划 §4 批2（[solver-implementation-plan.md:316-335](../plans/2026-11-09-solver-implementation-plan.md) 实测范围块）的**设计提案**；先出提案、过对抗复核，通过后另行派工实施。
> **只立文档：本提案不改任何代码、不改计划。** 实施切片（§5）须待本件过对抗复核后按计划 §9 工作流（:486-494）派工。
> 全部行号 = 2026-11-10 在当前 HEAD `b976803`（分支 modern）上 sed/grep 实测；`git status` 除既有未跟踪项（`android/build/`、`docs/findings/reviews/*.md`）外树净。**凡本提案未实测者一律标【待复核】，不装实。**
> ⚠ **引锚漂移提示**：计划正文引用的 `scene.rs` 行号系计划成文时旧 HEAD 所测（如 `try_run_scene` 计划引 `:926`、踩格引 `:4068`），本仓 scene.rs 此后 +1（`scene_solver_api.rs:36-38` Y3 已登记同源漂移）。**本提案一律以 HEAD `b976803` 实测值为准**：`try_run_scene` :927、踩格 :4069、`try_step` :3315、门臂 :3386。

## 0. 裁定与边界

- **范围权威** = 计划 §4 批2 :316-335：`solve_local.rs` 选**一层**（计划建议 floor1）做全量求解——可达性（findPath 语义/solver 内 BFS，可通行=terrain‖实体槽）+ 战斗闭式（`predict_battle_hp_loss`）+ 开门（`consume_key_for_door`）+ 层内脚本触发（type83/型69..75 胜利触发的 A 类转移）；三出口 `reachable` / `max_hp` / `min_kills`；路线输出兼容 `tools/route_gen.py` 航点语法。判据块 :326-332，TDD 起手式 :334。
- **三查询裁定（用户 2026-11-09）**：①剩余血量最大 ②最少杀怪（不拿终极装备能否通关=③变体）；解合法性=通关+QoL 全收集（计划 §0 :21）——**全局**口径在批3 全局层生效，本提案在单层切面上的处理见 §1.4/§2.1。
- **本提案解决什么**：批2 单层 MVP 的问题形式化（状态/动作/转移/目标）、单层边界（入口→出口、脚本触发、暗墙 probe）、TDD 红测试清单与计划 §7 A 编号归属、fixture 选层理由、确定性钉法、风险回退、切片 commit 序列。
- **本提案不解决什么**：批2 实现本身（`solve_local.rs` 尚未建）；批3 全局层间传播与状态离散化裁决（计划 §3.3 :240-258 明确「批3 主攻」）；不改计划、不改门禁、不改 game-core 行为。
- **红线不变**（AGENTS.md §2/§3；计划 §0 :20）：**不得重写已门面化的逻辑**——战斗/开门/寻路/脚本一律直调 game-core 既有 `pub` 面（§1.3 分工表即红线的可执行表述）；game-core 无 I/O、确定性；items 22-25 effect 未考证前以原版 activateItem 为准（计划 :20）。

---

## 1. 单层求解 MVP 的问题形式化

### 1.1 状态最小充分集（逐个论证 + 实码出处）

单层求解状态记 `s = (pos, terrain, ents, stats, bag, keys, flags, qol, kills)`。**取舍判据**：一个分量进状态 ⇔ 存在某条转移的**读取面**依赖它（否则可剔除）；反之只要有一条合法转移读它，剔除即错。**qol 不进 K₀ 的冗余性论证**：qol=筋斗云在 bag（type15 ∈ item_types；批2 动作空间无丢弃/转移通道，§1.2 枚举）⇒ qol 可由 K₀ 的 bag 分量派生，按上判据属派生量非独立分量；`s` 中保留 qol 仅作输出读回面（断言便利），不进 K₀——若批3 引入丢弃/转移通道则升独立分量【待复核】。逐项：

| # | 分量 | 为什么必须（读取面） | 实码出处（HEAD 实测） |
|---|------|--------------------|----------------------|
| 1 | `pos=(x,y)` 玩家格 | 走/撞/踩格全部按坐标寻址；路线输出即 pos 轨迹 | `view.player_px/py` 读回见 `script_abs.rs:191` 侧 Obs（`px/py` 字段）；格寻址 `scene.rs:3315` `try_step` |
| 2 | `terrain: Vec<Vec<bool>>` 层地形 | 寻路可通行的静态面（=terrain‖实体槽的 terrain 分量） | `pathfind.rs:21-27` `Walkability`（terrain + has_entity_slot + cols/rows）；floor1 真地形先例 `crates/game-oracle/tests/walk_replay_floor1.rs`（maplv1 + `tile_walkability.txt`，13×13） |
| 3 | `ents` 实体占据（门开关/怪存活/道具在否/暗墙态） | 走撞判定读实体槽；开门=实体移除；脚本 A 类变换改实体 | 门臂 `scene.rs:3386-3407`（开门 `entity::remove` :3404）；实体表 `crates/game-core/src/entity.rs:57`（`EntityTable`，批0 起 pub）；脚本直改经门面（§1.3） |
| 4 | `stats.hp/atk/def` | Refused 判据读 hp（`loss < ctx.player.hp`）；闭式读 atk/def；29/30 号丹改 atk/def（批2 不用道具，但键须预留——见下） | `combat.rs:51-63` `PlayerCombat`（全 pub 字段）；判据 `subsystem/battle.rs:121` |
| 5 | `keys=(黄,红,蓝)` | 开门转移读/写 | `combat.rs:51-63`；写面 `combat.rs:170` `consume_key_for_door` |
| 6 | `bag`（`item_types/item_uses/item_stack_size`） | 脚本 TAK/DES、落格拾取读写；Q3 禁装集合判定（批4） | `combat.rs:60-62`；拾取经 pub `try_step`（`scene.rs:3315`）落格触发（`apply_step_cell_effects` `scene.rs:3987` 内调 `scene.rs:4094`；`pickup_item_type` `scene.rs:4221` 为**私有内部，不直调**） |
| 7 | `flags` = `script_line_flags` 位图 | **一次性门**：`try_run_scene` gate 初值 `!script_line_flags[idx]`——不进键则门可重复触发，状态机直接错 | `scene.rs:418`（字段）、`scene.rs:931`（gate 初值）、`scene.rs:609-611`（scene() 初始化） |
| 8 | `difficulty`（本层不进键，**钉常量**） | case67 门读 `difficulty_index` | `scene.rs:1032`（case67 `gate &= difficulty_index > 0`）；默认 0 见 `scene_persist.rs:133` `difficulty_index: 0`（新局默认；菜单改值路径【待复核】） |
| 9 | `kills`（Q2 累积） | 目标函数读 | 计划 §3.3 :252 `kills: u32`（概念形状，字段名不许漂） |
| 10 | `qol` 收集位（本层=筋斗云 type15 捡否） | 解合法性=通关+QoL 全收集（计划 :21）的**层内分量**；floor1 恰有 type15×1（census 实测） | 计划 :21；floor1 census 见 §3.2 |

**明确不进单层状态的分量（论证）**：

- `gold`：floor1 无商人（census：无 type77/78 商人格），层内无任何转移读 gold ⇒ 可从**等价键**剔除；但读回面保留（A11 五元组要金币，`combat.rs:58` `pub gold`）。批3 商店层再入键。
- `floor`：单层 MVP 恒 =1，不进键（批3 才是 `SolverState.floor`，计划 §3.3 :245）。
- `enemy hp`（敌残血）：批2 动作空间只含 **bump 主动交战至死或 Refused**（§1.2 裁剪），不存在中途撤离的残血态 ⇒ 敌只需存活位（ents 分量）+ 战斗时现算。途中遭遇 RouteEngage（`scene.rs:2046`）会引入敌残血——**批2 裁出动作空间**，理由与回退见 §4.2 BR-6。
- `monsters_alive`：计划 §3.3 :253 保留字段，单层内由 ents 派生（非独立键），批3 全局口径时再 canonical。
- **离散化字段**（`DiscreteHp/DiscreteGold/CanonicalBag`，计划 §3.3 :245-250）：批3 裁定；**批2 单层一律精确整数**（13×13、无商店、hp 带宽有限——精确键天然 canonical，不引入桶宽争议）。

### 1.2 动作空间（按原版动作集裁剪，枚举清单 + Rust/Java 语义）

批2 动作集 `A = {Move, BumpEnemy, BumpDoor, StepPickup, StepStair, StepGate} ∪ 夹具专属 {Probe, BumpHiddenWall}`；**裁出**项显式列出防遗漏（原版动作全集的其余部分批4+ 扩）。

| 动作 | 原版语义（Java 锚=Rust 注释转引） | Rust 实锚（HEAD 实测） | 批2 |
|------|----------------------------------|------------------------|-----|
| `Move(dir∈{上,下,左,右})` | tryStep：目标格+interactWithCell 放行（a.java:5335-5356 注） | `scene.rs:3315` `pub fn try_step` | ✅ 内 |
| `BumpEnemy(type41..75)` | 撞怪开战，BumpEngage=撞击面无先手（a.java:5470-5490 注） | `scene.rs:3593` `resolve_bump(..., BumpEngage)` | ✅ 内 |
| `BumpDoor(type1黄/2红/3蓝)` | 撞门耗钥，无钥阻挡（a.java:5391-5411 注） | `scene.rs:3386` 门臂 → `scene.rs:3402` `consume_key_for_door` | ✅ 内 |
| `StepPickup` | 落格拾取道具/钥匙（a.java:7525-7621 注） | pub `try_step`（`scene.rs:3315`）落格 → `apply_step_cell_effects`（`scene.rs:3987`）内 `scene.rs:4094-4095`（拾取+remove）；`pickup_item_type` `scene.rs:4221` 私有不直调 | ✅ 内 |
| `StepStair(type7/8)` | 踩梯 `change_floor(param&0xff,…)`（a.java:5660/5666 注） | `scene.rs:4077-4082`（7 臂，change :4080）、`:4083-4088`（8 臂，change :4086） | ✅ 内（=**终止边**，§2.1） |
| `StepGate(type83)` | 踩场景门→过门才载脚本（a.java:5668 区注） | `scene.rs:4069` `try_run_scene(param,true)` → `scene.rs:4073` `load_level_script` | ✅ 内（§2.2） |
| `Probe(dir)` | 暗墙试探（双通道 T3 接面；BUG-038 场上非空闲拒绝） | `scene.rs:1553` `pub fn probe_step` | ⚠ 仅手工夹具（floor1 无暗墙，§2.3） |
| `BumpHiddenWall(type12→11)` | 隐形墙被撞变型11（gameplay.md R-move-4，a.java:3638-3661 注） | `scene.rs:3551-3576`（`bump_reveal_target(12)` :3562） | ⚠ 仅手工夹具 |
| `RouteEngage` 途中遭遇 | 敌先手开战（a.java:8771-8787 注） | `scene.rs:2046` | ❌ 裁出（BR-6） |
| `UseItem`（activate_item：回春丸/金刚丹/上行符…） | 表查询+逐动作执行 | `crates/game-core/src/item_effect.rs:141/147`（ChangeFloorUp/Down 臂）；`try_equip` `scene.rs:869` | ❌ 裁出（BR-4/风险见下） |
| 胜利特判换层（型69..75） | 战斗胜利触发 A 类换层 | `scene.rs:3804-3820`（73/74/75→load 22/21/20）、`:3882-3895`（69→36、70\|71→35、72→65） | ⚠ **不裁出计划范围**（计划 :320 明列）：floor1 census 无此类怪；批2 以**手工夹具**覆盖 A 类胜利换层转移建模（T8 扩展），真层不跑 |
| 商店/Equip/宝箱组门（5/76） | — | `scene.rs:4089` 注（cat1 特例块） | ❌ floor1 无 |

**裁出项的代价声明**：`UseItem` 裁出 ⇒ 批2 `max_hp` 是「**不使用道具**子策略空间内的最优」，可能低于全局最优（floor1 有回春丸 type31×4/长寿丹 type32 可回血）。这与计划 A1 的 `confidence` 三态（`optimal|bounded-best|feasible`，§7 :448）口径自洽——批2 单层版输出一律标 `bounded-best`（策略空间受限但空间内最优），不冒充 `optimal`。若复核裁定批2 必须含 UseItem，回退=§5 C6 后追加切片 C6b（估时 +0.5d）。批2 因 UseItem 裁出而**无筋斗云换层边**（type15 捡拾只入 qol 位、不激活；换层通道=批3 层间边，计划 R11 :418 归批3，§4.1 已列）。

### 1.3 转移计算分工（红线：不得重写已门面化的逻辑）

**判据**：凡是 game-core 已有 `pub` 实现的语义，求解器只许**直调**，不许复刻公式/判据/状态机；solver 自己算的只有四样：搜索编排、去重与支配、目标函数、路线序列化。

| 转移语义 | 谁算 | 直调面（实锚） | 禁止事项 |
|----------|------|----------------|----------|
| 战斗损失 | **直调** game-core | `subsystem/battle.rs:85` `predict_engagement`（读缩放表，内部走 `combat.rs:93` + `combat.rs:106`） | ❌ 禁止 solver 内自算 hits/loss（计划 §3.3 :258 保真红线原话） |
| 开战/Refused 判定 | **直调** `BattleResolver::resolve_bump` | `subsystem/battle.rs:107`（ZST `BattleResolver`）+ `:110` 入口 + `:121` 判据在**其内部** | ❌ 禁止把 `:121` 判据抄成 solver 本地 if（抄=复刻已门面化逻辑）。做法：solver 自备 BattleCtx 的 10 个写槽（`subsystem/battle.rs:59-75` 全 `pub` 字段）+ 由 ingest 提供 `enemies/enemy_base` 表，构造 `BattleCtx` 直调拿 `Engage/Refused` |
| 开门耗钥 | **直调** | `combat.rs:170` `consume_key_for_door` | ❌ 禁止自写「有钥减一」 |
| 层内寻路（可达性） | **直调** | `pathfind.rs:55` `find_path` + `pathfind.rs:21` `Walkability` | ❌ 禁止复刻其怪癖（扩展上限 100、cost=曼哈顿和，`pathfind.rs:1-16` 头注） |
| 层内最短路（最优步数） | **solver 自算 BFS**（计划 :320 明文允许「或 solver 内 BFS」） | 与 `find_path` **对拍 found 一致性**（计划 §5 R2 :409 对策原话「与 findPath 对拍（批2）」） | BFS 不得声称= findPath 语义；两者只对拍可达结论 |
| 层内脚本触发 | **直调门面** | `scene_solver_api.rs:59` `execute_script_instruction`（+`:45 parse_op`），驱动模式复用批1 `script_abs.rs:191` `side_a::drive`（行驱动到停摆）；终态读回用 `script_abs.rs:191` `side_a::drive` 返回的 `Obs`（`flags` 字段 `script_abs.rs:76`，由 `:208` `sc.script_line_flags.clone()` 灌入；另有 stats/px/py/entities）；difficulty/floor 等 Obs 没有的字段才用 `Snapshot::from_scene`（`script_abs.rs:118`）补读 | ❌ 禁止自建 23 枚举转移表（批1 方案 B 裁定，facade 提案 §1） |
| 换层 | **只读不执行**（单层 MVP） | 直调 `change_floor` 所在 harness 面读 `persist.pending_floor`（赋值 `scene_persist.rs:386`；`change_floor` 入口 `:349`）；落点语义查 `swap_floor`（`:410-434`）与 `find_floor_gate`（`:435-443`） | ❌ 单层不展开 `swap_floor`/层重载（批3 层间边） |
| 拾取 | **直调**（经 pub 面） | pub `try_step`（`scene.rs:3315`）落格触发 → `apply_step_cell_effects`（`scene.rs:3987`）内 `scene.rs:4094` 拾取；`pickup_item_type`（`scene.rs:4221`）私有不直调 | ❌ 禁止自写物品效果表（计划 §0 :20）、禁止直调私有面 |
| 状态编排/去重/支配/目标/路线序列化 | **solver 自算** | `crates/game-solver/src/solve_local.rs`（拟新增） | 纯度红线：无 I/O、无 `std::time`/`std::fs`（计划 A9 :456） |

**BattleCtx 装配的可行性论证**：`BattleCtx<'a>`（`subsystem/battle.rs:59-75`）10 个写槽 + 3 个读槽全 `pub`，`BattleResolver` 是无状态 ZST（:107 注释「禁 GameScene 加字段」），`EnemyDef` 表由批0 `ingest.rs` 提供 ⇒ 求解器可在**自有内存**里构造 ctx，不引 GameScene、不引平台 crate。装配语义（`battle_enemy_hp` 分源：撞击面=基础表 `enemy_base[idx].hp`、途中面=缩放表，:127+ 注）与真引擎的逐位一致性 = BR-2 风险面（§4.2），由夹具对拍钉住。

**harness 复用**：脚本转移需要 `&mut GameScene`——批1 已给出完整先例：`script_abs.rs:191` `side_a::drive(sc, idx)`（唯一 import `solver_api` 处，驱动到停摆，Obs 含 stats/flags/px/py/entities 全读回面）+ `scene_test_support`（`crates/game-core/src/lib.rs:31-33` 测试夹具导出，非生产 API）。批2 沿用同一模式：**脚本转移 = 从等价态灌一个 harness GameScene → drive → 读回 → 并回抽象态**；不把 GameScene 当搜索态本体（169 格搜索 × GameScene 克隆 = 爆炸，见 §4.1）。

### 1.4 目标函数（①max hp ②min kills 的形式化）

**解与合法性**（单层切面）：动作序列 `π = a₁…aₙ`，转移 `T(s,a)`；`π` 合法 ⇔ 每步 `T(s,a) ≠ ⊥`（Refused/越界/无钥撞门=⊥，不可行边，**不产生状态**）∧ 终态 `sₙ.pos ∈ Exit`（§2.1 出口集合）。QoL 位（筋斗云捡否）在批2 **记录不作硬约束**——单层无「全收集是否达成」的全局判定面，硬约束随批3 层间传播生效（计划 :21 是全局口径；本条是切面顺延，待复核确认不构成对 :21 的削弱）。

- **① max_hp**：`max_hp = max_{π 合法} hp(sₙ)`，`route_hp = argmax`（并列取步数少者——tie-break 钉死为确定性的一部分，§3.3）。floor1 无治疗动作（UseItem 裁出）⇒ `hp(sₙ) ≤ hp(s₀)`，天然可断言。
- **② min_kills**：`min_kills = min_{π 合法} kills(π)`，其中 `kills(π) = |{i : aᵢ = BumpEnemy ∧ T(s,aᵢ)≠⊥}|`。**Refused 不计杀**由「⊥ 不产生状态」直接推出，与计划 :334 TDD 起手式（「杀不过则 Refused 不计杀」）同构。
- **③ 变体（禁终极装备）**：批2 不实现（计划归批4 A3）；形式化上= 合法性追加 `equip ∩ {36,40} = ∅`（`state.rs` `Q3_EQUIP_SET`，批0 裁定常量），单层内因 UseItem/Equip 裁出而**恒真**——批2 无需动作即满足，故不占切片。

**多目标怎么合：提案 = 不合，双查询独立出解，理由三条**：

1. 计划判据本身就是三出口分立（:321 `reachable/max_hp/min_kills` 各带路线；A1/A2 :448-449 各自 JSON 断言）——联合目标无验收面。
2. 两目标真实冲突（少杀路线要绕路/多开门，血更优路线要清怪）：合并必须选权重或字典序，而**字典序方向本身是口径裁定**——计划 §3.3 :257 已把「分层字典序或 Pareto 集」标注为「批3 报审时定死」，批2 抢跑会与批3 裁定打架。
3. 搜索实现上并不重复：**同一套搜索骨架，跑两遍，各带一个代价泛型**（`Cost = HpLoss` / `Cost = Kills`），剪枝用同一 Pareto 支配（下）。

**剪枝支配关系（两查询共享，安全论证）**：结构键 `K₀`（§4.1）相同、`hp₁ ≥ hp₂ ∧ kills₁ ≤ kills₂` ⇒ `s₁` 支配 `s₂`——转移的**结构性读取面只依赖 K₀**（hp 只经 Refused 阈值 `subsystem/battle.rs:121` 单调影响可行性、kills 只被目标函数读，均不进转移的结构性分支），故 `hp₁ ≥ hp₂` ⇒ `s₁` 的后继可行域 ⊇ `s₂`（hp 更高不会新增 Refused），kills 更少在 Q2 目标下更优。该支配对两查询同时保序，故一次搜索维护 Pareto 前沿即可双查询取解。

---

## 2. 单层边界

### 2.1 入口与出口

- **入口 = `(6,11)`**：`scene_persist.rs:426-427` `else if floor == 1 && !went_down { place_player_at_cell(6, 11) }`（`swap_floor` :410-434 落点特例之一；同链另有 floor0→(1,2) :421、floor50→(6,7) :424、其余 `find_floor_gate` :435 踢下楼梯）。**census 实测（55 sprite 全扫，A 级，2026-11-10 对抗复核后更正）：全仓有 3 处楼梯 param&0xff==1 指向 floor1**——sprite0 type7 param=257(0x0101)@(1,1)、sprite2 type8 param=1@(1,1)、sprite54 type7 param=1@(7,1)（floor1 自身 type7 extra=0200 param=2 无误）；另 `change_floor(1,…)` 调用点= `scene.rs:2254`（菜单 case17 跳层，带 hp/atk/def 改写）与 `item_effect.rs:141` `ChangeFloorUp`（floor+1）。**入口特例只覆盖 `!went_down` 分支**（`scene_persist.rs:426`）：上行进 floor1（down=false：floor0/54 的 type7、菜单、上行符）→ (6,11)；**下行进 floor1**（sprite2 type8→1，went_down=true）→ `swap_floor` 走 `find_floor_gate(went_down)`（`scene_persist.rs:435`，`want=if up{7}else{8}`）→ 找 type7 → floor1 有 t7@(11,1) → `step_off_stairs` **落点 (11,1)=出口格本身**（旧主张「无 type8 ⇒ 不落点」作废，见文末勘误注）。solve-local 批2 钉入口=(6,11)（覆盖 `!went_down` 路径=判据面）；下行进入分支的换层语义归批3 层间边，单层 MVP 不跑（见 §6-3）。
- **出口集合 `Exit`**：
  1. **上行楼梯**：type7 @px352,py32=(11,1)，`param=2` → floor2（census + `scene.rs:4077-4082` change :4080）。这是 floor1 自身记录中**唯一**楼梯（floor1 census 无 type8；他层进入走 §2.1 特例与 §6-3）。
  2. **脚本换层边**：`StepGate` 触发的脚本内 MOV 踩梯/change_floor（§2.2）——归约为「已离开本层」的终止边，记录 `pending_floor`（`scene_persist.rs:386`）为目标层，**不展开跨层**（批3）。
- 单层 MVP **不区分**两种出口的「优劣」：`reachable` = 任一出口可达；`max_hp/min_kills` 在出口集合上取最优。

### 2.2 脚本触发如何进转移

floor1 三个 type83 门（census 实测）：`param=19 @（6,9）`、`param=66 @（6,9）`、`param=67 @（6,11）`（=入口格）。转移规则逐分支：

1. **触发条件**（直调 `scene.rs:927` `try_run_scene(idx, from_walk)`，gate 初值 `:931` `!script_line_flags[idx]`）：
   - case19 ∈ `8|9|11|12|19|30|41-45|48-50|52|53|55|56`（臂 `scene.rs:957-959`）⇒ `gate &= from_walk`（**踩格才触发**，出生不触发）；
   - case66 ⇒ `gate = false`（`scene.rs:1031`，**恒不触发**——census 记它但求解器可证明性剪除）；
   - case67 ⇒ `gate &= difficulty_index > 0`（`scene.rs:1032`）⇒ 钉 difficulty=0（§1.1#8）后**恒不触发**。
   ⇒ floor1 实际可达脚本只有 **slot19**；66/67 两门进状态（flags 位）但转移恒 ⊥，用红测试钉死（§3.1-T9）。
2. **执行**（过门后）：`scene.rs:4073` `load_level_script(param)` → 逐指令 `scene_solver_api.rs:59`，驱动复用 `script_abs.rs:191` `side_a::drive`（批1 已对拍过该面的 A/B 等价，A10 :457）。
3. **终态读回并回抽象态**：`side_a::drive` 返回的 `Obs`（`script_abs.rs:191`；`flags` 字段 `script_abs.rs:76`，`:208` 克隆 `script_line_flags` 灌入；另有 stats/px/py/entities）→ 并入 `s'`；difficulty/floor 用 `Snapshot::from_scene`（`script_abs.rs:118`，Obs 无此二字段）补读；`script_line_flags[idx]=true` 使门不可重入（幂等）。
4. **脚本内换层（floor1 特有边界）**：slot19 尾 `MOV_0_11_1 MOV_0_1_11`——MOV 把玩家**走上 (11,1) 梯格**触发 `change_floor(2,…)`，剩余指令在 floor2 续跑（`scene_persist.rs:374-379` BUG-026 更正注原话，Java mode11 续跑 fx-chang-e-floor1-taps T786）。**批2 归约**：harness 驱动中一旦 `persist.pending_floor` 被置（`:386`），即停驱、把该转移记为**终止边**（出口=脚本换层边，目标层 2），不执行 `swap_floor`、不展开续跑。丢弃的 floor2 续走状态（`MOV_0_1_11` 落点）= BR-1 风险面（§4.2），批3 层间边必须回收。

### 2.3 暗墙/隐藏门（probe）如何建模

- **事实**：floor1 census **无 type12（隐形墙）、无 type6（隐形路径）**——真层不覆盖暗墙。全仓 type12 仅 sprite23（24 枚，`docs/spec/gameplay.md:163` R-move-4 + `:334` R-scene-2），type6 见 sprite5/9/12/14/16/18/19/35/51/53/54（同注）。
- **模型（夹具内实现，真层批2 不跑）**：暗墙=实体的**两态机**——`type12`（隐形，视觉不显但碰撞阻挡）→ 被撞 `bump_reveal` 变 `type11`（`scene.rs:3551-3576`，`bump_reveal_target(12)` :3562；变形后仍阻挡，凿开需金勺子/玄明石，type11 直撞提示 `scene.rs:3540-3549`）；`type6` 隐形路径**踩上显形**（gameplay.md:163，a.java:3638-3646 注）⇒ 踩前=不可通行、踩后=通行。`probe_step`（`scene.rs:1553`）作为**主动试探动作**：不改变态只读可达性（BUG-038：场上非空闲拒绝）。
- **TDD 落点**：手工夹具（§3.1-T8）建 3×3+暗墙格，断言「撞前不可达/撞后可达、type12→type11 单次不可逆、type6 踩后显形」；**真层 floor1 断言无暗墙**（census 快照回归，防夹具漂真层）。
- **与计划 §5 R2 对齐**：R2（:409）对策「与 findPath 对拍（批2）」在 §3.1-T7 落地（暗墙改动 Walkability 的 has_entity_slot 面，恰是 findPath 语义分歧高发区）。

---

## 3. 验证钩子（TDD 先行）

### 3.1 红测试清单 ↔ 计划 §7 A 编号

批2 落码前先写的红测试（编号 T1-T10；每条=一个测试函数，先红后绿）：

| # | 红测试（先写，应失败） | 对应计划判据 | §7 A 归属 |
|---|------------------------|--------------|-----------|
| T1 | `solve_local` 存在性闸：`cargo test -p game-solver -- --list \| grep -c solve_local ≥ 1`（测试名含 `solve_local`） | :329 存在性闸（防空过） | 支撑 A8（每批门禁） |
| T2 | 3×3 手工夹具可达性：1 强怪挡路，**能杀 ⇒ reachable=true；杀不过 ⇒ Refused 且不计杀** | :334 TDD 起手式原文 | 批2 自身判据 |
| T3 | `max_hp`/`min_kills` 单调性断言（口径本提案钉死，见下） | :330「hp/kills 单调性断言绿」 | 预演 A1/A2（正式归批4） |
| T4 | 手工夹具有钥开门/无钥阻挡（`consume_key_for_door` 直调进转移；钥匙数变化断言） | :320「开门（consume_key_for_door）」 | 批2 自身判据 |
| T5 | floor1 census 快照：13×13、type7×1@(11,1)→2、type1×7、type26×5、type15×1、type83×3{19,66,67}、type12/type6=0——**断言挂 solve_local 的 fixture 装载路径**（未实现⇒红；直调 census.rs 既有断言=写时即绿，计划 :301/:314 B 类不得当红） | :327 `--floor 1` fixture 前置 | 防夹具漂（A9 同族） |
| T6 | floor1 端到端：`solve-local --floor 1` 输出 JSON 形状 `{"floor":1,"reachable":true,"max_hp":<int>,"min_kills":<int>,"confidence":"bounded-best","route":[…]}`（逐字段类型断言，防空过）+ **`min_kills ≥ 3` 断言**（§3.2 实测下界） | :327-328 预期输出（形状） | 预演 A1/A2 形状（正式归批4） |
| T7 | **findPath 对拍**：同层同起终点，`find_path` 的 found ⇔ solver BFS 的 found（floor1 全格对 + 暗墙夹具格对） | §5 R2 :409 对策「与 findPath 对拍（批2）」 | 批2 强制（R2 落地） |
| T8 | 暗墙夹具：type12 撞前不可达/撞后 type11 仍阻挡/凿开语义不在批2（断言 type 不变性）；type6 踩后显形；**+型69..75 胜利 A 类换层手工夹具**（计划 :320 范围，floor1 真层无此类怪、真层不跑）——断言经 solve_local 夹具求解路径触发（直调既有测试只作回归护栏，计划 :301） | §2.3 建模 + R2；:320 范围 | 批2 夹具面 |
| T9 | type83 三分支：case19 踩格触发（from_walk=true 才跑、flags 置位后不可重入）、case66 恒不触发、case67 difficulty=0 不触发——**断言经 solve_local StepGate 转移层**（未实现⇒红，计划 :314） | :320「层内脚本触发」 | 批2 自身判据 |
| T10 | **确定性**：同输入双跑，`route/max_hp/min_kills` 逐字节相等（测试内双跑 + 判据命令双跑 `diff` 两层） | A5 :452 形状（批2 起步实现） | 预演 A5（正式归批4） |

**T3 单调性口径（计划只写「单调性」三字，本提案钉死具体断言，待复核确认）**：
(i) `reachable=false ⇒ max_hp=min_kills=None`；(ii) `reachable=true ⇒ max_hp ≤ hp(s₀)`（无治疗子集）；(iii) `min_kills ≤ kills(route_max_hp)`。（旧 (iv)「杀数上限 k 参数化单调性」依赖批2 未承诺的 k 参数化，已按复核黄项删除；若批3 引入 k 参数化再补。）

**§7 A 编号归属总表（与计划逐条对齐）**：

| A | 计划归属 | 批2 动作 |
|---|----------|----------|
| A7 金样对拍 | 每批（:454） | 每 commit 跑，预期恒绿（批2 不改 game-core） |
| A8 门禁总闸 | 每批（:455） | 每 commit `gates check` FAIL=0（判据 :331） |
| A9 纯度/无环 | 每批（:456） | `solve_local` 必须落 **lib**（grep 闸排除 `bin/solver.rs`，新子命令壳才在 bin）；`cargo tree -i game-solver` 仅自身 |
| A10 分类对拍 | 批1（:457，已过） | 批2 不回退（跑一遍防回归） |
| A1-A4、A5 | **批4**（正式） | 批2 只出**单层版**形状（T6）与确定性起步（T10），JSON 字段名与 A1/A2 样例（:463-469）**预先对齐**，批4 无缝换全局 |
| A6 oracle 回放、A11 五元组判别器 | **批5**（:453/:458） | 批2 不做；但 T6 的 route 必须能喂 `route_gen.py`（A6 前置） |

### 3.2 fixture 选层理由（floor1）

1. **判据硬编码**：计划 :327 `solve-local --floor 1`——改层=改计划，本提案不改计划 ⇒ fixture 必须 floor1。
2. **真地形先例**：`walk_replay_floor1.rs` 已用真实 maplv1+sprite1 跑通 13×13 可通行面（计划 §2.8 :185 引），批2 可直接复用同一 fixture 源。
3. **真格 13×13**：批0 b38b694 考证（maplv1 头 26×26 raw→>>1），规模小到状态空间可穷举论证（§4.1）。
4. **动作覆盖好**：单出口上行无歧义（(11,1) param→2）；**7 黄门+5 黄钥匙**覆盖开门经济（耗钥/无钥双侧可断言）；11 只弱怪（type41-46）覆盖战斗闭式双侧（能杀/杀不过由 T2 夹具补足）；**3 个 type83 覆盖 try_run_scene 三分支**（T9）；**脚本19 换层边覆盖「脚本触发离开本层」边界**（§2.2.4）；QoL 件筋斗云 type15×1 恰在本层（QoL 位有真值可断言）。
5. **必经点已裁定（2026-11-10 A 级实测）**：**存在必经点，min_kills ≥ 3**——避敌（11 敌格全避、门全开）可达区仅 39 格、出口 (11,1) 不在区内；0-1 BFS min crossings=3，最优路必穿 (7,1)type41、(8,1)type42、(9,1)type41（门+怪全可通行时可达 78 格含出口；`walk_replay_floor1.rs:38`「怪物格阻挡」注佐证）。T6 断言 `min_kills ≥ 3`。
6. **短板显式声明**：floor1 无 type12/type6 暗墙、无商店——由 T8 夹具、批3 承接，**不假装 floor1 覆盖**。

### 3.3 确定性怎么钉（同输入两次同解）

1. **纯度闸（结构保证）**：A9 grep 闸（`:456`）——`solve_local` 不引 `std::fs/net/process/time`；不引 GameRng/Clock（搜索本身零随机）。
2. **遍历序钉死（结构保证）**：方向序 = 写死常量 `const DIRS: [Dir;4]`（顺序与 `route_gen.py:29` `KEY` 表同序，路线可复现）；实体遍历按槽位升序（`for e in 0..entities.count` 风格，对齐 `scene_persist.rs:437`）；**禁 HashMap 迭代序**——去重表用 `BTreeMap/Vec` 排序键（计划 :343 已在批3 注明同约束，批2 起执行）。
3. **tie-break 钉死**：`argmax_hp` 并列取步数少者、再并列取字典序最小路线；`argmin_kills` 同理——写进测试断言（T10），不留给实现者即兴。
4. **双跑验证（验收）**：测试内双跑（T10）+ 判据命令层 `solve-local --floor 1` 跑两次 `diff`（A5 :452 的单层预演）。
5. **输出确定性**：JSON 字段序/数值类型固定（T6 形状断言），杜绝「同解不同文」。

---

## 4. 风险与回退

### 4.1 计划 §5 已有风险的批2 承接

| 计划风险 | 批2 承接动作 |
|----------|--------------|
| **R1 状态爆炸**（:408） | 单层规模论证 + 去重键 + Pareto 支配（见下「去重键设计」）；爆了的回退=分两段（先可达性 BFS，再在可达子图上做最优搜索） |
| **R2 隐藏墙**（:409，对策明写「与 findPath 对拍（批2）」） | T7 对拍 + T8 暗墙夹具；floor1 真层无暗墙（不假装覆盖） |
| **R3 剧情强制 ROS_2/DES 可达性约束**（:410 计划原文；旧引「三查询目标打架」系误标，计划全文 grep「目标打架」=0） | floor1 slot19 含 TAK/DES 指令：已由批1 `side_a::drive`（`script_abs.rs:191`）直调覆盖并 A/B 对拍（A10 :457），批2 脚本转移全走 drive，无额外动作。**「双查询目标合不合并」=本提案自列非计划 R**：§1.4 不合目标、共享 Pareto 支配；字典序/Pareto 口径留批3 报审（§3.3 :257 原文） |
| **R7 ScriptOp parse/execute 私有**（:414，facade 提案已解） | 批2 消费面=既有 3 fn（`scene_solver_api.rs:45/52/59`），**不开新暴露面**；若 T9 暴露缺口，走 facade 提案「另立提案」条款，不混批2 |
| **R10 new_game_stats 无调用方**（:417，对策=S0 常量写死注证据） | **入口状态钉死**：s₀ = `state.rs:21-22` `s0()` = (hp 300, atk 10, def 10, floor 51)（真源 a.java:3482-3491 step8；keys/gold=0 由 ingest 侧，`state.rs:19` 注）；solve-local 输出 `s0` 字段注证据，§1.4 max_hp 基线 hp(s₀)=300 |
| **R11 筋斗云独立换层通道**（:418） | 批2 **无此边**（UseItem 裁出，§1.2）：type15 只记录捡拾入 qol 位、不激活 ⇒ 批2 层间终止边仅楼梯/脚本两类；**筋斗云换层边明确归批3**（DAG 显式边+区间门 4 分量） |
| **金样/门禁**（§0 :23） | 每 commit A7+A8；批2 不改 game-core ⇒ 金样恒绿是**回滚哨**（不绿=说明误改了 core，先查 diff 再继续） |

**去重键设计（批2 具体化，喂给批3）**：

```
K₀(s) = (pos, ents_digest, flags, keys, bag, atk, def)   # 结构键
支配维度 = (hp, kills)                                     # 不进结构键，只进支配
  - ents_digest：门开集+道具在否+怪存活 的槽位 bitset（层内单调：门只开不关、怪只死不活、
    道具只捡不放 ⇒ 后缀行为只依赖「已发生集合」，单调性使状态图无环化于该维度）
  - flags：script_line_flags 位图（单层 3 门 ⇒ 2³）
  - atk/def：**精确整数**（§1.1 批2 不离散化），进 K₀ 因闭式（combat.rs:106）读它们
  - hp/kills：精确整数，**不进 K₀**——hp 只经 Refused 阈值（subsystem/battle.rs:121）单调
    影响可行性、kills 只被目标函数读，均不进转移的结构性分支 ⇒ 不参与等价合并、只参与支配
  - gold 不进 K₀（floor1 无商店，论证见 §1.1）
等价合并：K₀ ∧ hp ∧ kills 全同 ⇒ 同一状态（精确合并无损——转移读取面 ⊆ K₀ ∪ {hp,kills}，
  hp/kills 不同的状态后继拓扑相同、只差阈值/代价，故结构等价必须连 hp/kills 一起比）
支配剪枝：K₀ 相同且 hp₁≥hp₂ ∧ kills₁≤kills₂ ⇒ 丢 s₂（§1.4 安全论证）
```

**规模上界粗算（floor1）**：169 格 × 门开集 2⁷ × flags 2³ × 钥匙 (0..5)³ 有界 × kills 0..11 × hp 带 [1,300]——裸上界 ~10⁹ 但被三重压缩：(a) ents 单调使门开集与 pos/kills 强相关（钥匙数=已开门数的函数上界）；(b) Pareto 支配把 hp×kills 双维压到前沿（每 K₀ 段至多 O(战斗次数) 个非支配点）；(c) 13×13 可达子图远小于 169（墙内）。**若仍爆**：回退顺序=①去掉非必需读回面 ②分两段搜索 ③（最后手段）把单层再切「可达性/最优」两个 commit 分别验收——不放宽正确性断言。

### 4.2 批2 特有风险（BR）

| # | 风险 | 证据锚 | 回退 |
|---|------|--------|------|
| BR-1 | **脚本19 跨层外溢**：归约终止边会丢 floor2 续跑状态（`MOV_0_1_11` 落点），若批3 不回收则全局解错 | `scene_persist.rs:374-379`（BUG-026 注，Java mode11 续跑 T786） | 批2 只记 `pending_floor` 边；本条写入批3 派工单的必答项（待复核 §6-6） |
| BR-2 | **BattleCtx 装配分叉**：求解器自备写槽的装配语义（`battle_enemy_hp` 分源等）与真引擎逐位不一致 ⇒ Refused 结论分叉 | `subsystem/battle.rs:59-75`、`:127+` 装配注 | T2 夹具双侧断言（求解器 ctx vs harness `resolve_bump` 同输入同结论）；A11 判别器（批5 :458）终审 |
| BR-3 | **find_path 非最短/有怪癖**（扩展上限 100、cost=曼哈顿和）：拿它当最短路会静默错 | `pathfind.rs:1-16` 头注、`:55` | 可达性=find_path、最短=BFS 自算，**只对拍 found 不对拍路径**（T7）；路径正确性由 T6 真层回放语义钉 |
| BR-4 | **difficulty 门**：case67 在非默认难度下触发，钉 difficulty=0 若与批4 query 的实际难度面不符 ⇒ 漏转移 | `scene.rs:1032`；默认 0 `scene_persist.rs:133` | 批4 接 query 前复核菜单改值路径【待复核 §6-4】；solve-local 输出带 `difficulty` 字段占位 |
| BR-5 | **闭式与引擎分叉（DC-9）**：`predictHpLossVsType` 桥已删（缺 23/24 护符二倍加成）。**2026-11-10 实测：分叉面在 floor1 不存在**——`trait_flags_for`（`combat.rs:42-49`）= `49\|53\|74⇒1, 69⇒2, _⇒0`，floor1 怪 41..46 全部 trait_flags=0；「含 23/24」系护符**道具 id**（TRAIT1/TRAIT2_AMULET=23/24，`combat.rs:162-165`）非 trait_flags | `combat.rs:133`（DC-9 注起）、`:42-49`、`:162-165` | 批2 无需动作；**批4 全图重测** 49/53/74/69 型怪（命中 → 该类怪改走 harness `resolve_bump` 直读结论，登记 finding） |
| BR-6 | **RouteEngage 裁出破功（必经点）**：findPath「实体槽可通行」把怪格当可走 ⇒ 可达性虚高。**2026-11-10 实测：必经点存在**（避敌区 39 格不含出口、min crossings=3）⇒ 该风险 concrete | `scene.rs:2046` 途中面；`pathfind.rs:11` 可通行=terrain‖实体槽；避敌区/0-1 BFS 实测（§3.2） | T6 断言 `min_kills ≥ 3` + 路线中路过怪格必须建模为 BumpEnemy 动作（不许白走，T2 夹具同族）；min_kills=3 上界在 key 经济（7 门 5 钥）下是否可达 = T6 真层输出核对 |
| BR-7 | **UseItem 裁出 ⇒ max_hp 非全局最优** | §1.2 裁剪声明 | 输出 `confidence=bounded-best` 不冒充 optimal（A1 :448 三态自洽）；复核若驳 → 加 C6b 切片（+0.5d） |

---

## 5. 切片与估时（批2 内部小步 commit，TDD 红→绿）

计划批2 估时 2–3d（:316）。每步=一个 commit，先红后绿，验收判据可复制：

| 切片 | 内容 | 红（先写）→ 绿（实现） | 验收判据 | 估时 |
|------|------|------------------------|----------|------|
| **C1** | `solve_local.rs` 骨架 + bin 子命令壳（壳在 bin、逻辑在 lib） | T1 存在性闸红 → 挂 `solve-local --floor 1` 输出占位 JSON | `cargo test -p game-solver -- --list \| grep -c solve_local ≥1`；`gates check` FAIL=0 | 0.25d |
| **C2** | 手工 3×3 夹具 + 可达性（find_path 直调 + BFS 自算）+ 战斗 Refused（BattleCtx 直调 resolve_bump） | T2 红（能杀/杀不过两例）→ 绿 | `cargo test -p game-solver solve_local` T2 绿；A9 纯度 grep 0 命中 | 0.5d |
| **C3** | 目标函数：max_hp / min_kills 双查询（同骨架两代价泛型 + Pareto 支配） | T3 单调性红 → 绿 | T3 三断言 (i)-(iii) 绿；并列 tie-break 有断言 | 0.5d |
| **C4** | 开门转移（`consume_key_for_door` 直调）+ 钥匙经济 | T4 红（有钥开/无钥挡）→ 绿 | T4 绿；T5 census 快照绿（floor1 7 门 5 钥进 fixture） | 0.25d |
| **C5** | 脚本触发转移（门面 execute_script + side_a drive + Snapshot 读回 + pending_floor 终止边） | T9 红（三分支）→ 绿 | T9 绿；slot19 触发→终止边断言；66/67 恒不触发断言 | 0.5d |
| **C6** | floor1 端到端 + route 序列化（route_gen 航点语法）+ 确定性双跑 | T6/T10 红 → 绿 | 计划判据 :327-328 四条命令全绿（含 `gates check` :331）；route 逐航点过 `route_gen.py` 语法（`route_gen.py:12-15`） | 0.5d |
| **C7** | findPath 对拍 + 暗墙夹具 + 回归收尾 | T7/T8 红 → 绿 | T7/T8 绿；A7 金样绿、A8 门禁绿、A9 纯度绿、A10 对拍不回退 | 0.25d |

**总计 ≈ 3.25d**（计划 2–3d 的上沿；若 C6 的 UseItem 争议触发 BR-7 回退，+C6b 0.5d）。每 commit message 按 AGENTS.md §9：`feat(solver): 批2 C<n> …`，含证据行号与门禁结果；**不触 `data/naming/`、`reference/`、`original/` ⇒ 不需 Evidence 字段，但正文仍写实测锚**（本仓习惯）。全部切片过对抗复核前不推送（任务纪律）。

---

## 6. 待复核攻击面（复核者会验什么）

1. **census 标签冲突**：计划 :92 写「type12 门=24」，而实测门=type1/2/3（`scene.rs:3386`）、type12=隐形墙（gameplay.md:163，全仓仅 sprite23）——复核者应裁定是计划勘误还是另有 type12 门语义；本提案按 scene.rs 实测走。
2. **scene.rs 全部行号**：本提案 scene.rs 锚（927/1553/2046/2254/3315/3386/3402/3540/3551-3576/3593/3987/4069/4073/4077-4086/4094/4221/4495）均为 HEAD `b976803` 实测；换 HEAD 必漂（计划内旧锚已 +1）。复核者应抽 6 个 sed 复测。
3. **floor1 入口路径（2026-11-10 更正）**：**旧主张「全仓无楼梯指向 floor1」已证伪**——3 反例实测（sprite0 type7 param=257、sprite2 type8 param=1、sprite54 type7 param=1，全 55 sprite 扫描，见 §2.1 与文末勘误注）。下行进入（went_down=true）**也不再主张「不落点」**：`find_floor_gate`（`scene_persist.rs:435`）反读 type7 → 落 (11,1)=出口格。遗留待复核=**下行入口 (11,1) 是否需批2 覆盖**（本提案裁定不覆盖：入口=(6,11) 只覆盖 `!went_down`，下行分支归批3）——该裁定是否削弱批2 判据面，复核裁定。
4. **difficulty 默认值**：`scene_persist.rs:133` 新局=0 是代码语境（B 级）；菜单改值路径与批4 query 实际难度面未实测——钉 difficulty=0 的前提要复核。
5. **BattleCtx 装配等价**：求解器自备 10 写槽 + `battle_enemy_hp` 分源装配（`subsystem/battle.rs:59-75/:127+`）与 `scene.rs` 真引擎装配路径是否逐位一致，只靠 T2 夹具覆盖是否足够。
6. **script19 终止边归约**：丢弃 floor2 续跑状态（`MOV_0_1_11` 落点）是否可被批3 回收——BUG-026 注（`scene_persist.rs:374-379`）只证 Java 续跑存在，未证求解器归约无损。
7. **DC-9 分叉面（2026-11-10 floor1 面关闭）**：实测 `trait_flags_for`（`combat.rs:42-49` = 49\|53\|74⇒1、69⇒2、_⇒0）⇒ floor1 怪 41..46 全 0，批2 闭式分叉面不存在（BR-5 已更正；「23/24」系护符道具 id 非 trait_flags，`combat.rs:162-165`）；DC-9 注现锚 `combat.rs:133`；遗留=**批4 全图重测** 49/53/74/69 型怪。
8. **RouteEngage 必经点（2026-11-10 存在性关闭）**：**已实测存在**——避敌区 39 格不含出口、0-1 BFS min crossings=3（§3.2）；遗留待复核=T6 断言 `min_kills ≥ 3` 是否足以拦住 findPath 可达性虚高（白走怪格），及 key 经济（7 门 5 钥）下 min_kills=3 的上界可达性（T6 真层输出核对）。
9. **A 编号归属**：本提案把 A1-A5/A10 正式验收留在批4/批1、A6/A11 留在批5，批2 只背 A7/A8/A9 + 计划 :326-332 自身判据——与计划 §7 表逐行核对是否有越领/漏领。
10. **route_gen 语法兼容的验收强度**：计划 :322 只说「兼容语法」，本提案 T6 落成「route 逐航点过 route_gen 解析」——是否要把 `route_gen.py` 实跑（含其 BFS 填充三重校验，`route_gen.py:6-7` floor8/14 教训）纳入批2 CI 判据，计划未明说，待裁定。

---

## 7. 证据指针表

| # | 锚（HEAD `b976803` 实测） | 证明什么 |
|---|---------------------------|----------|
| E1 | `docs/superpowers/plans/2026-11-09-solver-implementation-plan.md:316-335` | 批2 范围权威（内容+判据+TDD 起手式） |
| E2 | 同文件 `:444-469` | §7 验收总表 A1-A11 归属 |
| E3 | 同文件 `:404-419` | §5 风险 R1-R12（R2 :409 含批2 对拍对策） |
| E4 | 同文件 `:240-258` | §3.3 SolverState 概念形状与保真红线（hp 禁自算） |
| E5 | `crates/game-core/src/combat.rs:106` | `predict_battle_hp_loss` 闭式 |
| E6 | `crates/game-core/src/combat.rs:170` | `consume_key_for_door` 开门直调面 |
| E7 | `crates/game-core/src/combat.rs:51-63` | `PlayerCombat` 全 pub（状态字段源） |
| E8 | `crates/game-core/src/subsystem/battle.rs:59-75,85,110,121` | BattleCtx/pub、predict_engagement、resolve_bump、Refused 判据 |
| E9 | `crates/game-core/src/pathfind.rs:21,55` | Walkability/find_path 寻路直调面 |
| E10 | `crates/game-core/src/scene_solver_api.rs:45,52,59` | 批1 门面 3 fn（脚本直调面） |
| E11 | `crates/game-solver/src/script_abs.rs:76,118,191,208` | Obs.flags 读回（:76/:208）+ Snapshot 补读 difficulty/floor（:118）+ side_a drive（:191） |
| E12 | `crates/game-core/src/scene.rs:927,931` | try_run_scene 与 gate 初值（脚本门） |
| E13 | `crates/game-core/src/scene.rs:1031,1032` | case66 恒 false / case67 difficulty 门 |
| E14 | `crates/game-core/src/scene.rs:4069,4073` | 踩 type83 → 过门才载脚本 |
| E15 | `crates/game-core/src/scene.rs:4077-4088` | 踩梯 change_floor(param&0xff) |
| E16 | `crates/game-core/src/scene.rs:3386-3404` | 门=type1/2/3、耗钥、移除 |
| E17 | `crates/game-core/src/scene.rs:3593` | 撞怪 BumpEngage 直调 resolve_bump |
| E18 | `crates/game-core/src/scene.rs:1553,3551-3576` | probe_step / type12→11 暗墙 |
| E19 | `crates/game-core/src/scene_persist.rs:349,386,410-443` | change_floor/pending_floor/swap_floor 落点特例（(6,11) :426-427） |
| E20 | `crates/game-core/src/scene_persist.rs:374-379` | BUG-026：script19 脚本行走踩梯 mode11 续跑 |
| E21 | `docs/spec/gameplay.md:163,168,334` | type6/type12 暗墙语义与 23 层独占 |
| E22 | `tools/route_gen.py:12-15,29` | 航点语法与方向键表 |
| E23 | `crates/game-solver/src/state.rs`（s0、Q3_EQUIP_SET） | 批0 裁定常量 |
| E24 | `crates/game-core/src/lib.rs:31-33` | scene_test_support 夹具导出（harness 依据） |

---

## 8. 修订勘误注（2026-11-10 对抗复核后原地更正）

> 依据 `docs/findings/reviews/2026-11-10-solver-batch2-proposal-review.md`（红6·黄项）修订；按 AGENTS.md §3 证伪更正文化，旧主张在此留痕、不删历史：

1. **scene_persist 行号漂移（红6-R6）**：旧引 → 现值（本批 sed 实测）：BUG-026 注 `:371-378` → **`:374-379`**；`swap_floor` `:413-437` → **`:410-434`**；`find_floor_gate` `:438-446` → **`:435-443`**；floor0 落点 `:423` → **`:421`**；floor50 落点 `:425` → **`:424`**；（floor1 落点 `:426-427` 复测无误）。E19 范围同步 `:413-446` → `:410-443`。
2. **combat DC-9 锚（红6-R6）**：`combat.rs:131+` → **`:133`**（DC-9 注起始行实测）。
3. **入口 census 证伪（红6-R4）**：旧主张「全仓无任何楼梯 param&0xff==1 指向 floor1」→ 现主张=**3 反例**（sprite0 type7 param=257@(1,1)、sprite2 type8 param=1@(1,1)、sprite54 type7 param=1@(7,1)，全 55 sprite 实测）；旧「下行进入 floor1 无 type8 ⇒ `find_floor_gate` 不落点」→ 现主张=`find_floor_gate`（`:435`）反读 type7 → 落 (11,1)。正文 §2.1/§6-3 已同步。
4. **风险表错标（红6-R5）**：旧「R3 三查询目标打架(:410)」→ 计划 :410 R3 实为**剧情强制 ROS_2/DES 可达性约束**（「目标打架」计划全文 grep 0 命中）；旧「R7/R10 门面暴露」→ 拆为 R7(:414) / R10(:417) 两行，R10=new_game_stats 无调用方（S0 写死对策已承接，§1.4 基线 hp(s₀)=300）；旧「§3.3 :256」→ **`:257`**。
5. **Snapshot/flags（红6-R2）**：`Snapshot::from_scene`（`script_abs.rs:95` `pub struct Snapshot` 起，struct 至 :115）**无 flags 字段**——终态 flags 读回改经 `side_a::drive` 返回的 Obs（`:76` 字段、`:191` drive、`:208` 克隆灌入），Snapshot 只补 difficulty/floor（`:118`）。
6. **拾取 pub（红6-R3）**：`pickup_item_type`（scene.rs:4221）为**私有** fn——红线表改经 pub `try_step`（:3315）落格触发（apply_step_cell_effects :3987 内 :4094），不列不可达面。
7. **形式化（红6-R1）**：去重键由 `K`（含 hp/kills，致支配恒等永不可触发）改为 **`K₀=(pos,ents_digest,flags,keys,bag,atk,def)` + (hp,kills) 支配维度**；等价合并/支配剪枝/§1.4 措辞同步（转移读取面 ⊆ K₀∪{hp,kills}，hp/kills 只参与支配）。
8. **黄项 Y1-Y8 全部落地**：型69..75 不裁出（改手工夹具 T8 扩展）、T5/T8/T9 绑 solve_local 产物层防写时即绿（计划 :301/:314）、删 T3(iv)（C3 改三断言）、BR-5 实测关闭（floor1 分叉面不存在）、必经点 min_kills≥3 实测入 T6、R11 筋斗云换层边显式归批3、T6 JSON 加 `confidence:"bounded-best"`（对齐 A1 样例 :466）、qol 派生论证补入 §1.1。**复核报告标题写「黄9」但正文只列 Y1-Y8 共 8 条（无 Y9），此处按 8 条处置。**
