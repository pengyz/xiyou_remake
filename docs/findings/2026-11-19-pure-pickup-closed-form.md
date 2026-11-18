# 纯拾取层的闭式解（fail-closed）—— 判定闸门、真引擎回放、f54 破界实测

日期：2026-11-19 　分支：`modern`（基线 `cabb1b3`）　范围：新增 `crates/game-solver/src/pure_pickup.rs`、
`crates/game-solver/src/solve_local.rs`（层入口接入 + 只读桥）、`crates/game-solver/tests/pure_pickup.rs`。

> 【2026-11-19 P6 更新（提交 `64ae5f1` 之后）】范围扩到 **`crates/game-core/src/effect_registry.rs`**
> ——新增**只读采样探针** `certified_monotone_pickup`（与 `certified_pure_hp_gain` 并列，
> **不改后者语义、不改任何 game-core 行为**）。闭式的语义从「纯加血层」推广为「**单调拾取层**」，
> 判据从 7 条增到 15 条（+7 条 P6 判据、+1 条 `#[ignore]` 的证伪记录），witness 契约升到
> `xiyou/witness/v2`。**结论先说**：语义推广成立，但**在现有 55 层数据上不产生任何新的闭式命中**
> （f54 的 18 枚全是 HpUp，P5 已覆盖；其余层被 G1/G2/G2a 挡住）——详见 §0.7、§7.3、§9.2。

## 0. 结论摘要

1. **判定「纯拾取子层」的六道闸已实现并有反例覆盖**（§1）。任一不成立 ⇒ `try_closed_form` 返回
   `None`，`pure_pickup_stat().reason` 记**具名理由**，调用方原样走通用搜索（fail-closed，禁静默）。
2. **f54 闭式命中**：`out_hp = 827 + 2400 = 3227`（理论最优），`ok=true`、`truncated=false`，
   `route` 63 个 waypoint，**0.01s**（旧：`out_hp=2027`、`truncated=true`、23 waypoint、5.82s）。
3. **路线真走通**：闭式产出的 `route` 被逐步喂回真引擎（`Search::pickup_step` → `try_step` + 拍推进 +
   读回），62 步全成功，终态 hp/位置/实体表/零副作用逐项断言通过（§3）。
4. **层界被打穿**：`solver run` 从 `floors=3 / total_steps=196 / blocked at floor 54` 变成
   `floors=6 / total_steps=275 / blocked at floor 3`（§7）。f54 之后 f1/f2 由通用搜索走通（hp 不变），
   **撞界点后移到 f3**。
5. 判据 1–7 全绿；变异自证 A/B 各红一次且**失败面互不相同**（§6）——A 证明「漏算一颗药」会被回放
   抓住并 fail-closed，B 证明「假 route + 跳过回放」能骗过层目标与整条 `solver run`、但**骗不过判据 2**。
6. 引擎一行未改；`SOLVER_LAYER_LIMIT` 语义、默认层界、`ok` 契约、74 金样 sha 全不变（§8）。
7. **P6：单调拾取推广**（§2 的定理版、§7.3/§7.4 的实测）。G3 → G3′：拾取物的效果不再要求「只加血」，
   改为经**真引擎采样**认证「六轴（hp/atk/def/黄/蓝/红）全部不下降 + 背包纯追加 + gold/装备/教学位零变化」；
   闭式同时记录 atk/def/三钥/背包的期望值，回放**逐字段**断言（新增 `replay_stat_mismatch`/
   `replay_key_mismatch`/`replay_bag_mismatch`/`bag_no_room`）。
   **f54 在 P5/P6 下逐字段相同**（它本来就是全 HpUp：`31×8 + 32×10 = 2400`）；
   f51..f54 四层与父提交 `0a244ff` 的 15 个共字段**逐字段一致**，schema 标签 v1→v2（§7.4）。
   **f3 的「4027 闭式命中」被实测证伪**（§9.2）——f3 不是纯拾取层，任务书该条前提不成立。

## 1. 闸门表（谁在守什么）

实现：`crates/game-solver/src/pure_pickup.rs:421`（`try_closed_form`）。编号**不是执行序**，
执行序见模块头注释（短路的闸先跑，避免早退遮蔽具名理由）。

| 闸 | 条件 | 违反时 `reason` | 反例（实测） |
|---|---|---|---|
| G0 | `level` 存在、`entry` 在界内 | `no_level` / `entry_out_of_bounds` | — |
| G6 | `policy == ClosurePolicy::OFF` | `policy_not_off` | Q1/Q2/Q3 是「按谓词剪枝」的搜索语义，闭式不建模 |
| G5 | 入场 `hp > 0` | `entry_hp_not_real` | 占位 hp=0 不是真实入场态 |
| G2a | **入场格**步臂只允许「门已开的场景门」 | `step_arm_hazard_on_entry` | 合成层：61 放入场格 ⇒ 回退（判据 4 补） |
| G2 | 危险步臂格（除入场格）不可几何可达 | `step_arm_hazard_reachable` | 合成层：61 在走廊 ⇒ 回退；**f52 实测 `reason_ty=83`** |
| G1 | 无怪物（`cat8`） | `monster_ents` | 合成层：41 ⇒ 回退；**f1 实测 41、f53 实测 51** |
| G3′ | 拾取物：每格 1 枚、不在入场格、格纯、`certified_monotone_pickup=Some(g)` 且 `g.is_gain()`（六轴 ≥0、背包纯追加、至少一轴严格增、gold/装备/教学位零变化，**全部来自真引擎采样**）；另需背包有余位 | `pickup_cell_shared` / `pickup_on_entry_cell` / `pickup_cell_not_pure` / `pickup_not_certified` / `bag_no_room` | 合成层：**13**（已持有且 uses 会 2→1 **下降**）⇒ `pickup_not_certified`；旧版用 26 是 P5 口径，26 在 P6 已是合法单调增益 |
| G4 | 至少一条 `Stair` 出口**能从纯图上走上去** | `no_stair_exit_reachable` | f54 的 (7,10) 出口唯一邻格是入场格 ⇒ **结构性排除**（不是「未证明」，见 §9.1） |

**G2a 是本片新发现的真风险（A 级）**：`crates/game-core/src/scene.rs:3878-3887`（`apply_entry_step_effects`
的注释与实现）明确写着进图链 `dispatchModeEntry` case 3 会在**出生格**调 `apply_step_cell_effects`
（"进图瞬间出生格触发（type83 → try_run_scene/loadLevelScript）"）。求解器的 `build_scene` 不走这条链
（引擎语义本片不动），所以「入场格有陷阱」时闭式的 `entry_hp + Σ` 会与**真实玩家**的入场扣血不一致。
⇒ 只放行 `ty83 ∧ probe_gate(param)`：门已开时 `gate_arrive` 直接 `Pass`、脚本根本不跑（上一片已证
脚本 64 不动 hp），这才是可证中性的唯一情形。f54 的入场格 `(7,9)` 正好是 `ty83 param=64`，`probe_gate=true`。

**G1 为什么不用任务书的 `41..=76`**：那一段混着 6 种**踩格副作用臂**（`5/60/61/62/76/83`，见
`solve_local.rs:1321` 的 `STEP_ARM_HAZARD_TYPES`），而本作数据里 `ty60/61/62` 的 `category` 正是 8
⇒ 若按「`cat8` 或 `41..=76`」判怪，任何陷阱层都会以 `monster_ents` 提前失败、**永远看不到**
`step_arm_hazard_reachable`。故把 G2 排在 G1 之前，怪物只认 `cat8`（与求解器 `monster_ents` 同源）。

**G2 的「可达」必须让 hazard 格自身可走**（`grid.geo_reach`，`pure_pickup.rs:186`）：第一版用 `pure`
做遍历条件，而陷阱格永远不是 `pure`（`cat8`⇒`classify` 判 `Monster`）⇒ 陷阱格从不进入几何可达集、
这道闸形同虚设（实测：`pure_pickup_fails_closed_with_hazard_cell` 一开始红在 `monster_ents`）。
现为「`pure ∨ hazard` 都可走、可不可踩由 G2 判」。

## 2. 闭式与路线构造

- **定理（P6 口径）**：若某层满足 G1/G2/G2a 且**每个可达拾取物的效果向量 Δ 六轴全部 ≥0、
  背包只做纯追加、gold/装备/教学位零变化**（G3′，全部由真引擎采样认证），则该层「把可达拾取物
  全捡一遍再出层」是 **HP 最优**：每枚增益非负且互不抵触（Δ 是常量向量、与顺序无关），
  子集效果 = 逐项相加 ⇒ 全捡支配任何少捡 ⇒ `out_hp = in_hp + Σ hp_delta` 是上界且可达，
  可达性由引擎回放背书。**定理不覆盖什么**：层内有怪就可能**必须杀怪扣血**（f3 就是，§9.2）、
  层内有陷阱/宝箱/门就可能踩格扣血——那些层根本进不了 G1/G2。
- **上界**：G1 无怪 ⇒ 无战斗扣血；G2 无可踩危险臂 ⇒ 无陷阱/宝箱/组门/场景门副作用；
  G3′ 全部拾取六轴非负；G5 入场 hp 真实 ⇒ 层内 `hp ≤ entry_hp + Σ hp_delta`。
- **可达（全捡最优）**：G4 + 纯图连通 ⇒ 存在把每枚药都捡完再去出口的走法。拾取**不挡路**
  （拾取格在拾取前后都是 `Cell::Pickup`/`Cell::Move`，都放行、都 `+1` 步，见上一片 §1.2），
  步数只是 tie-break（`HpLoss`：hp 大 → steps 小 → route 字典序）。
- **路线构造**：`pick_sequence`（`:275`）在纯图上反复 BFS「最近未拾取物优先」，`tie-break` 取最小
  `(y,x)` 保确定性；然后 `bfs_path` 逐段连路（`entry → 药₁ → … → 药ₙ → 出口`）。
- **禁再进入场格**：`step_ok`（`:181`）排除 `entry`。理由：入场格是起点不是「到达」，重入会触发
  该格的步臂（G2a 只对「门已开的场景门」证明了中性，别的没证）。f54 的直接后果：`(7,10)` 出口的
  唯一邻格就是入场格 ⇒ **纯图只剩 `(7,1)` 一个候选出口**（实测 `route` 终点即 `(7,1)`）。
- **有限回溯**：`ORDERS`（`:80`）四种顺序（最近/最远/行序/逆行），**上限写死 4**，第几种成功写进
  `orders_tried`；全失败 ⇒ `give_up` 回退。f54 用第 0 种（最近）即成功（`orders_tried=0`）。

## 3. 引擎回放（判据 2 的实现，本片硬要求）

- 桥：`solve_local.rs:3088`（`replay_pickup_route`）+ `:3076`（`PickupReplayOut`）；逐步调
  `Search::pickup_step`（`solve_local.rs:2916-2985`：`build_scene` → `game_mode=Map` →
  `try_step(facing)` → 拍推进到落格 → 读回 `player` 10 字段 / `readback_ents` / `flags` 位图）。
  它**不调 `gate_arrive`** ⇒ 纯路线必须自己排除 hazard 格（与 G2 一致）。
- 闭式在 `pickup_step` 之上再做 7 项断言（`pure_pickup.rs:606-674`）：
  终态位置 == 出口；`hp == entry_hp + Σ`（一字不差）；`atk/def/三钥/gold/item_types/item_uses/
  item_stack_size` 逐字段不变；`flags` 位图不变；拾取实体**全消失**；非拾取实体按
  `(ty,x,y,param)` 多重集不变；`steps == cells.len()-1`。
- 任一项不符 ⇒ `reason`（`replay_step_failed`/`replay_pos_mismatch`/`replay_hp_mismatch`/
  `replay_side_effect`/`replay_flags_changed`/`replay_pickup_left`/`replay_ents_mismatch`/
  `replay_steps_mismatch`）⇒ 回退通用搜索。
- **背包槽的规范化（差点踩的假红）**：入参 `item_types=[]`，真引擎读回是 `[0;32]` ⇒ 逐槽比较会假红
  `replay_side_effect`。现用 `trim_slots` **只裁尾部 0 槽**，非零槽仍逐槽比对（**不放宽**断言）。

## 4. 可观测性（结果里必须看得出闭式是否被采用）

- `PurePickupStat`（`pure_pickup.rs:90`）：`attempted/adopted/reason/reason_ty/pickups/sum_delta/
  route_len/replay_steps/orders_tried/entry_hazards/hp_expected/hp_replayed/exit`；
  `pure_pickup_stat()`/`pure_pickup_stat_set()`（thread_local，体例照 `PillDomStat`）。
  判据 1 实测一行为：`reason=adopted, pickups=18, sum_delta=2400, entry_hazards=1, orders_tried=0,
  replay_steps=62, hp_expected=3227, hp_replayed=3227`。
- `ExitSolve.pure_pickup: bool`（`solve_local.rs:435`）：层结果直读「这条是闭式来的吗」。
- **memo 也要能回放**（`solve_local.rs:698`）：`LAYER_MEMO` 值从 6 元组扩成 7 元组（末项 stat），
  命中时 `pure_pickup_stat_set(hit_pure)`。否则第二次问同一层会看到 `reason=not_run` ⇒ 静默。
- **理由优先级（由变异 A 逼出来的）**：`REASON_RANK_*`（`pure_pickup.rs:346`）+ `note_reason`（`:352`）。
  第一版「记最后一条理由」，f54 的 `(7,10)` 出口（纯图不可达 ⇒ `no_stair_exit_reachable`，**预期内的
  无害失败**）把真正有价值的 `replay_pickup_left`（**假设被打破**）覆盖了。现在 rank
  `replay_*=3 > pickup_unreachable=2 = bag_no_room=2 > no_stair_exit_reachable=1 > route_construct_failed=0`，
  同级保留先出现的。
- **P6 新增可观测面**：`PurePickupStat.sum_atk/sum_def/sum_keys/sum_bag`（结构体末四项，`CLEARED` 同步置 0）；
  `ExitSolve` 除 `pure_pickup: bool` 外再补 `pure_pickup_reason: Option<String>`（`solve_local.rs`，取
  `PurePickupStat::reason_label()`：未尝试 ⇒ `None`，有 `reason_ty` ⇒ `"理由/类型"`），witness 的 floor
  记录把它落成 `pure_pickup` / `pure_pickup_reason`（契约 `xiyou/witness/v2`）。

## 5. 判据（行锚 + 谁在守谁）

`crates/game-solver/tests/pure_pickup.rs`：

| 判据 | 测试（行） | 断言点 | 守什么 |
|---|---|---|---|
| 1 | `pure_pickup_closed_form_hits_theoretical_max`（:228） | `out_hp==3227`、`ok&&!truncated`、`pure_pickup`、`stat.pickups==18/sum_delta==2400/entry_hazards==1/orders_tried==0/hp_expected==hp_replayed==3227`、`route.len()==63` | 「口径对」：真值 = 827+2400 |
| 2 | `pure_pickup_route_replays_in_engine`（:278） | 独立回放 `hp==3227`、`pos==(7,1)`、`steps==62`、实体 37→19；**负控**：只走前 20 步 ⇒ `hp==1977`、`pos==(10,3)≠出口` 且断言必须失败 | 「路线真走通」；负控证明回放不是自证 |
| 3 | `pure_pickup_fails_closed_with_monster`（:340） | 无污染控制 `adopted/500`；加 41 ⇒ `!pure_pickup` + `reason==monster_ents` + `reason_ty==41` | 闸门 + **反证同层**（差异只有那一只怪） |
| 4 | `pure_pickup_fails_closed_with_hazard_cell`（:355） | 61 在可达格 ⇒ `step_arm_hazard_reachable/61`；61 在**入场格** ⇒ `step_arm_hazard_on_entry/61` | 陷阱不可达 + G2a |
| 5 | `pure_pickup_fails_closed_when_pickup_not_certified`（:369） | ty26 换掉一颗药 ⇒ `pickup_not_certified/26`，且 `hp_expected/hp_replayed` 保持 0（未跑到回放） | 认证是硬门槛，不是启发式 |
| 6 | `pure_pickup_does_not_change_exact_results_on_non_pure_floors`（:419） | f51/f52/f53 的 `(ok,truncated,in_hp,out_hp,exit,route.len(),route_fnv)` **逐字段**等于旧基线（`OLD_CHAIN`，从 `cabb1b3` worktree 实测抓取）；另断言三层拒绝理由具名 | 黑盒差分：非纯层结果一字不动 |
| 7 | `pure_pickup_chain_breaks_the_f54_wall`（:470） | `run_witness(max_floors=4)` ⇒ `floors==4`，f54 `ok/hp==3227/exit[7,1]/waypoints==63` | 链层面：层界真的破了 |

`route_fnv`（`:405`）= FNV-1a over 每个 token + `|`：把整条 route 的**内容**压成可断言的 u64，
比只断长度强（顺序/坐标变动都会变）。

**P6 新增判据**（同在 `crates/game-solver/tests/pure_pickup.rs`；合计 13 passed + 1 ignored）：

| # | 测试 | 守什么 |
|---|---|---|
| 8 | `monotone_floor_collects_keys_and_stats` | 合成层（黄钥 26 + 威力丹 29 + 回春 32）⇒ `out_hp=300`、atk=11、def=10、yellow=1、`sum_atk==1`、`sum_keys==(1,0,0)`；**证「单调」不只是加血** |
| 9 | `non_monotone_pickup_still_fails_closed` | 合成层放 **13**（探针已持有、uses 会 2→1 下降）⇒ `pickup_not_certified/13`；同层换 26 ⇒ 采用（正对照）。旧版用 26 是 P5 口径，26 在 P6 已是合法单调增益 |
| 10 | `monotone_floor_collects_bag_item` | 32 槽玩家 + 19 ⇒ `stack=1/types[0]=19/uses[0]=1/sum_bag=1`；证背包纯追加被建模 |
| 11 | `pure_pickup_flag_reflects_monotone_hit` | witness 单层 ⇒ `floors[0].pure_pickup==true` + `reason=="adopted"` |
| 12 | `f3_is_excluded_by_g1_g2` | f3 真数据 ⇒ `!pure_pickup` 且 reason ∈ {`monster_ents`,`step_arm_hazard_reachable`,`step_arm_hazard_on_entry`} |
| 13 | `monotone_floor_f3_hits_closed_form`（`#[ignore]`） | **任务书要求的那条**：`ok && !truncated && out_hp==4027`。实测真红（§9.2），保留为证伪记录 |
| A1 | `witness_floor_records_pure_pickup_flag` | f54 层 `pure_pickup==true`；f1 层 `false && reason 非空` |
| A2 | `witness_json_has_no_unknown_schema_drift` | `meta.schema=="xiyou/witness/v2"` + 字段声明序 + 类型 |

core 侧新增 `#[cfg(test)] mod monotone_probe_tests`（2 条）：`certified_monotone_pickup_reports_each_axis`
（32⇒hp200、29⇒atk1、26⇒yellow1、19⇒bag[(19,1)]、20⇒bag[(20,3)]、13/15/41⇒None；**证增益与 uses 全部来自采样**）、
`pickup_gain_add_assign_sums_axes`。

## 6. 变异自证（各红一次，失败面互不相同）

**变异 A：闭式漏算最后一颗药**（`grid.pickup` 里删掉 key 最大的那颗，Σ 与路线一起少一颗）：

```
f54: out_hp=2027 ok=false truncated=true ... stat{reason=replay_pickup_left, pickups=17,
     sum_delta=2200, hp_expected=3027, hp_replayed=0}   ← 3227−200
panicked: assertion `left == right` failed  left: "replay_pickup_left"  right: "adopted"
```

两层证据：① `hp_expected` 精确变成 `3227−200`（口径跟随）；② **回放抓住**——路线掠过那颗药的格子
⇒ 引擎照样捡走 ⇒ 终态还留着拾取实体 ⇒ `replay_pickup_left` ⇒ fail-closed，`out_hp` 回落到
`2027/truncated`（**不会**把 3027 当答案交出去）。这条同时证明了「回放不是装饰」。

**变异 B：跳过回放验证 + 不连最后一段到出口**（凭空合成「到达出口且血对上」的终态）：

```
test pure_pickup_closed_form_hits_theoretical_max ... ok        ← 判据1 照样绿（假声明"完美"）
test pure_pickup_route_replays_in_engine ... FAILED             ← 判据2 抓
test pure_pickup_chain_breaks_the_f54_wall ... ok               ← 整条 solver run 也照样绿（黑盒看不出来）
  replay: hp=3227 pos=(4, 10) steps=50   left: (4, 10)   right: (7, 1)
```

结论：**层目标（hp/exit）与 `solver run` 都不足以守住路线真实性**，判据 2 的独立引擎回放是唯一防线。
这也是把「回放」写进产品代码（而非只在测试里）的理由。

## 7. 实测（f54 前后）

环境：release，`SOLVER_LAYER_LIMIT=200000`（=默认层界）。

### 7.1 `profile_floor 54 200000`（层隔离）

| | 旧（`cabb1b3`） | 新 |
|---|---|---|
| prefix f51..f53 | 3.08s | 3.18s（走同一通用搜索，未变） |
| f54 isolated | `ok=false truncated=true out_hp=2027` 23 waypoint **5.82s** | `ok=true truncated=false out_hp=3227` 63 waypoint **0.01s** |
| witness chain（4 层上限） | `status=capped blocked=floor 54 reason=truncated` 0.03s | 4 层全成 `status=blocked reason=max_floors` 0.02s |
| 总墙钟 | **8.99s** | **3.23s** |

### 7.2 `./target/release/solver run --out …`

| | 旧 | 新 |
|---|---|---|
| `status` | `capped` | `capped` |
| `floors` | **3** | **6** |
| `total_steps` | 196 | 275 |
| `blocked` | floor 54 / `truncated` | **floor 3** / `truncated` |
| 墙钟 | 9.23s | 8.10s |

新链逐层：f51 247/69、f52 427/102、f53 827/25、**f54 3227/63**、f1 3227/6、f2 3227/10。
f54 之后换轮回到 f1（hp 保持 3227、waypoint 6/10），f1/f2 由**通用搜索**走通（**不是**闭式：
hp 与入场同值；f1 的闸门理由实测 `monster_ents/41`）。
⇒ **f54 不再是墙；新的撞界点是 f3**（f3 在 200k 内 `truncated=true`，`out_hp=3827 只作参考`）。

> 【2026-11-19 P6 更正】本节原写「f1 的闸门理由实测 `monster_ents/41`」——**在链上入场格 `(6,11)`
> 下实测是 `step_arm_hazard_on_entry/83`**（`ty83` 门在入场格，闸门执行序 G2a 先响，G1 轮不到）。
> 拒绝理由**依入场格而变**：把 entry 换成该层 `start` 时 f53 从 `step_arm_hazard_reachable/83` 变成
> `monster_ents/51`、f54 从命中变成 `pickup_unreachable/0`。引用理由时必须同时写出入场格。

### 7.3 全层扫描：P6 有没有带来新的闭式命中（答案：没有）

临时探针 `crates/game-solver/examples/scan_floors.rs`（未入库，验收前删除）：loader 与判据同源，
入场格用 `landing()`（逐字复刻 `examples/profile_floor.rs:352` 的 `landing_replica`；复算 f51..f54 落点
`(1,11)/(2,11)/(6,10)/(7,9)` 与链上一致 ⇒ 复刻可信），`SOLVER_LAYER_LIMIT=1` 让通用搜索 1 节点秒退
（闭式在构造 `Search` 之前尝试），逐层跑 `solve_local_exit_toward(inp, entry, hp=1000, flags=0, None, None)`：

| 层 | 结果 |
|---|---|
| f1..f50 | **全部回退**，无一层满足 G1/G2/G2a |
| f51/f52/f53 | `step_arm_hazard_reachable/83`（与链上一致） |
| f54 | **HIT**：`out_hp=3400`、`exit=(7,1)`、`picks=18`、`sum_hp=2400`、`sum_atk/def=0`、`keys=(0,0,0)`、`bag=0` |

结论：**P6 的语义推广在现有 55 层数据上没有产生任何新的闭式命中**。f54 的 18 枚拾取物**全是 HpUp**
（`ents_in` 里只有 `ty32×10` 与 `ty31×8`，`+200×10 + 50×8 = 2400`），P5 的窄证书已完整覆盖它；
其余层与「拾取物是哪种类型」无关，是被「无怪 / 无危险步臂」这两道闸挡住的。
P6 真正多出来的是**断言面**：f54 的 atk/def/三钥/背包现在也有期望值并被逐字段比对。

### 7.4 P6 回归对照（与父提交 `0a244ff` 逐字段）

`/tmp/w20_base.json` = 父提交产物、`/tmp/w20_b.json` = P6 之后，同一命令
`./target/release/solver run --max-floors 20 --out …`：

| | 父 `0a244ff` | P6 |
|---|---|---|
| `status` / `floors` / `total_steps` | `capped` / 6 / 275 | **完全相同** |
| `blocked` | floor 3 / `truncated` / `layer_ok=false` | **完全相同** |
| 6 层 × 15 个共字段（floor,entry,exit,kind,route,in_hp,out_hp,in_player,out_player,in_flags,out_flags,ents_in,ents_out,ok,truncated） | — | **逐字段相同**（脚本投影比对） |
| `meta.schema` | `xiyou/witness/v1` | `xiyou/witness/v2` |
| 新增键 | — | `floors[*].pure_pickup` / `floors[*].pure_pickup_reason` |

逐层（P6）：f51 300→247 `[2,2]` 68 步 `step_arm_hazard_reachable/83`；f52 247→427 `[11,11]` 101 步 同；
f53 427→827 `[6,1]` 24 步 同；**f54 827→3227 `[7,1]` 62 步 `pure=true adopted`**；
f1 3227→3227 `[11,1]` 5 步 `step_arm_hazard_on_entry/83`；f2 3227→3227 `[1,11]` 9 步 `step_arm_hazard_reachable/83`。

## 8. 失败/回退路径清单（什么情况回退、有无静默风险）

| 情形 | 行 | 是否可见 | 静默风险 |
|---|---|---|---|
| 非 OFF 闭包策略 | `pure_pickup.rs:435` | `reason=policy_not_off` | 无 |
| 无层/入场越界 | `:432-445` | 具名 | 无 |
| 入场 hp ≤ 0 | `:446` | `entry_hp_not_real` | 无 |
| 入场格有陷阱/宝箱/组门/关着的门 | `:475` | `step_arm_hazard_on_entry` + `reason_ty` | 无 |
| 危险步臂格可达 | `:489` | `step_arm_hazard_reachable` + `reason_ty` | 无 |
| 层内有怪 | `:503` | `monster_ents` + `reason_ty` | 无 |
| 拾取物不纯/同格多枚/在入场格/格不纯 | `:514` | 四种具名 | 无 |
| **P6** 拾取效果非单调（任一轴下降 / 背包非纯追加 / gold·装备·教学位变化 / 零增益） | G3′ 段 | `pickup_not_certified` + `reason_ty` | 无 |
| **P6** 背包放不下全部可达拾取（`n0+need > 32` 或超过 `item_types` 长度） | G3′ 段 | `bag_no_room`（rank 2） | 无 |
| 层内无认证药 | `:544` | `no_certified_pickup` | 无 |
| 无 Stair 出口可达 | `:556` | `no_stair_exit_reachable` | 无 |
| `hp+Σ` 溢出（i32） | `:551` | `hp_overflow` | 无（**不能**用 `?` 早退：会让 stat 停在上一次快照 ⇒ 静默） |
| 路线构造失败（4 种顺序全败） | `:573`（`last` 汇总）/`:689`（`give_up`） | `route_construct_failed`/`pickup_unreachable` | 无 |
| 回放任一项不符 | `:606-692` | **P6 起 11 种** `replay_*`：`replay_stat_mismatch`（atk/def）/`replay_key_mismatch`（三钥）/`replay_bag_mismatch`（背包）/`replay_side_effect`（gold）/其余 7 种沿用 | 无 |
| **已知缺口**：`solve_local()`（旧公开入口）返回的 `SolveResult` 没有 `pure_pickup` 字段 | — | 见 `pure_pickup_stat()`（同线程可读） | **有**：只看 `SolveResult` 的消费方必须额外读 stat 才知道是否闭式 |

所有回退都**不修改**任何共享状态（`LAYER_TRUNC`/`GOLD_PEAK`/`SHOP_BUYS`/`PillDomStat` 在闭式路径里
只在**成功时**写），所以回退后通用搜索的行为与基线逐字段相同（判据 6 实测）。

## 9. 未查清 / 假设

1. **`(7,10)` 那个出口被闭式「结构性排除」，不是「未证明」**（2026-11-19 复核黄 F1 更正）。
   纯图禁止再进入场格（`pure_pickup.rs` 的 `step_ok`：`cell != entry`），`(7,10)` 的唯一邻格就是
   入场格 `(7,9)` ⇒ 它**永远进不了闭式候选出口集**（`exits.retain(...)` 一处过滤，见
   `pure_pickup.rs` G4 段）。因此：
   - 闭式**只覆盖** `(7,1)` 这条边，`out_hp=3227` 是「出口 `(7,1)` + 全捡」的值；
   - `(7,10)` 那条边的出层最优值**本模块不计算、不声称**（旧通用搜索实测能给出
     `2227/26/(7,10)`，那是**搜索**的答案）；
   - 这不是「已证明 `(7,10)` 不可达」——引擎里它是可达的，只是**必须踩入场格**；
     而入场格的步臂只有「门已开的场景门」被 G2a 证过中性 ⇒ 允许重入等于把未建模的踩格臂
     塞回路线、把上界变成猜想。**目标集合收缩是刻意的取舍**（宁小勿假）。
   - 若要覆盖它，正确做法是给「重入入场格」补一条独立的步臂中性证明（G2a 的推广），
     而不是把 `cell != entry` 删掉。本片**不做**这件事。
2. **【2026-11-19 P6 已查清 → 任务书前提被证伪】f3 不是纯拾取层，闭式命中 `out_hp=4027` 不可能成立。**
   实测（A 级，两条独立路径：链上 `solver run` 与隔离进程 `examples/probe_f3`）：
   - f3 = 13×13、`entry=(2,11)`、`start=(1,11)`、35 实体，其中 **`cat8` 怪 7 个**：
     `ty45@(1,3)`、`ty45@(10,8)`、`ty44@(1,7)`、`ty43@(10,2)`、`ty43@(3,5)`、`ty41@(7,5)`、`ty42@(8,10)`
     ⇒ **G1 结构性失败**；
   - 另有入场格 `ty83@(2,11)`（param=0，门**未开** ⇒ G2a 失败）与可达的 `ty83@(5,11)`（param=27）
     ⇒ **G2 也失败**。闸门执行序 G2a→G2→G1 下实测 `reason=step_arm_hazard_reachable`（`ty=83`）、
     `entry_hazards=1`；
   - 即使假装这三道闸不存在，**HP 上界也不可能是 4027**：f3 有 800 hp 的拾取物，但全捡必然要战斗，
     `ty41`（hp35/atk18/def1）对 `atk10/def10` 的 `predict_battle_hp_loss` = `ceil(35/9)=4` 击 ⇒
     `(18-10)*3 = 24 hp > 0`（还有 6 个更强的怪，`ty43` atk38 对 def10 一击就 28）。
     「每枚增益非负」这条定理前提在 f3 上**不成立**：层内存在必然的非拾取代价。
   - 实测数字：链上 `solver run` 到 f3 是 `truncated` 且 `out_hp=3827 只作参考`（通用搜索的截断值，
     **不是最优、也不可消费**）；隔离探针用 `entry=(2,11)`、不带 toward 跑
     `solve_local_exit_toward` 得 `ok=false truncated=true out_hp=3753 exit=(11,11) route_len=34`。
     两个数不同源于入场/优先目标不同，**都不是 4027**。
   - 因此判据 13（`monotone_floor_f3_hits_closed_form`）标 `#[ignore]` 并保留原断言，跑
     `cargo test -- --ignored` 会**真红**（`[B13] f3 ok=false trunc=true out_hp=3753 pure=false
     reason=step_arm_hazard_reachable` → `任务书要求 f3 出解且不撞界`）。绿色替代是判据 12
     `f3_is_excluded_by_g1_g2`（断言被 G1/G2 结构性排除，而不是「慢」）。
   - 下一个真撞界点仍是 f3，但**原因不是血量算不出来，而是它需要建模战斗**（P6 的定理明确把
     「层内有怪」排除在外）。
3. **引擎进图链的出生格步臂**：A 级读到 `scene.rs:3878-3887` 的注释与 `apply_entry_step_effects` 实现，
   但**求解器的 `build_scene` 不走这条链**——这是既有求解器的建模选择，本片没改，也没为它补证据。
   G2a 是**绕开**这个建模差异（不放行未证中性的入场臂），不是修好它。
4. **`probe_gate=true ⇒ 脚本不跑`**：来自 `gate_arrive` 的「判门未开⇒写 memo 后 Pass / 门开⇒Pass」实现，
   加上上一片对脚本 64 的逐 token 结论（`CES/TAK/DES`，不动 hp）。**B 级**（代码直读 + 上一片脚本表），
   没有为「门已开时脚本一定不跑」单独做一次探针。
5. **Cloud/UseItem 出口不进闭式**（G4 只认 `Stair`）：船/符的落点语义未建模。
6. 合成层（判据 3/4/5 的夹具）用 f54 的 `enemies`/`enemy_base` 表克隆，只是为了让通用搜索在回退后
   不越界；夹具的可走区只有 5 格，**不代表真实层几何**。

## 10. 复现

```bash
cargo test -p game-solver --release --test pure_pickup            # 13 条判据（+1 ignored 证伪记录）
cargo test -p game-solver --release --test pure_pickup -- --ignored monotone_floor_f3_hits_closed_form   # 看 f3 真红
cargo test -p game-core --release --lib monotone_probe            # 探针 2 条
cargo test -p game-solver -p game-core                            # 全量（debug）
python3 gates/cli.py check                                        # 总闸 PASS=13
cargo run --release -p game-solver --example profile_floor -- 54 200000
./target/release/solver run --out /tmp/x.json                     # status/floors/total_steps
```

旧基线对照：`git worktree add /tmp/dsp-pure-old cabb1b3 && ln -s <repo>/assets /tmp/dsp-pure-old/assets`，
`CARGO_TARGET_DIR=/tmp/dsp-pure-old-target cargo build --release --manifest-path /tmp/dsp-pure-old/Cargo.toml -p game-solver --bin solver`。
