# 对抗复核报告（二轮）：solver-batch2 提案修订验证

- **对象**：`docs/superpowers/proposals/2026-11-10-solver-batch2-proposal.md`（310 行，`eace26f`，父 `78a3bea`，分支 modern，+63/-41）
- **判据**：一轮报告 `docs/findings/reviews/2026-11-10-solver-batch2-proposal-review.md`（红6·黄9·绿8 → 需修订）原文
- **方法**：sed/grep 行号实测、python 全量 census 与 0-1 BFS 复算、`find_path` 逐行复刻仿真、git diff 逐段核对、门禁只读重跑
- **范围**：只写本文件；未改任何代码/提案/计划

---

## 总判：**红 1 · 黄 1 · 绿 21 → 需再修（回派）**

| 类 | 条目 |
|---|---|
| 红 1 | **R2-1（新红，Y5 修订引入）**：`min_kills ≥ 3` 断言与提案自身出口集合/脚本换层边归约矛盾（详见 §三） |
| 黄 1 | **Y2-1（新黄）**：C5 行未同步 R2 的 Obs 读回口径（详见 §四） |
| 绿 21 | 红6 逐条复测 6 + 黄8 处置抽查 8 + 绿面抽复测 3 + 新行号汇总 1 + 夹带 1 + 闸 2 |

---

## 一、红6 逐条复测（全绿，6/6）

### R1（绿）去重键 K₀ + 支配维度
- grep `K=(pos` 全文仅 §8 勘误注（:309，允许的留痕引用）1 处；旧表述「K 相同」「K 含 hp/kills 做支配丢弃」残留 **0**。
- 三处一致实测：§1.4 :100（结构键 K₀ + `hp₁≥hp₂∧kills₁≤kills₂` 支配）、代码块 :205-206（`K₀(s) = (pos, ents_digest, flags, keys, bag, atk, def)`，支配维度 `(hp,kills)`）、规模粗算 :219（「每 K₀ 段」）。
- 等价合并（:214，K₀∧hp∧kills 全同 ⇒ 同一状态）与支配剪枝（:216，K₀ 相同 + hp/kills Pareto ⇒ 丢 s₂）两概念区分正确。

### R2（绿）终态 flags 改经 drive 返回 Obs
- 实测 `script_abs.rs`：Obs `flags: Vec<bool>` 字段 `:76`；`pub fn side_a::drive` `:191`；`:208` `flags: sc.script_line_flags.clone()` 灌入；`Snapshot` struct `:95-115` 无 flags 字段，`from_scene` `:118` 只补 difficulty/floor。
- 提案三处已同步：§1.3 :77、§2.2.3 :124、E11 :282——均改「drive 返回 Obs 读回 flags，Snapshot 只补 difficulty/floor」，不再声称 Snapshot 含 flags。（C5 行漏同步见 §四 黄 Y2-1。）

### R3（绿）红线表改经 pub try_step 拾取
- 实测 `scene.rs`：`pub fn try_step` `:3315`；`fn apply_step_cell_effects` `:3987`（私有，仅作内部路径引用）；`:4094` `self.pickup_item_type(t)`；`fn pickup_item_type` `:4221` 私有。
- 提案三处（:31/:54/:79）+ §8.6（:308）全改「经 `pub try_step` → `apply_step_cell_effects:3987` 内 `:4094` 拾取，`pickup_item_type` 私有不直调」。

### R4（绿）入口 census 3 反例 + find_floor_gate 反读 + !went_down 边界
- python 全 55 sprite census 独立复算（extra_len 表复刻，assert pos==len 全过）：楼梯指向 floor1 恰 3 处 = sprite0 type7 param=257@(1,1)、sprite2 type8 param=1@(1,1)、sprite54 type7 param=1@(7,1)；floor1 自身 type7 extra=0200 param=2@(11,1)——提案 §2.1 :108 逐字一致。
- `scene_persist.rs:435-443` sed 实测：`fn find_floor_gate(&self, up: bool)` `:435`、`let want = if up { 7 } else { 8 }` `:436`——下行（want=8）反读 type7 → floor1 的 t7@(11,1) → 落点 (11,1)（旧「无 type8 不落点」已作废并在勘误注留痕）。
- 「入口特例只覆盖 `!went_down`」（`scene_persist.rs:426`）sed 实测在位；下行进入分支归批3 的边界声明（§2.1 尾 + §6-3）正确。

### R5（绿）风险表 R3/R7/R10 拆行 + :256→:257
- 计划 `:410` sed 实测 = R3「剧情强制 ROS_2/DES 可达性约束」（旧「目标打架」计划全文 grep **0** 命中）；`:414` = R7 ScriptOp parse/execute 私有；`:417` = R10 new_game_stats 无调用方。提案拆行后 :197/:198 与计划逐字对齐，R10 承接 `state.rs:21-22` `s0() = (300,10,10,51)` sed 实测 ✓。
- `:256→:257`：正文 :97/:196 均用 :257，:256 仅存 §8 勘误注引用（允许）。

### R6（绿）scene_persist 5 组行号 + combat DC-9 + 勘误注留痕
- sed 逐个实测：BUG-026 注 `:374-379`、`swap_floor` `:410-434`、`find_floor_gate` `:435-443`、floor0 落点 `:421`、floor50 落点 `:424`、floor1 落点 `:426-427`；`combat.rs:133` DC-9 注起始行 ✓。
- §8 勘误注 **8 条在位**（:299-310），每条旧→新原地留痕（AGENTS.md §3 证伪更正文化）；勘误注 8 关于「一轮标题写黄9、正文只列 Y1-Y8（无 Y9）」的说明**实测准确**（一轮报告 grep `Y9` = 0 命中）。

---

## 二、黄8 处置抽查（全绿，8/8）

| # | 处置 | 实测证据 |
|---|---|---|
| Y1 | 型69..75 不裁出 + T8 扩展 | 计划 `:320` sed 实测原文「层内脚本触发（type83/型69..75 胜利触发的 A 类转移）」；提案 :61 改「⚠不裁出计划范围（计划:320 明列）：手工夹具覆盖（T8 扩展）」，T8(:151) 加「型69..75 胜利 A 类换层手工夹具」✓ |
| Y2 | T5/T8/T9 防写时即绿 | T5(:148) 加「断言挂 solve_local fixture 装载路径（未实现⇒红，计划 :301/:314）」、T9(:152) 加「断言经 solve_local StepGate 转移层（计划 :314）」；计划 :301/:314 sed 实测均为「B 类不得当红/不作红测试」原文 ✓ |
| Y3 | T3(iv) 删除 | T3 三断言 (i)-(iii)（:156），(iv) 已删并注明「依赖批2 未承诺的 k 参数化，若批3 引入再补」；C3(:243) 改「三断言 (i)-(iii)」✓ |
| Y4 | trait_flags 分叉面关闭 | `combat.rs:42-49` 实测 `trait_flags_for` match `49|53|74⇒1, 69⇒2, _⇒0`；护符道具 id `:162-165`（TRAIT1=23/TRAIT2=24）；floor1 敌 types{41,42,43,44,45,46} 全不在触发集 ⇒ 全 0，DC-9 分叉面在 floor1 不存在——论证成立 ✓ |
| Y5 | 必经点翻转为 min_kills≥3 | 处置**落地**（§3.2-5 :175 新增 A 级实测条目 + T6 断言 + BR-6/§4.2-8 同步），0-1 BFS=3、避敌区 39 格、全可通行 78 格数字独立复算全对——**但该断言本身有新红，见 §三** |
| Y6 | R11 筋斗云归批3 | 计划 `:418` sed 实测 = R11 筋斗云独立换层通道（批3/批5）；提案 §1.2 :64 尾 + §4.1 新增 R11 行显式声明 ✓ |
| Y7 | confidence 对齐 | 计划 `:466` sed 实测样例含 `"confidence":"bounded-best"`；T6(:149) JSON 形状已加该字段 ✓ |
| Y8 | qol 冗余性论证 | §1.1 新增「qol=筋斗云在 bag ⇒ 可由 bag 派生」论证段 ✓ |

---

## 三、新红 R2-1（Y5 修订引入）：`min_kills ≥ 3` 与提案自身出口集合矛盾

**结论**：T6(:149)、§3.2-5(:175)、BR-6(:230)、§4.2-8(:262) 的 `min_kills ≥ 3` 断言为**假**——按提案自己的形式化，floor1 存在 **0 杀合法路** 经脚本换层边离开本层；真层实现正确时 T6 必红（这不是「未实现红」，是断言与事实矛盾）。

### 矛盾链（全部 A 级实测/文本实测）

1. **量化范围含脚本边**：§1.4 :88 合法性 = 每步 `T≠⊥` ∧ 终态 `sₙ.pos ∈ Exit`（§2.1 出口集合）；:91 `kills(π)` **只数** `aᵢ = BumpEnemy` 的求解器动作；:112 明写「`max_hp/min_kills` 在出口集合上取最优」。§2.1 Exit#2（:111）= 「StepGate 触发的脚本内 MOV 踩梯/change_floor——归约为『已离开本层』的终止边，记录 `pending_floor`」。⇒ min_kills 的取小**包含**经 Exit#2 的路。
2. **0 杀路径 Path B（逐步实测）**：
   - 钥匙 (5,10) type26 = 黄钥（`effect_registry.rs:122` `(26,"Key",{yellow:true,…})`），0-1 BFS dist=0（0 杀可达，途中无门无敌）；door (6,9) type1 = YELLOW（`scene.rs:3397`）耗黄钥（`combat.rs:154/:173`）——色兼容，开门不需杀。
   - 开门进 (6,9) → 落格 arrival（from_walk=true）→ `try_run_scene(19,true)`：`gate = !script_line_flags[19]` 初真（`scene.rs:927-931`）+ case19 `gate &= from_walk` → 载 slot19。
   - **find_path 复刻仿真**（`/tmp/r2_findpath.py`，`pathfind.rs:57-152` 逐行复刻含幻影头/插入怪癖/100 扩展上限；walkability = terrain‖实体槽，与 `scene.rs:1181-1190` 构造同款）：slot19 玩家 4 腿全部 DONE、**零门阻**：
     - `MOV_0_2_8` (6,9)→(2,8) = [(6,9),(6,8),(5,8),(4,8),(3,8),(2,8)]
     - `MOV_0_1_2` (2,8)→(1,2) = [(2,8),(1,8),(1,7),(1,6),(1,5),(1,4),(1,3),(1,2)]
     - `MOV_0_6_1` (1,2)→(6,1) = [(1,2),(1,1),(2,1),(3,1),(4,1),(5,1),(6,1)]
     - `MOV_0_11_1` (6,1)→(11,1) = [(6,1),(7,1),(8,1),(9,1),(10,1),(11,1)]，恰穿三敌 (7,1)type41/(8,1)type42/(9,1)type41（T6 自证 0 损可杀），终点 (11,1)
     - 其余 6 扇未开的门 {(2,9),(3,5),(6,6),(8,3),(10,5),(10,8)} 一扇未碰。
   - **战后续走链**：撞敌 engage → `advance_script_walk` `!ok` → 解除武装但 `walk_phase==5` **保留步数**（`scene_script.rs:48-54`）→ `tick_battle` 胜利重步（`scene.rs:3841-3843`，T768 注 `:3507`）→ 战后 `handle_field_input` 见 `game_mode==Dialog` **不查 armed** 直接 advance（`scene.rs:3255`）→ 续走至 (11,1)。
   - 上 (11,1) 踩 type7 → BUG-026 脚本行走踩梯 mode11（`scene_persist.rs:374-379`）→ `change_floor(2)` → `pending_floor`（`:386`）→ §2.2.4(:125) 归约记**终止边 = Exit**。
3. **计数**：Path B 的动作序列无任何 `aᵢ = BumpEnemy`（脚本内撞敌是 `T(s,a)` 内部演化，不是求解器动作）⇒ `kills(π) = 0` ⇒ `min_kills = 0 < 3`。
4. **两说法必坏其一**：若改称「Rust drive 不完成 Path B」，则 §2.1 Exit#2、§2.2.4 归约（:125）、§2.2 覆盖 bullet（:174「脚本19 换层边覆盖『脚本触发离开本层』边界」）、T9「slot19 触发→终止边断言」同时坍塌——提案自述（含 Java T786 `fx-chang-e-floor1-taps` 走位戏走完换层的 A 级引证）就是按「完成」写的。
5. **§3.2-5 的 A 级实测只测了楼梯**：39 格避敌区/78 格全通/0-1 BFS=3 全部是**步行到 (11,1)** 的性质，没覆盖 §2.1 自定义的脚本出口（gate (6,9) 在避敌区内 dist=0、钥匙 dist=0）。一轮 Y5 原文同样只分析楼梯（「避开全部 11 敌格不可达……最少强制交战 3 场——顶行挡在出口前走廊」，:90），**两轮共同盲区**，修订时未补测。
6. **诚实边界**：本轮未在 Rust 侧端到端跑通 drive（只允许写报告文件，不加测试）；红的支点是 (i) 文本/逻辑二难（脚本边在 Exit 内 ⇒ 0<3 必假；不在 ⇒ Exit#2 等四处声称必假）+ (ii) 静态机制逐段实证 + find_path 忠实复刻 + Java T786 引证。二难本身已足够判「需再修」。

**回派修复方向（三选一，须全文同步）**：
- (a) 裁定批2 `min_kills` 只对步行楼梯出口取值 ⇒ 必须同步改 §2.1 Exit#2/§2.2.4/:174/T9/C5，并说明脚本边为何不入 min_kills 量化；
- (b) 承认 `min_kills = 0` ⇒ 删除 T6/§3.2-5/BR-6/§4.2-8 的 ≥3 断言，BR-6「findPath 白走怪格」防御另找钉法；
- (c) 实跑 Rust drive 证伪 Path B ⇒ 须给出卡点的 A 级实证，并与 §2.2.4 自述（完成→终止边）对冲修文。

---

## 四、新黄 Y2-1：C5 行未同步 Obs 读回口径

- `:245` C5 验收列仍写「门面 execute_script + side_a drive + **Snapshot 读回** + pending_floor 终止边」；R2 修法已把终态读回改为 Obs（§1.3 :77 / §2.2.3 :124 / E11 :282 三处），C5 未跟。
- 且 `pending_floor` 实不在 Snapshot/Obs（`script_abs.rs` grep `pending` = 0 命中，harness 读 `scene.persist` `:386`），C5 措辞两处失准。
- 建议改：「side_a drive + **Obs 读回（flags）** + Snapshot 补 difficulty/floor + `scene.persist.pending_floor` 终止边」。属措辞同步黄，不动结论。

---

## 五、新伤扫描（绿：绿面抽 3 + 新行号抽验 ≥8）

### 绿面抽复测 3 条（全绿）
1. **A1-A11 归属（G2）**：提案 :166-173 vs 计划 `:444-469` sed 逐行——A7=:454、A8=:455、A9=:456、A10=:457、A6=:453、A11=:458、A1-A4+A5=批4、样例块 `:463-469`（`:466` 含 `confidence:"bounded-best"`）——无越领，与一轮 G2 结论一致且 diff 未破坏。
2. **选层三理由（§3.2-1..4）**：计划 `:327` = `solve-local --floor 1` ✓；计划 `:185` = oracle 回放先例行（引 `walk_replay_floor1.rs`）✓；13×13 批0 考证（一轮已核）✓——锚全命中，diff 只新增第 5 条、重编号未碰原 1-4 条。
3. **直调 pub 面（G4/G5）**：R3 改写后 `try_step:3315 pub`、`apply_step_cell_effects:3987/pickup_item_type:4221` 私有且**不列直调** ✓；`accepts_field_tap:1217 pub`、`find_path` pub、`side_a::drive:191 pub`、`consume_key_for_door` pub 实测在位；双查询不越权（计划 :257「批3 报审时定死」）未被 diff 触碰。

### 新引入行号抽验 14 条（全命中）
计划 `:301`/`:314`/`:327`/`:418`/`:444-469`/`:463-469`；`scene.rs:2046`（RouteEngage 实测）；`pathfind.rs:11`（「可通行=terrain‖实体槽」doc 行实测）；`walk_replay_floor1.rs:38`（「实体阻挡（门消耗钥匙后放行；怪物格阻挡）」实测）；`subsystem/battle.rs:121`（Refused 阈值 `loss>=0 && loss<hp` 实测）；`script_abs.rs:76/:208`、`combat.rs:42-49/:162-165/:133`、`scene_persist` 5 组、`scene.rs:3315/:3987/:4094/:4221`、`state.rs:21-22`（以上 b1 轮已实测复核）。

### 其他
- diff 未破坏形式化绿面（Refused 不计杀、Q3、gold/difficulty 不进 K 等 G6 面未触碰，K₀ 改写本身已复测）。
- §8 勘误注 8 条在位、原地留痕；diff 全量（前 250 行 + 尾部）逐段过目，除 C5（§四）外无其他漏同步。

---

## 六、夹带检查（绿）

- `eace26f` **恰 1 文件** = 提案文档本身（+63/-41），父 = `78a3bea`。
- commit message 逐项列红6黄8处置，与 diff 实际改动面一一对应，无夹带代码/计划/门禁改动。

---

## 七、闸独立重跑（绿，只读未改码）

- `python3 gates/cli.py check` → **PASS=13 FAIL=0 SKIP=0**（含 `state-baseline [GREEN] 金样逐字节等价：74 场景 sha 全同`、`test-ratchet total=1046`、`rust-tests cargo test --release 全绿`）。
- 独立复跑 `python3 tools/state_baseline.py --check` → **exit 0，`[GREEN] 金样逐字节等价：74 场景 sha 全同`**（双确认）。

---

## closing

**总判：红 1 · 黄 1 · 绿 21 → 需再修（回派）。**

**红项（回派必答）**：
- **R2-1**：`min_kills ≥ 3`（T6 :149 / §3.2-5 :175 / BR-6 :230 / §4.2-8 :262）与提案自身 Exit#2（:111）+ §2.2.4 终止边归约（:125）+ kills 定义（:91）矛盾——钥匙(5,10)0 杀 → 开门(6,9) → 踩 gate19 → drive slot19 → 4 腿 find_path 全通（复刻实测零门阻）→ 战后续走穿三敌（T768 重步链）→ (11,1) 换层 = Exit，`kills(π)=0` ⇒ `min_kills=0`。修复三选一（§三 (a)/(b)/(c)）并全文同步；若反称 drive 不完成，则 §2.1/§2.2.4/:174/T9 四处声称同时坍塌，两说法必坏其一。

黄 Y2-1（C5 行同步 Obs 读回口径）随修一并处理。修订后复核通过则呈用户裁定落码。
