# 对抗复核：求解器批2「单层求解 MVP」设计提案

- 对象：`docs/superpowers/proposals/2026-11-10-solver-batch2-proposal.md`（288 行）
- 被审版本：commit `78a3bea`（父 `b976803`，分支 `modern`）；`git status --porcelain` 无已跟踪修改（仅 `?? android/build/`、`?? docs/findings/reviews/*.md`）
- 对照物：`docs/superpowers/plans/2026-11-09-solver-implementation-plan.md`（下称"计划"）、`crates/game-solver/src/`（批0）、`crates/game-core/src/scene_solver_api.rs`（批1 门面）、`crates/game-core/src/script_abs.rs`（批1）
- 方法：全部主张以 `sed -n` / `grep -n` / python 解析 `assets/raw/sprite*` + `maplv*` 实测复现，不凭记忆；行号均从实测输出计数

## 总判：**红 6 · 黄 9 · 绿 8 → 需修订**

红项集中在三类：①形式化自相矛盾（K 含 hp/kills 致支配剪枝恒空、Snapshot 无 flags）；②自立红线不可执行（pickup_item_type 非 pub）；③A 级/引证错误（楼梯 census 被证伪、find_floor_gate 反读、计划 R3/R10 风险错标、scene_persist 等行号系统性漂移）。修订后可放行落码。

---

## 逐面明细

### 面1 计划对齐 —— **黄**（1 红 1 黄 1 绿，红见面5 R5）

- **绿 G1**：范围引用准确。计划批2 块 sed 实测 316-335：316 标题（估时 2-3d）→ 318 `**内容**` → 320 `solve_local.rs：选一层（建议 floor1…）`（**明列"type83/型69..75 胜利触发的 A 类转移"**）→ 321 三出口 → 322 航点 route_gen → 324 `**判据**` → 326 ```bash → 327 `solve-local --floor 1` → 328 `# 预期输出（形状）{"floor":1,…}` → 329 `grep -c solve_local ≥1（存在性闸，防空过）` → 330 `cargo test … hp/kills 单调性断言绿` → 331 `gates check FAIL=0` → 332 ``` → 334 `依赖批1 + TDD 手工 3×3 红测试`。提案 :10「判据块 :326-332、TDD :334」、T1→:329、T3→:330、T6→:327-328、§3.2-1→:327 全部命中 ✓。
- **绿 G2**：A 编号归属逐条对 ✓。计划 sed 444-460：A1 q1…A3 q3=批4、A4 落盘=批4、A5「确定性复跑 连跑2次 query --all 后 diff」=批4、A6 五元组 replay=批5、A7 金样=每批（:23/:488）、A8 门禁=每批、A9 `grep "game_core::" --include=*.rs | -v bin/solver.rs`=批2、A10 批1、A11 批5。提案 :166-167 归属（A1-A5 批4 正式 / A6、A11 批5 / A7、A8 每批 / A9 批2 / A10 批1 / T10 预演 A5）无越领、无漏领。
- **黄 Y1（缩水）**：计划 :320 批2 内容明列「**型69..75 胜利触发的 A 类转移**」，提案 :61 直接 ❌ 裁出（理由：floor1 census 无此类怪）。floor1 census 确认无 69-75（实测），但这是对计划批2 范围的收缩——须用户/计划侧确认修订（或补手工夹具覆盖 A 类转移），不能由提案单方面裁掉。
- 无批3/批4 内容偷渡：§1.4 明确「字典序/Pareto 口径留批3 报审」、Q3 形式化仅注恒真不实现、A5 正式实现归批4（T10 只预演）✓。

### 面2 行号锚抽验（实测 35+ 条，21/24 E 锚绿）—— **红 2 条（R4、R6）**

抽验明细（实测值 vs 提案引用）：

| # | 提案引用 | 实测（sed/grep） | 判 |
|---|---|---|---|
| 1 | E1 计划:316-335 | 批2 块 316-335 逐行命中 | ✅ |
| 2 | E2 计划:444-468 | A1-A11 行 448-458、注 459-460、样例 json 462-467 | ✅ |
| 3 | E3 计划:404-419 | R1-R12 齐（R1:408/R2:409/R3:410/R10:417/R11:418/R12:419） | ✅ |
| 4 | E4 计划:240-258 | §3.3/`struct SolverState`:244/canonical:257/保真红线:258 | ✅（提案另一处引 :256 见 R5） |
| 5 | E5 combat.rs:106 | `pub fn predict_battle_hp_loss` :106 | ✅ |
| 6 | E6 combat.rs:170 | `pub fn consume_key_for_door` :170 | ✅ |
| 7 | E7 combat.rs:51-63 | `pub struct PlayerCombat` :51，hp/atk/def:52-54，keys:54-56，gold:58，item_types/uses/stack:60-62 | ✅ |
| 8 | E8 battle.rs:59-75/85/110/121 | BattleCtx:59-75（10 写槽 61-70 全 pub + 3 读槽 72-74）、`predict_engagement`:85、`impl resolve_bump`:110、Refused 判据 `if !(loss >= 0 && loss < ctx.player.hp)`:121 | ✅ |
| 9 | E9 pathfind.rs:21/55 | `pub struct Walkability`:21、`pub fn find_path`:55 | ✅ |
| 10 | E10 scene_solver_api.rs:45/52/59 | `parse_op`:45、`execute_op`:52、`execute_script_instruction`:59 全 pub | ✅ |
| 11 | E11 script_abs.rs:118/191 | `Snapshot::from_scene`:118、`pub fn drive`:191 | ✅ |
| 12 | E12 scene.rs:927/931 | `try_run_scene`:927、gate 初值:931 | ✅ |
| 13 | E13 scene.rs:1031/1032 | case66:1031、case67:1032 | ✅ |
| 14 | E14 scene.rs:4069/4073 | 踩 type83→try_run_scene:4069、load:4073 | ✅ |
| 15 | E15 scene.rs:4077-4088 | 7 臂 4077-4082（change:4080）、8 臂 4083-4088（change:4086）（sed 4077-4092 实测） | ✅ |
| 16 | E16 scene.rs:3386-3404 | 门臂 `1 \| 2 \| 3 => {`:3386、`consume_key_for_door`:3402、`entity::remove`:3404 | ✅ |
| 17 | E17/E18 3593、1553、3551-3576 | BumpEngage resolve_bump:3593、probe_step:1553、type12 臂:3551、bump_reveal_target(12):3562 | ✅ |
| 18 | E21 gameplay:163/168/334 | R-move-4（type12 隐形墙→撞变 type11）:163、字节佐证:168、R-scene-2:334 | ✅ |
| 19 | E22 route_gen.py:12-15/29 | 航点语法:12-15、KEY:29 | ✅ |
| 20 | E24 lib.rs:31-33 | `pub mod scene_test_support {`:31、pub use:32、`}`:33 | ✅ |
| 21 | E23 state.rs s0/Q3_EQUIP_SET | `s0()`:21、`Q3_EQUIP_SET: &[36,40]`:29 | ✅ |
| 22 | E19/E20 scene_persist:413-446 / :371-378 | **swap_floor 实 410-434、find_floor_gate 实 435-443、BUG-026 注实 374-379** | ❌ R6 |
| 23 | §2.1 scene_persist:423/425 | floor0 place(1,2) 实 **:421**、floor50 place(6,7) 实 **:424**（floor1 :426-427 ✓） | ❌ R6 |
| 24 | BR-5/§6-7 combat.rs:131+ | DC-9 注起 **:133** | ❌（并入 R6 修） |
| 25 | §4.1 R3 行「§3.3 :256 原文」 | canonical 实 **:257**（244 struct/245 floor/252 kills/253 monsters_alive/257 canonical/258 保真红线） | ❌ 并入 R5 |
| 26 | §2.1/§6-3「全仓无楼梯 param&0xff==1 指向 floor1（55 sprite 扫描，A 级）」 | **python 全 55 sprite 实测 3 处反例**（见 R4） | ❌ R4 |
| 27 | :108「下行…find_floor_gate 找 type8=无⇒不落点」 | **反读**（见 R4） | ❌ R4 |
| 28 | 选层理由：plan:327 判据硬编码 floor1 | sed 327 = `cargo run … solve-local --floor 1` | ✅ |
| 29 | 选层理由：plan:185 walk_replay_floor1 先例 | sed 185 = §2.8 oracle `walk_replay_floor1` | ✅ |
| 30 | 选层理由：13×13、plan:84 | maplv1 头 `1a00 1a00` = 26×26 raw >>1 = 13×13；walk_replay_floor1.rs 存在 13×13 | ✅ |
| 31 | :180 确定性引 plan:343 | 批3 块内固定遍历序行 :343 | ✅ |
| 32 | §3.2-3「批0 b38b694」 | `git cat-file -t b38b694` = commit | ✅ |
| 33 | slot19 尾 `MOV_0_11_1 MOV_0_1_11`（:125） | fixture 第 20 行逐字一致，无 delay/dialog 指令 | ✅ |
| 34 | floor1 census（:110/:116/:174 等） | type7×1@(11,1)param2、type1×7、type15×1@(10,11)、type26×5、type83×3{19@(6,9),66@(6,9),67@(6,11)}、敌 11 只 types{41..46}、无 type8/12/6/77/78 | ✅ |
| 35 | 报审声称的 6 条 scene.rs 漂移复测（926→927、4068→4069、3315、3386 等） | 全部独立复测命中 | ✅ |

- **绿 G3**：≥12 条要求达成（实测 35 条，21/24 E 锚绿）；报审声称的 6 条漂移复测全对。
- **红 R4、R6**：见下方红项清单。

### 面3 形式化正确性 —— **红 3（R1、R2、R3）+ 1 绿**

- **红 R1**：去重键 K 自相矛盾（见红项）。
- **红 R2**：终态读回声称 Snapshot 带 flags，实际 Snapshot 无 flags 字段（见红项）。
- **红 R3**：直调清单列了非 pub 的 `pickup_item_type`（见红项）。
- **绿 G4/G5/G6**：
  - 双查询独立出解**不越权**：计划 :257 原文「批3 用分层字典序或 Pareto 集，**口径在批3 报审时定死**」——提案 :97 明确不选边，Pareto 前沿措辞与 :257 一致；三出口分立与计划 :321/A1/A2 一致。
  - 转移「只许直调」其余项 grep 实测全存在且 pub：`resolve_bump`（battle.rs:79 trait/:110 impl）、`predict_engagement`:85、`predict_battle_hp_loss` combat:106、`consume_key_for_door`:170、`find_path`:55、`Walkability`:21（头注:10「可通行=terrain||can_path_through_entity」）、`parse_op/execute_op/execute_script_instruction` 45/52/59、`side_a::drive`:191、`probe_step`:1553、`try_step`:3315、`change_floor` pub:349、`BattleCtx` 59-75 全 pub 写槽、`effective_attack_vs_type`:93（:96-98 护符×2 分支）、item_effect 141/147。
  - 其余形式化成立：Refused 不计杀推导与 :121 判据一致；出口集合/终止边归约自洽；gold/difficulty/floor 不进 K 论证成立（floor1 census 无商人 type77/78）；Q3 恒真引 state.rs `Q3_EQUIP_SET` ✓。
- **黄 Y8**：qol 分量进 s(:22) 不进 K(:202)，冗余性（qol=筋斗云在 bag ⇒ 可由 bag 派生）未按 §1.1 自立判据写一句论证。

### 面4 TDD 红测试 T1-T10 —— **绿（选层三条全中）+ 黄 2**

- **绿**：选层三条理由逐条实测 ✓——(a) plan:327 判据硬编码 `solve-local --floor 1`；(b) plan:185 `walk_replay_floor1` oracle 真地形先例存在；(c) 13×13（maplv1 头 `1a00 1a00`=26×26 raw>>1、walk_replay_floor1.rs 13×13、plan:84）。T2 与计划 :334 TDD 原文同构（1 强怪挡路：能杀则 reachable、杀不过则 Refused 不计杀）✓。T1/T2/T4/T6/T7/T10 均依赖未落地的 C1/C2/C4/C6/C7，写时可红 ✓。
- **黄 Y2（恒绿）**：T5 census 快照（:148）及 T8/T9 中直接断言既有 core 行为的部分，写时即绿——计划 :301/:314 明文「B 类'状态不变'天然空过，**不得当红**」。须绑到 solve_local 夹具加载/转移层（红测试挂在批2 产物上）才真红，否则从"红测试清单"降级为回归护栏。
- **黄 Y3**：T3(iv) 依赖「杀数上限 k 参数化」这一未承诺的实现形态，而 C3 验收（:236）写死"四断言 (i)-(iv) 绿"——删 (iv) 或在切片里承诺参数化，二选一。

### 面5 风险面 —— **红 1（R5）+ 黄 4（Y4-Y7）+ 绿 1**

- **红 R5**：§4.1 承接表两行错标（见红项）。
- **绿 G7**：R1:408（状态爆炸）/R2:409（隐藏墙，对策「与 findPath 对拍（批2）」原话 sed 实测）/R12（经 plan:20「items.json 22-25 effect 禁作求解器效果源」红线覆盖）承接准确；BR-1..BR-7 与计划 R 系列除 R3/R10 错标外无重复矛盾。
- **黄 Y4（可直接关闭）**：BR-5 前提措辞错——`trait_flags_for`（combat.rs:42-49）：49|53|74→1、69→2、**其余→0** ⇒ floor1 怪 41..46 的 trait_flags **全 0**，闭式分叉面在 floor1 不存在；且 trait_flags ∈{0,1,2} 位标志，"含 23/24"混淆了护符道具 id（`TRAIT1_AMULET=23`，combat.rs:162）。修法：BR-5 改写为实测结论，保留"批4 全图重测 49/53/74/69"。
- **黄 Y5（可直接关闭）**：§6-8 必经点已可裁定（A 级 python 连通性实测）：findPath 语义下 (6,11)→(11,1) 可达 ✓；**避开全部 11 敌格不可达**（门全开也不可达，避敌可达区仅 39 格、出口在区外）；**最少强制交战 3 场**——顶行 (7,1)type41、(8,1)type42、(9,1)type41 挡在出口前走廊，最短强制路穿门 (6,9)。提案 :175「无 RouteEngage 必经点【待复核】」应翻转为「存在，min_kills≥3」；T6 可加 `min_kills ≥ 3` 断言；walk_replay_floor1.rs:40 注释亦证"怪物格阻挡"。
- **黄 Y6**：R11（计划:418 筋斗云邻梯瞬移=独立换层通道，归批3）全文未提；floor1 有 type15，批2 因 UseItem 裁出不会触发——应在 §4.1/§1.2 显式声明"批2 无筋斗云换层边，R11 归批3"。
- **黄 Y7**：BR-7 输出 `confidence=bounded-best`（:224）与 T6 JSON 形状（:149 无 confidence 字段）不一致——形状加字段或 BR-7 改为"批4 输出时标"。

### 面6 夹带 —— **绿 G8**

`git log -1` = 78a3bea，`git show --stat` 恰 1 文件（提案文档本身）；工作树无已跟踪修改；父提交 b976803 ✓。

---

## 红项逐条（含修法）

### R1 · 去重键含 hp/kills，支配剪枝恒空（形式化自相矛盾）

- **证据**：提案 :202 `K=(pos, ents_digest, flags, keys, bag, hp, atk, def, kills)`；:100/:209 支配条件「**K 相同**且 hp₁≥hp₂ ∧ kills₁≤kills₂ ⇒ 丢 s₂」。K 相同 ⇒ hp 相等 ∧ kills 相等 ⇒ 支配条件退化为 `hp₁=hp₂ ∧ kills₁≤kills₂`（恒真当且仅当相等）——**永不可能丢掉不同状态**；:212(b)「Pareto 支配把 hp×kills 双维压到前沿（每 K 段至多 O(战斗次数) 个非支配点）」同理落空，规模论证失效。另 :208「转移只读 K 分量」与 Refused 读 hp（battle.rs:121）冲突。
- **修法（二选一）**：① K 去掉 hp/kills → `K₀=(pos, ents_digest, flags, keys, bag, atk, def)`，hp/kills 作为**支配维度**（与 §1.4 双查询 Pareto 表述一致），:208 改写为"转移读取面 ⊆ K₀ ∪ {hp, kills}，hp/kills 不参与等价合并、只参与支配"，:212(b) 保留；② 删除支配剪枝，承认 hp×kills 全维展开并**重算规模上界**（:212 裸上界 ~10⁹ 失去 (b) 压缩）。

### R2 · 终态读回引用 Snapshot 带 flags——Snapshot 无此字段

- **证据**：提案 :77「终态读回用 `script_abs.rs:118 Snapshot::from_scene`」、:124「Snapshot::from_scene（:118）读 stats/flags/px/py/entities」。实测 `Snapshot` struct（script_abs.rs:95 起）字段为 stats/item_types/item_uses/facing/px/py/difficulty/floor/equip*/entities，**无 flags**；flags 只在 `Obs`（:73-82，`pub flags: Vec<bool>`:76）与 `drive` 返回（:208 `flags: sc.script_line_flags.clone()`）。与提案 :84「Obs 含 stats/flags/px/py/entities」自相矛盾。
- **修法**：:77/:124 改为"终态读回 flags/px/py/entities/stats 用 `drive` 返回的 Obs（:191/:208）；difficulty/floor 等才用 Snapshot::from_scene"，或统一全用 Obs。

### R3 · 直调清单列非 pub 的拾取函数，违反自立红线

- **证据**：提案 :14 红线「一律直调既有 pub 面」；:31/:79 把 `scene.rs:4221 pickup_item_type` 列为拾取出处。实测 `fn pickup_item_type(&mut self, t: i32)`（scene.rs:4221）**无 pub**；:4094 落格也在私有 `apply_step_cell_effects`（:3987）内。
- **修法**：改为"经 pub `try_step`（scene.rs:3315）落格触发（apply_step_cell_effects 内部调 pickup）"，或走批1 门面提案「另立提案」条款补暴露；红线表不得列不可达面。

### R4 · 楼梯 census A 级主张被证伪 + find_floor_gate 反读

- **证据①（census）**：提案 :108/:250「全仓无任何楼梯 param&0xff==1 指向 floor1（55 sprite 扫描，A 级）」。python 全 55 sprite 实测 **3 处反例**：`sprite0` type7 `extra=0101`（&0xff=1）、`sprite2` type8 `extra=0001`（=1）、`sprite54` type7 `extra=0001`（=1）；floor1 自身 type7 `extra=0200`=2 ✓。sprite2 存在 t8→floor1 的下行换层边（`change_floor(1, true, false)`）。
- **证据②（反读）**：:108/:250「下行进入 floor1…find_floor_gate 找 type8=无 ⇒ 不落点」——实测 scene_persist.rs:429 `find_floor_gate(self.persist.went_down)`，下行 `went_down=true` → :436 `let want = if up { 7 } else { 8 }` 找的是 **type7**；floor1 有 type7@(11,1) → `Some` → `step_off_stairs` **会落点**。推理方向反了。
- **修法**：更正 census 主张（"无楼梯"不成立，须按实测列出 3 处并排除/处理）与 §6-3 推理；批2 钉入口 (6,11) 仍成立，但须声明**只覆盖 `!went_down` 路径**（scene_persist.rs:426），下行进入分支交由换层语义处理。

### R5 · §4.1 风险承接表错标 R3/R10 + 入口状态未钉死

- **证据①**：:195「**R3 三查询目标打架**（:410）」——计划 :410 R3 实为「**剧情强制 ROS_2/DES 可达性约束**（op_ros_*/op_des_* 可能使最优路线被剧情否决）」；grep `目标打架` 于计划全文 **0 命中**（"三查询"只出现在 §1.2/批4 语境）。同行引「§3.3 **:256** 原文」应为 **:257**（grep `canonical` =257）。
- **证据②**：:196「**R7/R10 门面暴露**」——计划 :417 R10 =「**new_game_stats 无调用方**（§2.5）｜初始状态接线不确定｜…**S0 常量先按 (300,10,10,51) 写死并注证据**」，非门面暴露（门面暴露只有 R7 :414）。因此批2 实际**未承接入口状态**：§1.4 `hp(s₀)` 全文未定义，max_hp 输出缺基线（仅 E23 提 state.rs `s0()` 未引值）。
- **修法**：R3 行改为引真实风险（剧情强制约束——floor1 slot19 的 TAK/DES 已由 drive 覆盖，可写明）或标"提案自列 BR"；R10 单列一行，承接「入口状态 = state.rs `s0()`（hp₀=300/atk₀=10/def₀=10/keys/gold 钉死并注证据）」；:256→:257。

### R6 · scene_persist 等行号系统性漂移（提案自称"全部行号实测"）

- **证据（sed/grep 实测 vs 提案引用）**：

| 位置 | 提案引用 | 实测行 |
|---|---|---|
| BUG-026 注（E20/:125/:218） | 371-378 | **374-379** |
| `swap_floor`（:78/:108/E19） | 413-437（E19: 413-446） | **410-434**（E19 段: 410-443） |
| `find_floor_gate`（:78/:108） | 438-446 | **435-443** |
| floor0 `place(1,2)`（:108） | 423 | **421** |
| floor50 `place(6,7)`（:108） | 425 | **424** |
| combat.rs DC-9 注起（BR-5/§6-7） | 131+ | **133** |

（同文件正确锚：floor1 `place(6,11)` 426-427、`pending_floor=`:386、`change_floor` pub:349、`for e in 0..self.entities.count`:437、reload:444。）
- **修法**：按实测值全文更正（含 E19/E20 证据行、§1.3、§2.1、§2.2.4、BR-1、:222 DC-9 行）；修订后重跑一遍 `grep -n` 自查，杜绝"实测"声明下的漂移。

---

## 附：攻击面 10 点裁定速览（§6）

1. **§6-1 census 标签冲突** → **计划勘误**：plan:92 原文「type12 门=24、type13=1；楼梯 type7(上)=54、type8(下)=51」；type12 真身=隐形墙（gameplay.md:163 R-move-4、scene.rs:3551 臂、sprite23 24 枚字节佐证）；"门"标签疑似源自 sprite_spawn.rs:17-18 `DoorTables{door12,door13}`「门参数表」。提案按 scene.rs:3386 门=type1/2/3 走**正确**。
2. **§6-2 六锚复测** → 全对（见面2 表）。
3. **§6-3 入口路径** → **证伪**（R4）。
4. §6-4/5/6 → 各锚实测成立（面2 表）。
5. §6-7 trait_flags → **关闭**（Y4）。
6. §6-8 必经点 → **关闭**（Y5：存在，min_kills≥3）。
7. §6-9 A 归属 → 核对无误（G2）。
8. §6-10 route_gen CI 强度 → 提案自问，留用户裁定，不判红。

## 结论

**红 6 · 黄 9 · 绿 8 → 需修订。** 红项 R1-R6 逐条给出修法，全部可在提案文档内完成（不动代码）；黄项 Y1/Y3/Y4/Y5/Y7 建议随修订一并处理（Y4/Y5 可直接用本报告实测数据关闭）。修订后呈用户裁定落码。
