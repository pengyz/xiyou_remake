# 精确拾取 DP（P9）：把闭式解推广到「拾取集」，打穿 f3

日期：2026-11-19　实现：`crates/game-solver/src/exact_pickup.rs`　判据：`crates/game-solver/tests/exact_pickup.rs`
上一片：[纯拾取层闭式解](2026-11-19-pure-pickup-closed-form.md)　同期：[hp 上界闸门](2026-11-19-hp-bound-p8b-spawn-gate.md)

## 1. 结论（A 级，可复现）

* **f3 打穿**：`exact_dp_f3_reaches_4027` 断言 `out_hp = 4027 = 3227 + 800`（18 件拾取全拿），
  `exit = (11,11)`（上梯），`steps = 106`，`picks/doors/kills = 18/5/7`，`states = 130835`，
  真引擎逐动作回放终态与 DP 预测**逐字段相同**，`truncated = false`。**4027 可达**，不是上界巧合。
* 链路：`solver run` 由 `floors=6 blocked=floor 3` 推进到 `floors=8 blocked=floor 5`（f3、f4 解开，f5 truncated）。
* f51..f54 四条 floor 记录与改动前 `/tmp/before.json` **逐字节相同**（本片不触碰通用搜索）。
* 覆盖面（上界口径，见 §6）：54 层里 **8 层**被精确解接管。

## 2. 为什么这是「精确」而不是「搜索」

闭式解的定理是：层内 hp 只可能被「拾取物的加血」改变（外加金币/钥匙/atk/def 的单调收益），
于是 `out_hp = hp0 + Σ已捡`。f3 的难点不是「谁更优」，而是**18 件拾取物的子集组合**：
掉一件就少 50/200 血，而搜索必须遍历海量无关排列（1M 节点仍 truncated）。

P9 把「已捡集合」升成状态维：在 `(cell, 已捡位图, 已开门位图, 已杀位图)` 上做**分层 BFS**，
每个状态只保留一次，目标是在所有「到达出口格」的状态里取 `hp` 最大。这不是启发式：
状态键包含所有**会影响后续可达性/收益**的量，且下面 §4 的支配引理保证剪掉的只有严格劣态。

## 3. 安全前提（fail-closed，全部满足才启用，否则 `Off(理由)`）

消费面**复用 game-core 的既有判定**，本模块不新增规则表：

| # | 前提 | 复用的判定 | 反例理由字面 |
|---|---|---|---|
| A1 | 层内 cat8 怪每只 `predict_engagement(no_counter=true)` 损血 0 | `solve_local::bump_outcome`（内部 `game_core::combat::BattleResolver::resolve_bump` + `predict_engagement`） | `monster_damage` |
| A2 | 无**伤害型**步臂（陷阱 `ty60/61/62`、宝箱 `ty5`、组门 `ty76`）可达 | `pure_pickup::hazard_gate`（G2′ 入场臂 + G2 几何可达，**与闭式解同源**，本片把两段闸门抽成共用函数） | `step_arm_hazard_on_entry` / `step_arm_hazard_reachable` |
| A3 | 无非一次性门（槽 32）；门脚本不得写 hp / 不得生成 hp 实体 / 不得有未知 op | `game_core::rules::gate::gate_slot_is_one_shot` + `hp_bound::plan`（1b/2b/2c） | `retriggerable_gate_slot` / `script_hp_spawn` / `script_op_unknown` |
| A4 | 无「击杀后生成实体」通道 | `hp_bound::plan` 的 `post_kill_hp_spawn`（`drop_gain` 递归 4 层） | `post_kill_hp_spawn` |
| A5 | 拾取物 ≤ `MAX_PICKUPS`（20） | 本模块常量 | `too_many_pickups` |
| A6 | 场景门（`ty83`）踩上**恒等**（真引擎逐扇采样） | `solve_local::exact_gate_effect`（`Search::gate_arrive` 的 `GateArrive::Pass` 分支） | `gate_mutates_state` |
| A7 | 无商店臂、拾取物必须是 `certified_monotone_pickup` 且不改背包 | `game_core::effect_registry::certified_monotone_pickup` | `shop_present` / `pickup_uncertified` / `pickup_not_gain` / `pickup_adds_bag` |

A5 的 20 是**性能护栏**（状态数 `2^k × 格子数`），不是正确性假设：`k` 再大结论仍成立，只是算不动。
f3 是 18 件 ⇒ `2^18 × 46` 量级，实测 130835 个存活状态（支配剪枝后），8.4s。

`MAX_PICKUPS`/`MAX_DOORS`/`MAX_KILLS` 同时是**状态键位宽的碰撞安全边界**（`u128`：
`pos(16)|pm(24)|dm(16)|km(16)`），不是「够用就行」的经验值——超限直接 `too_many_dims`。

## 4. 支配引理（本片唯一的「剪枝」）

同 `(cell, dm, km)` 下，若 `pm₁ ⊇ pm₂`，则 `pm₂` 被 `pm₁` **严格支配**：

1. 可走格集只由 `dm/km`（门/怪是否已清）决定，与 `pm` 无关；
2. `hp/atk/def/三钥` 逐轴 ≥（拾取收益单调，闭式解已证）；
3. `gold` 只由击杀赏金决定（`pm` 无关）；
4. 照抄 `pm₁` 的后续路线，`pm₂` 只会多捡、不会少捡。

故 `pm₂` 的任何延伸都不是最优 ⇒ 可安全丢弃。实现是 `front[(pos,dm,km)]` 上做子集检查。
**实证教训**：第一版把方向写反（把超集当被支配），症状是 f3 只出 3227/263 states/门全未开——
这类「静默变劣」只有 `out_hp` 判据能抓，故判据 1 必须断言**精确数值**而不只是「ok」。

## 5. 防线：真引擎回放（唯一可信度来源）

DP 的每一步都通过引擎口径的**同一个函数**推进（`exact_bump`/`exact_open_door`/`exact_gate_effect`），
但求解器自建的群众状态仍可能与引擎分歧。故每条 goal 的完整 route 都喂 `exact_replay_route`
（内部 `Search::pickup_step` → 真 `try_step` + tick + 回读），逐字段核对
**player（含背包规范形）/ flags / 实体表（多重集）/ 位置 / 步数 / 击杀数**。
任何一条不等 ⇒ `Off(replay_mismatch)` 并在 stderr 打印，**绝不静默降级**。
判据 3 用篡改 DP 预测的夹具证明这条防线真的会响。

## 6. 覆盖面与边界

`cargo run --release -p game-solver --example exact_coverage`（口径写在文件头）：
玩家固定 hp/atk/def=9999、`flags=0`、`entry=层 start` ⇒ **上界口径**（A1 几乎不响）。
结果：`adopted = 8/54`；拒绝直方图 `exit_unreachable 12`、`pickup_uncertified 8`、
`step_arm_hazard_reachable 6`、`gate_mutates_state 4`、`pickup_adds_bag 4`、`post_kill_hp_spawn 4`、
`too_many_dims 3`、`kill_spawns_ents 2`、`no_stair_exit 2`、`retriggerable_gate_slot 1`。

统计口径的三点保留：强属性玩家 ⇒ 真实链路（atk=def=730、有钥匙、有背包）的覆盖率只会更低；
`flags=0` 会关掉一部分脚本分支；`entry=start` 不等于链路进入点。**这不是「54 层里 8 层可解」的
最终结论**，而是「这套闸门在当前属性下放行多少层」的可复现切片。

## 7. 与既有结论的关系（没有回头推翻它们）

* 子集支配的**负结果**仍然成立（无独立 hp 源时支配条件联立 ⇒ 已捡集合相同）：本片不是靠支配
  把 f3 压下来，而是**把子集升成状态维**后靠 BFS 的完备性 + 状态数本身可算。
* hp 上界剪枝在 f3 仍恒无效（零损血 ⇒ `bound ≡ 4027`）：本片不再依赖上界，`hp_bound::plan`
  在这里只当**闸门**（A3/A4）用，不当剪枝用。
* 闭式解仍是更便宜的一档，顺序 `pure_pickup → exact_pickup → Search` 保持不变：
  精确层永远不会被闭式解误判成「纯拾取层」之外的层（闭式解 G1 拒怪、G2 拒步臂）。
