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

> 【2026-11-20 更正】本节三条主张被独立复核推翻，逐条更正（原主张保留在上方，不改写历史）：
>
> 1. **「精确最优 / 106 步 / 18 件全捡」不成立。** 复核 F4（`docs/findings/reviews/2026-11-19-exact-pickup-review.md`）
>    实测：删掉 `exact_pickup.rs` 的反向 dead 段后，同一层得 `hp=4027 steps=88 picks=14 doors=5 kills=7
>    states=4,741,000`，回放同样逐字段通过。根因是反向支配对 **steps 不健全**（见 §4 更正）：BFS 是 FIFO
>    单位步 ⇒ 子集态必然 steps ≤ 超集态，反向剪枝把「同 hp、更少步」的最优解杀了。
>    事实口径改为：**hp 最优 4027；步数 88 / 拾取 14 亦可达 4027**。判据 `exact_dp_f3_reaches_4027`
>    现断言 `(steps, picks) == (88, 14)`，另加 `steps <= 106 && picks <= 18` 的上界护栏；新增判据 1b
>    （`exact_dp_same_hp_fewer_steps_survives`）用最小夹具钉住「同 hp 更少步不被剪」。
> 2. **「链路推进到 floors=8」是同一条 bug 的下游产物。** 88 步路线只捡 14 件 ⇒ `out_player` 比 106 步
>    路线少 3 把黄钥匙与 1 点 def（`def 731→730`、`yellow 3→0`）⇒ f4 无黄钥匙、目标集里没有指向 f5 的解。
>    当前实测：`solver run` = `status=blocked floors=7 total_steps=363 blocked=floor 4 reason=no_exit_toward`。
>    即**旧链路 f4 的可解性依赖了非最优的 18 件路线**；逐层 `(hp, steps)` 目标与跨层钥匙需求冲突，本片未修（遗留项）。
>    【2026-11-20 定向复核补数】f4 **上行最少需要 2 把黄钥**（独立 python BFS：`yellow=0/1` ⇒ `hit=false`；
>    `yellow=2` ⇒ `hit=true`，88 步 / `out_hp=4327`；`yellow=3` ⇒ 77 步）。上面「少 3 把」是**旧路线相对新路线的
>    余量差**，不是 f4 的需求量；`0 < 2`，结论方向不变。
> 3. **「8/54 覆盖面」的口径与真实性都要收窄**（见 §6 更正）：5 层是 `picks=0` 的平凡层；真实链路只采用过 3 层
>    （f2/f3/f4），而 f2 的采用本身是**空商品表口径的产物**（见下一条）。
>
> 另两处未披露事实一并补上：`bin/solver.rs` 曾把真实 17 行商品表丢弃（`let (gi, _shops) = …`）⇒ witness 链一直
> 以空表求解，「打穿 f3」只在空表口径成立（已修，见 §8）；`gates/cli.py status` 实际是 **SKIP=1**
> （visual-diff）——**但这条本身是复核 F10 把两个命令混了**：`gates/cli.py check` 是 13 门
> （不含 reference-regression / diff-trace / visual-diff）⇒ 实测 `PASS=13 FAIL=0 SKIP=0`；
> `gates/cli.py status` 是 14 门（多这三门）⇒ 才有 `SKIP=1`。`ad5fece` 正文写的是 **check** 口径，
> 数字正确，不是错误记录（一次「更正」的更正，留在这里防再犯）。

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

> 【2026-11-20 更正】130,835 是**反向剪枝还在**时的数字，而那条剪枝不健全（§4 更正）。改成只留正向剪枝后，
> f3 实测 **4,741,000** 个存活状态、release 3.1 s / RSS ≈ 570 MB（debug 档算不动，只能 release 跑），
> 故 `MAX_STATES` 2M → 6M。`MAX_PICKUPS=20` 仍是性能护栏（不是正确性假设），但「20 件算得动」这句
> 只在正向剪枝 + 6M 预算下成立。

`MAX_PICKUPS`/`MAX_DOORS`/`MAX_KILLS` 同时是**状态键位宽的碰撞安全边界**（`u128`：
`pos(16)|pm(24)|dm(16)|km(16)`），不是「够用就行」的经验值——超限直接 `too_many_dims`。

## 4. 支配引理（本片唯一的「剪枝」）

同 `(cell, dm, km)` 下，若 `pm₁ ⊇ pm₂`，则 `pm₂` 被 `pm₁` **严格支配**：

1. 可走格集只由 `dm/km`（门/怪是否已清）决定，与 `pm` 无关；
2. `hp/atk/def/三钥` 逐轴 ≥（拾取收益单调，闭式解已证）；
3. `gold` 只由击杀赏金决定（`pm` 无关）；
4. 照抄 `pm₁` 的后续路线，`pm₂` 只会多捡、不会少捡。

故 `pm₂` 的任何延伸都不是最优 ⇒ 可安全丢弃。实现是 `front[(pos,dm,km)]` 上做子集检查。

> 【2026-11-20 更正】上面第 1–4 条只证明了 **hp 轴**不劣，**没有**证明 `(hp, steps)` 字典序不劣——
> 「任何延伸都不是最优」这句话是错的。反例（判据 1b 的最小夹具）：7×5 全通层，`ty32`(+200hp) 在
> 6 步直路上、`ty29`(仅 +1atk) 在 10 步绕路上；旧实现输出 `10 步 / 2 件`，正确答案是 `6 步 / 1 件`。
> 机制：`pm₁ ⊇ pm₂` 只保证「同一条后续路线上 hp ≥」，而**步数**是第二目标；反向 dead 段把
> 「同 hp、更少步」的子集态整棵子树砍掉了（FIFO 保证子集态先到、步数 ≤ 超集态）。
>
> 正确的支配谓词必须把 steps 纳入：`pm₁ ⊇ pm₂ ∧ steps₁ ≤ steps₂`。在 FIFO 的 BFS 里超集态必然
> `steps ≥` 子集态 ⇒ **反向剪枝恒空**。故实现改为**只保留正向剪枝**（先到的超集支配后到的子集，
> FIFO 同时给出 `steps₁ ≤ steps₂` 与 `hp₁ ≥ hp₂`，两条目标都不劣）。代价：f3 存活状态
> `130,835 → 4,741,000`，`MAX_STATES` 2M → 6M（release 3.1 s / RSS ≈ 570 MB）。
>
> 还记录一次**被实证证伪的中间方案**（避免后人重走）：曾改成「hp 严格更大 ⇒ 可杀」
> （`n_hp > hp₂ || (hp 相等 ∧ steps ≤)`），f3 得 `98 步 / 16 件`——同样不是最优。反例机制：支配者
> 站得高 hp，但被支配者剩余路线上还有 hp 拾取物，**两者到出口会打平**，而支配者步数更多。
> 凡「同出口 hp 打平」的层，这条静默给次优解；判据 1b 与 f3 的 88/14 都能抓它。
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

> 【2026-11-20 更正】上面 8/54 与整张直方图是**空商品表**口径的产物，且措辞需要三处收紧
> （原数字保留在上方，作为历史切片）：
>
> 1. **口径必须写明**：玩家 `hp/atk/def = 9999`、`flags = 0`、`entry = 层 start`。这是**属性上界口径**，
>    不是「真实可玩性」。任何「54 层里 N 层可解」式的转述都属于过度声明。
> 2. **平凡层必须点名**：旧 8 层里 5 层是 `picks=0 doors=0 kills=0 out_hp=9999 steps=10`
>    的**空层**（f2/f23/f24/f27/f28——走 10 步到出口，什么也不捡），它们不构成「精确解有用」的证据。
> 3. **真实链路只有 3 层被实际采用过**（f2/f3/f4），不是 8 层；而且 f2 的采用还是空表口径的产物。
>
> 用**真实 17 行商品表**重测（示例已改为 `solve_local_exit_toward_shops(…, &si.shops)`，与生产链同口径）：
> `adopted = 4/54`，`adopted` 明细只剩 f23 / f24 / f27（三者仍是 `picks=0` 平凡层）与
> **f36**（`picks=2 doors=0 kills=3 out_hp=10799 steps=15`，唯一非平凡）；直方图
> `exit_unreachable 9`、`shop_present 8`、`pickup_uncertified 8`、`step_arm_hazard_reachable 6`、
> `gate_mutates_state 4`、`pickup_adds_bag 4`、`post_kill_hp_spawn 4`、`too_many_dims 3`、
> `no_stair_exit 2`、`kill_spawns_ents 1`、`retriggerable_gate_slot 1`。
> 新增的 8 个 `shop_present`（f2/f6/f7/f12/f15/f28/f31/f39）里有 4 个（f2/f6/f12/f28）正是旧口径下
> 「被精确解接管」的层 ⇒ **旧的 8/54 高估了 4 层**，根因是 `shops` 表被丢弃（见 §8）。
>
> 【2026-11-20 定向复核更正】上面的『根因是 `shops` 表被丢弃』**不成立**：精确 DP 的 `shop_can_buy(param)`
> 只读 `param` 的 `idx>0 ∧ flags&2`，根本不读商品表；同一 f3 链态在空表与真表下都是 `adopted 88/14/4027`。
> 真根因是**旧判据 ty 盲**——`!shops.is_empty() ⇒ Off(shop_present)` 把「表里有没有商品」当成「这一格能不能买」，
> 于是把 4 个（f2/f6/f12/f28）按引擎语义不可买的商店格也放行了；当时测量又传了空表，两者叠加才有旧的 8/54。
>
> 修复后真实链路（atk=def=730、带钥匙/背包/商品表）里**只剩 f3 一层**由精确 DP 接管：
> f2 现在 `Off(shop_present, ty=78)`（`ty78@(11,7) param=0x201` ⇒ `flags&2 ∧ idx=1` 真能开价），
> f3 的 `ty77@(11,4) param=0x116`（`flags=1`）按机制放行 ⇒ `adopted`；f4 则因 f3 的新路线缺黄钥匙而不可达。

## 8. 【2026-11-20】复核整改记录（P9.1/P9.2）

独立复核 `docs/findings/reviews/2026-11-19-exact-pickup-review.md` 判红 1 / 黄 4 / 绿 5。四件事的整改：

* **红 F4（支配剪枝对 steps 不健全）**：删掉反向 dead 段，只留正向剪枝（§4 更正）。f3 由
  `106 步/18 件` 更正为 `88 步/14 件`（hp 4027 不变，回放逐字段通过）；`MAX_STATES` 2M → 6M。
  新增判据 1b 钉住「同 hp、更少步不被剪」。
* **黄 F3/F8（witness 链商品表恒空 + 未披露）**：`bin/solver.rs` 不再丢弃 `shops`（原
  `let (gi, _shops) = solve_global_input(1)`），`witness::run_witness(+_with)`/`solve_local_exit_toward_shops`
  一路透传真实 17 行表；新增判据 `witness_forwards_real_shop_table_to_layer_solver` 断言层求解器收到的
  指针就是 `si.shops`。同时把「表非空 ⇒ `shop_present`」换成**机制判据** `shop_can_buy(param)`：
  `idx>0 ∧ flags&2` 才 `Off`，否则按阻挡格（引擎撞商家格 `allowed=false`，`scene.rs:3543`）。
  判据 2g（可买 ⇒ `Off(shop_present) ty=77`）/ 2h（`param=0x116` 不可买 ⇒ `adopted`）。
  **未建模边界（如实披露）**：`flags&2==0 ∧ flags&1!=0` 的告示格，关面板后 `param ^= 256` 且实体被移除
  ⇒ 事后可通行；DP 没有「告示已读」维，仍按阻挡处理（只会低估可达性，不会高估收益）。
* **黄 F10（过度声明/未披露）**：见 §1、§4、§6 三处日期更正。**其中「SKIP=0 写错」这一条经实测撤回**：
  `gates/cli.py check` = `PASS=13 FAIL=0 SKIP=0`（11m23s，13 门，不含 reference/diff-trace/visual），
  `gates/cli.py status` = 14 门、`SKIP=1`（visual-diff 未就绪）+ `TOLERATED=0`。
  复核 F10 引的是 `status` 口径，原文写的是 `check`——两个命令，数字都对。
  教训：**引用门禁数字必须带命令名**，`check` 与 `status` 的门集不同。
* **附带更正（复核 F5 与本片自查）**：f2 的 `steps 9→10` **不是路线变化**，是**记录口径**差异——
  Search 系层的 `route[0]` 就是入场格本身（f51..f54/f1 全部如此：`f52 entry=(2,11)`、`route[0]="2,11"`），
  精确 DP 的 route 不含这个 token。实测 `after.route[1:] == before.route`（逐 token 相同）。
  复核报告里「f2 路线变了（首 token `1,2`→`1,3`）」其实是同一口径翻转：P9 前 f2 走 Search（10），
  P9 后走精确 DP（9），修复后又因真实商品表落回 Search（10）。**物理路径三次都没变。**
  遗留披露：`total_steps` 因此混了两种口径（精确层比 Search 层少 1 个入场 token），跨层求和只能同口径比。
* **黄 F6（失败判据空洞）**：`MAX_PICKUPS` 原有判据 2f（`too_many_pickups`/`reason_ty=21`）；
  新增 `exact_dp_state_cap_is_off_not_truncation`（预算压到 1 ⇒ `state_cap`，`reason_ty=1`）与
  `exact_dp_gate_budget_is_off_not_truncation`（门预算压到 1 ⇒ `gate_budget`，`reason_ty=2`），
  两者都断言 `Off` 且**不携带半成品统计**；把护栏改成 `Ok(None)`/`break`（静默截断）时两条都变红。

## 7. 与既有结论的关系（没有回头推翻它们）

* 子集支配的**负结果**仍然成立（无独立 hp 源时支配条件联立 ⇒ 已捡集合相同）：本片不是靠支配
  把 f3 压下来，而是**把子集升成状态维**后靠 BFS 的完备性 + 状态数本身可算。
* hp 上界剪枝在 f3 仍恒无效（零损血 ⇒ `bound ≡ 4027`）：本片不再依赖上界，`hp_bound::plan`
  在这里只当**闸门**（A3/A4）用，不当剪枝用。
* 闭式解仍是更便宜的一档，顺序 `pure_pickup → exact_pickup → Search` 保持不变：
  精确层永远不会被闭式解误判成「纯拾取层」之外的层（闭式解 G1 拒怪、G2 拒步臂）。
