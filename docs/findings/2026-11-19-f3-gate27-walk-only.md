# f3 的「门 27」真实语义 + 一条退役的证伪判据（P7）

> 日期：2026-11-19（P7 片）
> 起因：P6 复核的两条「黄」——F8（`#[ignore]` 占 test-ratchet 名额）、F10（真墙是脚本不是战斗）——
> 加上任务书 B 片的前提「本地搜索撞不开 f3 的门 ⇒ 进不去口袋」。
> 结论级别：本文所有数字都是**本仓引擎实跑**（A 级：`solver run` / `examples/probe_f3` / 单元判据），
> 不是从反编译文本推断的。行锚以提交时的文件为准。

---

## 1. 退役的判据（原文保留，不删主张）

`crates/game-solver/tests/pure_pickup.rs` 里曾有一条**从不运行**的测试（P7 之前）：

```rust
// ---------------------------------------------------------------------------
// 判据 13（B）：任务书要求的 f3 闭式断言——**前提为假**，保留为 ignored 记录
// ---------------------------------------------------------------------------
/// 为什么 `#[ignore]`：任务书假设 f3 是「纯加血层」，实测不成立——
/// ① G1：f3 有 **7 个 cat8 怪**（ty45@(1,3)、ty45@(10,8)、ty44@(1,7)、ty43@(10,2)、
///    ty43@(3,5)、ty41@(7,5)、ty42@(8,10)，`render_category(41..45)=8` 且无 per-floor
///    override）；② G2：ty83 场景门 @(5,11) 在纯图内可达；③ G2a：入场格 (2,11) 本身
///    就是 ty83。三者任一都足以让闭式回退，与「单调拾取」无关。
#[test]
#[ignore = "前提为假：f3 有 7 个 cat8 怪 + 可达/入场 ty83，且全捡必杀怪掉血 ⇒ 4027 非 HP 最优"]
fn monotone_floor_f3_hits_closed_form() {
    let si = load_si();
    let inp = floor_input(&si, 3);
    let ps = s0_spec(IN_HP_F3);
    let (ex, _hit) = solve_local_exit_toward(&inp, F3_ENTRY, &ps, 0, None, None);
    println!(
        "[B13] f3 ok={} trunc={} out_hp={} pure={} reason={}",
        ex.ok, ex.truncated, ex.out_hp, ex.pure_pickup, pure_pickup_stat().reason
    );
    assert!(ex.ok && !ex.truncated, "任务书要求 f3 出解且不撞界");
    assert_eq!(ex.out_hp, IN_HP_F3 + 800, "任务书要求 f3 = 3227+800 = 4027");
}
```

**为什么退役**（P7）：

1. `#[ignore]` 的测试**永远不跑**，但 `cargo test` 仍把它计入 "running N tests" ⇒ test-ratchet 的
   `total=1193` 里含这一条 ⇒ 基线虚高、且它冒充「覆盖」。`--list` 实测 14 条 = 13 跑 + 1 忽略。
2. 它的 `assert!` 是**任务书原文照抄**，不是真值断言；真正的结论（f3 不是闭式层）已由判据 12
   `f3_is_excluded_by_g1_g2` 覆盖。
3. 其中的「全捡必杀怪掉血 ⇒ 4027 非 HP 最优」只对 `atk/def=10/10` 的隔离探针成立（见 §4）。

**现在的替代物**（会跑，P7 新增）：`f3_is_not_closed_form_g2_first`
（`crates/game-solver/tests/pure_pickup.rs:791`），断言四件实测事实：
`probe_gate(f3, 0) == true`（**G2a 通过**）、`pure_pickup_stat().reason == "step_arm_hazard_reachable"`、
`reason_ty == 83`、`entry_hazards == 1`、`pure_pickup == false`。
其中第一条是 P6 复核 §1 的错误更正：入场格 `param=0` 的门**不是**关着的。

---

## 2. 门 27 的真实语义：**只认「走上去」，撞不开**（A 级）

`crates/game-core/src/rules/gate.rs:147`：

```rust
27 => gate &= from_walk && v.flag(0),
```

- 引擎两条触发路径（`crates/game-core/src/scene.rs`）：踩上格子 `apply_step_cell_effects` → case 83 →
  `try_run_scene(param, true)`；从邻格撞 `interact_with_cell` → case 83 → `try_run_scene(param, false)`。
- ⇒ **撞（`from_walk=false`）对 param 27 恒为 `false`**，与 flags 无关。撞门在物理上不可能触发脚本 27。
- 脚本槽 27 = `data/../level_scripts.txt` 第 28 行 = `TAK_113_117 TAK_133_135 DES_84_6_11 GIN_0_13`；
  里面没有「置 flags[27]」的指令（`TAK`=对话、`DES`=转场、`GIN`=给道具 13）。
  ⇒ 任务书判据「撞 `(5,11)` 后 `flags[27]==true` 且拿到 item 13」**两半都不成立**。
- 全层扫描（`examples/_scan_gates.rs`，未入库）里**撞门专用**的参数只有 4 个：
  `1`（`flag(0) && !from_walk`）、`2`（`flag(1) && !from_walk`）、`24`（`!from_walk && hp>0`）、
  `32`（`!flag(7) && !from_walk && cross_floor_snapshot(23,12)`）——**27 不在其中**。

---

## 3. 「入口口袋被封」被证伪（A 级：route 逐字）

f3 真实链态（`entry=(2,11)`、hp=3227、atk/def=730、gold=225、`flags=0xe337be0000000000`（**bit0=0**）、
bag[0]=(13,255)）跑 `solve_local_exit_toward`，200k 节点实测 route 前 6 步：

```
2,11 → 3,11 → 4,11 → 5,11 → 5,10 → 5,9 …
```

`(5,11)` 被**直接走上去穿过**。原因：`flag(0)=0` ⇒ `evaluate(27, ·, true).run == false` ⇒ 门不跑 ⇒
`allowed` 保持 true ⇒ 玩家走进去（碰撞/脚本都不拦）。`pure_pickup` 的 G2 把它当「危险步臂」拒绝，
那是**闭式模型的选择**（不建模脚本侧效应），不是几何墙。

⇒ 复核 F10 的方向（「真墙是脚本不是战斗」）在**闭式语义**上对；但把它读成「搜索进不去口袋、
需要撞门动作」是错的：**搜索从来不缺这条动作就能出门**。

---

## 4. 3827 不是 f3 的最优（A 级：同一进程，只改节点界）

同一入场态、只改 `SOLVER_LAYER_LIMIT`（**没有**为了让数字好看而调参）：

| 节点界 | `ok` | `truncated` | `out_hp` | route 长 | 耗时 |
|---|---|---|---|---|---|
| 20 000 | false | true | 3427 | 30 | 0.5 s |
| 200 000（默认） | false | true | **3827** | 34 | 4.7 s |
| 1 000 000 | false | true | **3877** | 42 | 24.8 s |

`out_hp` 随层界**单调上升**且三次都 `truncated` ⇒ 3827 是「200k 内搜到的最好」，**不是最优**。
对照：同一层用 `atk/def=10/10` 的隔离探针，20k 与 1M 都给 3753（战斗真有代价）。
差别只有一个原因——**链上 f3 的入场 atk/def 是 730/730**（f52 建塔层的加成被带到后面每层），
此时打 7 只怪近乎免费，「全拾取 + 全清怪」才是最优，远高于 3827。

⇒ 任务书 B 片的验收目标 `out_hp == 3827` 与 P6 复核的 3827 都建立在 10/10 的战损假设上，
**与真实链态不符**；且 f3 的真卡点是**节点界（状态爆炸）**，不是能力缺口。
按纪律（禁调 `node_limit`、禁为对数字调参）本片**没有**动层界。

---

## 5. 撞门动作的边界（如果将来要做）

全层扫描（`ty83` 实体，`flags=0`；三态 = 走上门 / 撞门 / 走上门且 `flags[0]=1`）：

| 层 | 撞门可触发（`bump=true`）的实体 | 备注 |
|---|---|---|
| f3 | `param=0@(2,11)` | 入场格；撞门没有意义（人已经在格上/相邻） |
| f35 | `param=24@(4,6) (5,6) (6,6)` | **撞门专用参数**（`!from_walk`）⇒ 动作覆盖（**不是**能力缺口，见 §7） |
| f51 | `param=40@(1,11)` | 链上层；`walk=true` ⇒ 走能触发（见下） |
| f54 | `param=64@(7,9)` | 链上层；`walk=true` ⇒ 走能触发（见下） |
| f15/16/20/36/41/42/48/50 | `5 / 28 / 6 / 29 / 13 / 14 / 38 / 16` | 都是「恒过」臂，撞门效果 = 跑脚本不移动 |

### 5.1【2026-11-19 P7 落地】撞门臂已加，且**不改变**链上输出（实测推翻上表旧推断）

本节初版推断「加撞门会改 f51/f54 输出」，**该推断已被实测推翻**，原地更正：

- 落地面：`solve_local.rs::Search::gate_bump`（撞门转移）+ `bump_only_gate_params`（预筛），
  展开点 `expand_inner` 里先于 `classify` 的独立臂；判据 `tests/gate_bump.rs`（5 条）。
- 预筛条件 = **撞能触发 ∧ 走上去不触发**（`evaluate(param,·,false).run && !evaluate(param,·,true).run`）。
  走能触发的门（f51 `p40`、f54 `p64` 皆 `walk=true`）**连候选都不进**，因此不产生任何新后继；
  即便进了候选，引擎 `try_run_scene(param,false)` 也会拒（f3 `p27` 即此例，见 §2）。
- 实测回归：`solver run --max-floors 30` 前后两份 JSON **字段级零差异**（`floors/total_steps/
  blocked/route/in_hp/out_hp` 全同，6 层逐层一致）。
- 动作覆盖面：f35 的 `param=24`（撞-only 臂）在真层数据上**真触发**（判据
  `gate_bump_fires_on_real_floor35`）；该层不在当前链上（链在 f3 截断），故未进 `run` 输出。
  **进度收益 = 0**：脚本槽 24 只有 `TAK_101_105`（纯演出），旗标 24 也没有任何别的门臂读它
  ⇒ 撞完 `pos/hp/ents` 全不变，只置一个无人读的位（§7 有全量审计表与机械判定）。

结论（更正后）：撞门在引擎里是**真路径**、求解器原来确实没有这条动作；P7 已补。但它
**救不了 f3**（27 号门只认走），也**不会**改 f51..f54。更准确地说：P7 买到的是**动作覆盖**
（引擎真路径现在走得到），**当前关卡表上零进度收益**——全 55 层里撞-only 门只有 f35 `param=24`
三格，而那层根本不需要它（§7）。【2026-11-19 A 片降级】

---

## 6. 未查清 / 假设

1. **f3 真最优是多少**：未算。1M 仍截断；要闭式化需要新的支配关系（例如「拾取单调 + 战斗免费」的
   层判据，或按 `ents` 子集做松弛支配）。这是**下一片**的事，本文档不下结论。
2. **`flags[0]` 为什么是 0**：链上 f3 入场 `flags=0xe337be0000000000` 的 bit0 为 0，而 `param=27` 的门
   要求 `flag(0)`。推测（**C 级**）：`flags[0]` 是「脚本槽 0 已播」标记，而链上从 `(6,11)` 换轮进入 f3
   时并没有播过槽 0 ⇒ 该门这条分支在本局里永远不成立。未做逐 tick 取证。
3. **撞门在全 55 层里是否还有别的 `param`**：扫描只覆盖 `flags=0`、difficulty=0 的初态；带 flags 的
   组合未穷举（`param=32` 需要 `cross_floor_snapshot(23,12)`，本仓实现恒 false ⇒ 未出现）。

---

## 7.【2026-11-19 A 片】52 扇 `ty83` 门全量审计（P7 复核红 F8 的处置与降级）

工具：`crates/game-solver/examples/audit_gates.rs`（证据工具，不参与门禁；
`cargo run --release -p game-solver --example audit_gates`）。判门用 `probe_gate_verdict`
（与搜索同一个 `StGate`，禁第二份 `GateView`），取格类用 `probe_cell_class`（与 `Search::classify` 同实现）。

### 7.1 列定义与读法

- **撞(0/A) / 走(0/A)**：`evaluate(param, view, from_walk)` 在两档上下文下的布尔结果。
  `0` = `flags=0, hp=1, 空包`；`A` = `flags=bit0..67 全 1, hp=1, 背包含道具 16`。
  两位分别是两档（`10` = 第一档 true、第二档 false）。**两档取并** ⇒ 该臂在本模型下**能否**打开的上界。
- **格类**：`blocked / move / door / monster / pickup / shop`（`Search::classify`）。
- **4邻可达**：门格的 4 邻是否有一格从 `level.start` 出发的几何 BFS 可达（`classify != blocked`）。
  撞门要站在邻格上，所以这是「这扇门能否被撞」的必要条件。
- **跨门可读旗标**：本门 `set_flags` ∪ `{param}`（脚本完成旗标）中，**存在 `param' != 本门 param`
  的门臂读它**的成员（机械判定：置该位会让某扇门某臂判决翻转，再剔除「只被自己读」的位）。
- **分类**：`both / walk_only / bump_only / neither`（按两档取并后的两臂可得性）。

### 7.2 总量

`GATES=52`，`TALLY={"both": 12, "walk_only": 21, "bump_only": 3, "neither": 16}`。

- **`bump_only`（撞-only）= 3 格，全部是 f35 `param=24`** ⇒ 与 P7 复核一致。
- **`neither`（两臂都打不开）= 16 扇**：其中一部分是 `blocked` 格上的 `from_walk` 臂
  （如 f41 `p25@(10,2)`、f2 `p1@(3,7)`），属**本模型下打不开**，不是「漏了撞门臂」。

### 7.3 三处结论

**(1) 「撞门臂有没有存在价值」——有，但当前只是动作覆盖。**

判定规则（可复用）：`bump_only ∧ 4邻可达 ∧ (脚本有非演出 op ∨ 跨门可读旗标非空)` 才算
**进度相关**。f35 `param=24` 三条全不满足：

- 脚本槽 24 逐字为 `TAK_101_105`（`game-core/tests/fixtures/level_scripts.txt`）⇒ 纯演出，
  无 `CES/MOV/ADD/GIN/RES/ROS/SWD` 等状态 op；
- 撞前撞后 `pos/hp/ents` 全不变（复核探针：`pos` 不变、`hp` 3227 不变、实体 28→28）；
- 只置 `flags[24]`，而 **flag 24 没有任何跨门读者**。

**(2) 【修正复核表述】不是「无任何门臂读 flag(24)」，而是「只有它自己读」。**

`crates/game-core/src/rules/gate.rs:73` 是 `let mut gate = !v.flag(param);` —— **每扇门都读自己
那一位**当「已经开过」的自守卫。所以：

- 机械扫描出的**自重读集合** `SELF_READ_FLAGS` = 52 扇门用到的 36 个位（含 24）；
- 真正的**跨门可读集合**只有 `{0, 1, 3, 7, 46, 47}`（读者：`0→{0,1,27}`、`1→{2}`、`3→{3,4}`、
  `7→{7,33}`、`46/47→{46,47}`）；
- 24 ∉ 跨门可读集合 ⇒ 撞 p24 **不可能**让任何别的门从关变开（它只能把自己关死，
  因为撞完脚本完成会置 `flags[24]`，`scene.rs:4834-4835`，此后 `!flag(24)` 恒假 ⇒ 一次性）。

复核的结论（零进度收益）**成立**；只有「无门臂读 flag(24)」这句需要改成上面的精确形式。

**(3) f35 不需要撞门（绕门可达性）。**

几何 BFS（`classify != blocked` 作可进，门格本身 Blocked，撞门不移动玩家）：
`f35 start=(1,11)`，出口 `(11,1)` 距离 **22**（复核给 23，差异来自实现按几何上界且未计门/怪代价）、
出口 `(5,11)` 距离 **24**；从 `(1,11)` 出发共 51 格可达 ⇒ **不撞任何门直接出层**。

### 7.4 全量审计表

| 层 | param | 格 | 格类 | 4邻可达 | 撞(0/A) | 走(0/A) | 分类 | 脚本 op | 跨门可读旗标 | 结论 |
|---|---|---|---|---|---|---|---|---|---|---|
| f1 | 19 | (6,9) | door | true | 0/0 | 1/0 | walk_only | `CES+MOV+TAK+MOV+TAK+MOV+TAK+MOV+TAK+MOV+TAK+MOV+DES+TAK+MOV` | — | 走臂可开（撞门臂被支配） |
| f1 | 67 | (6,11) | move | true | 0/0 | 0/0 | neither | `TAK` | — | 两臂在本模型下都打不开（死门） |
| f1 | 66 | (6,9) | door | true | 0/0 | 0/0 | neither | `TAK+SMS` | — | 两臂在本模型下都打不开（死门） |
| f2 | 1 | (3,7) | blocked | true | 0/0 | 0/0 | neither | `TAK+ROS+TAK+MOV` | `1` | 两臂在本模型下都打不开（死门） |
| f2 | 2 | (1,8) | move | true | 0/0 | 0/0 | neither | `TAK+MOV+DES` | — | 两臂在本模型下都打不开（死门） |
| f2 | 26 | (11,4) | move | true | 0/0 | 0/0 | neither | `TAK+DES+GIN` | — | 两臂在本模型下都打不开（死门） |
| f2 | 33 | (11,10) | move | true | 0/0 | 0/0 | neither | `TAK+DES` | — | 两臂在本模型下都打不开（死门） |
| f3 | 0 | (2,11) | move | true | 1/0 | 1/0 | both | `CES+MOV+TAK+CES+TAK+ROS+TAK+ROS+TAK+MOV+TAK+DES+LAY+ROS+RES+GUT` | `0` | 两臂可开（脚本文本有 CES/MOV 等状态op） |
| f3 | 27 | (5,11) | move | true | 0/0 | 0/0 | neither | `TAK+DES+GIN` | — | 两臂在本模型下都打不开（死门） |
| f10 | 3 | (6,6) | move | true | 0/0 | 1/0 | walk_only | `TAK+MOV+TAK+DES+CES+SWD+TAK` | `3` | 走臂可开（撞门臂被支配） |
| f10 | 4 | (6,2) | monster | true | 0/0 | 0/0 | neither | `CES+TAK+MOV+ROS+TAK+MOV+DES` | — | 两臂在本模型下都打不开（死门） |
| f15 | 5 | (9,1) | blocked | true | 1/0 | 1/0 | both | `TAK+MOV+DES` | — | 两臂可开（撞门被走臂支配） |
| f16 | 28 | (1,11) | blocked | true | 1/0 | 1/0 | both | `TAK+DES+TAK+GIN` | — | 两臂可开（撞门被走臂支配） |
| f20 | 6 | (6,6) | monster | true | 1/0 | 1/0 | both | `TAK` | — | 两臂可开（撞门被走臂支配） |
| f26 | 30 | (6,11) | move | true | 0/0 | 1/0 | walk_only | `MOV+TAK` | — | 走臂可开（撞门臂被支配） |
| f26 | 31 | (6,7) | door | true | 0/0 | 0/0 | neither | `TAK+DES+MOV+TAK+DES+TAK` | — | 两臂在本模型下都打不开（死门） |
| f29 | 7 | (3,10) | move | false | 0/0 | 1/0 | walk_only | `TAK+MOV+TAK+CES+TAK+DES+ADD` | `7` | 走臂可开（撞门臂被支配） |
| f29 | 32 | (3,2) | blocked | false | 0/0 | 0/0 | neither | `TAK` | — | 两臂在本模型下都打不开（死门） |
| f32 | 8 | (6,10) | move | true | 0/0 | 1/0 | walk_only | `ROS+CES+MOV+TAK+MOV` | — | 走臂可开（撞门臂被支配） |
| f33 | 9 | (10,5) | move | true | 0/0 | 1/0 | walk_only | `SWD` | — | 走臂可开（撞门臂被支配） |
| f35 | 10 | (5,10) | move | true | 1/0 | 1/0 | both | `CES+MOV+TAK+DES` | — | 两臂可开（脚本文本有 CES/MOV 等状态op） |
| f35 | 24 | (4,6) | monster | true | 1/0 | 0/0 | bump_only | `TAK` | — | **仅动作覆盖，零进度收益** |
| f35 | 24 | (5,6) | move | true | 1/0 | 0/0 | bump_only | `TAK` | — | **仅动作覆盖，零进度收益** |
| f35 | 24 | (6,6) | move | true | 1/0 | 0/0 | bump_only | `TAK` | — | **仅动作覆盖，零进度收益** |
| f35 | 34 | (4,7) | move | true | 0/0 | 0/0 | neither | `TAK` | — | 两臂在本模型下都打不开（死门） |
| f35 | 34 | (5,7) | move | true | 0/0 | 0/0 | neither | `TAK` | — | 两臂在本模型下都打不开（死门） |
| f35 | 34 | (6,7) | move | true | 0/0 | 0/0 | neither | `TAK` | — | 两臂在本模型下都打不开（死门） |
| f36 | 29 | (9,8) | blocked | false | 1/0 | 1/0 | both | `TAK+DES+GIN` | — | 两臂可开（撞门被走臂支配） |
| f38 | 11 | (2,6) | move | true | 0/0 | 1/0 | walk_only | `SWD` | — | 走臂可开（撞门臂被支配） |
| f39 | 18 | (6,4) | door | true | 0/0 | 0/0 | neither | `DES+CES` | — | 两臂在本模型下都打不开（死门） |
| f40 | 12 | (6,7) | move | true | 0/0 | 1/0 | walk_only | `TAK+MOV+TAK+MOV` | — | 走臂可开（撞门臂被支配） |
| f41 | 13 | (10,2) | blocked | true | 1/0 | 1/0 | both | `CES` | — | 两臂可开（脚本文本有 CES/MOV 等状态op） |
| f41 | 25 | (10,2) | blocked | true | 0/0 | 0/0 | neither | `CES` | — | 两臂在本模型下都打不开（死门） |
| f42 | 14 | (6,10) | move | true | 1/0 | 1/0 | both | `ROS+TAK+DES+TAK+DES` | — | 两臂可开（撞门被走臂支配） |
| f48 | 38 | (4,1) | blocked | true | 1/0 | 1/0 | both | `CES` | — | 两臂可开（脚本文本有 CES/MOV 等状态op） |
| f49 | 15 | (6,8) | move | true | 0/0 | 0/0 | neither | `ROS+CES+TAK` | — | 两臂在本模型下都打不开（死门） |
| f50 | 16 | (6,5) | monster | false | 1/0 | 1/0 | both | `TAK+GUT` | — | 两臂可开（撞门被走臂支配） |
| f51 | 40 | (1,11) | move | true | 1/0 | 1/0 | both | `TAK+SEE` | — | 两臂可开（撞门被走臂支配） |
| f51 | 41 | (3,10) | move | true | 0/0 | 1/0 | walk_only | `TAK+SEE+ROS+SEE` | — | 走臂可开（撞门臂被支配） |
| f51 | 42 | (6,11) | move | true | 0/0 | 1/0 | walk_only | `TAK+SEE` | — | 走臂可开（撞门臂被支配） |
| f51 | 43 | (7,10) | monster | true | 0/0 | 1/0 | walk_only | `TAK+SEE+TAK+SEE` | — | 走臂可开（撞门臂被支配） |
| f51 | 44 | (10,7) | move | true | 0/0 | 1/0 | walk_only | `TAK+ROS+SEE+ROS+SEE` | — | 走臂可开（撞门臂被支配） |
| f51 | 45 | (8,4) | move | true | 0/0 | 1/0 | walk_only | `ROS+SEE+ROS+SEE` | — | 走臂可开（撞门臂被支配） |
| f51 | 46 | (5,4) | move | true | 0/0 | 1/0 | walk_only | `SEE+ROS+SEE` | `46` | 走臂可开（撞门臂被支配） |
| f51 | 47 | (6,1) | move | true | 0/0 | 1/0 | walk_only | `ROS+SEE` | `47` | 走臂可开（撞门臂被支配） |
| f51 | 48 | (2,4) | move | true | 0/0 | 1/0 | walk_only | `TAK+ROS+SEE+ROS+SEE` | — | 走臂可开（撞门臂被支配） |
| f52 | 49 | (3,11) | move | true | 0/0 | 1/0 | walk_only | `ROS+SEE` | — | 走臂可开（撞门臂被支配） |
| f52 | 50 | (5,10) | move | true | 0/0 | 1/0 | walk_only | `ROS+SEE+ROS+SEE+CES+SEE` | — | 走臂可开（撞门臂被支配） |
| f52 | 53 | (11,7) | monster | true | 0/0 | 1/0 | walk_only | `TAK+ROS+CES+SEE+CES+SEE+DES` | — | 走臂可开（撞门臂被支配） |
| f52 | 52 | (6,6) | move | true | 0/0 | 1/0 | walk_only | `TAK+SEE+TAK` | — | 走臂可开（撞门臂被支配） |
| f53 | 56 | (6,8) | move | false | 0/0 | 1/0 | walk_only | `TAK+MOV+TAK+DES+CES+GUT` | — | 走臂可开（撞门臂被支配） |
| f54 | 64 | (7,9) | move | false | 1/0 | 1/0 | both | `TAK` | — | 两臂可开（撞门被走臂支配） |

### 7.5 边界与已知低估（必须与表一起读）

1. **`ever` 不是「必开」**：两档上下文取并只证明「在本模型某组状态下能开」，不证明链上会开。
   `param=32` 两档皆假，因为本仓 `StGate::cross_floor_snapshot` 恒 `false`（见 §8.3 的契约判据）。
2. **`4邻可达` 用 `level.start` 作入口**：塔层（f51..f54）在链上是带 landing 偏移进入的，
   与本层 `start` 不同 ⇒ 该列对塔层会**低估**（例：f54 `p64@(7,9)` 报 `adj=false`，
   但链上 f54 的入口正好就是 `(7,9)`，该门在链上确实被走穿）。
3. **`neither` 不等于是死代码**：其中 16 扇里有 `from_walk` 臂但在 Blocked 格上，
   若将来把臂改成「格不可走时不预筛」会改变 f41/f48 的状态空间（见 §8.2），属行为变更。
4. 本表**不含** `ty1` 宝箱门（那是另一套 `door12/door13` 夹具）与 `MASTER` 层（55）。

### 7.6 对 P7 结论的最终裁定

> P7 的撞门臂买到的是**动作覆盖**（引擎真路径从「不可达」变成「可达且被搜索利用」），
> **当前关卡表上零进度收益**。因此它**不构成**「真能力缺口」；保留它的理由是
> **正确性/动作完备性**（一旦关卡表出现撞-only 且带状态 op 或跨门旗标的门，臂已经在位），
> 而不是「补上就能多拿血」。原「真能力缺口」表述在本节降级。

---

## 8.【2026-11-19 A 片】P7 复核黄 F1/F3/F4 + 变异 M5/M6/M7 的处置（含 `文件:行号`）

### 8.1 黄 F1 —— 三处静默丢弃全部具名计数（`crates/game-solver/src/solve_local.rs`）

| 原位置 | 原行为 | 现在 | 行号 |
|---|---|---|---|
| `try_run_scene(param,false)==false` | `continue` 无痕 | `rejected_not_fired += 1` | `:3105` |
| `ns.pos != st.pos` | `continue` 无痕 | `rejected_pos_moved += 1` | `:3116` |
| `Drive::Pending(_)` | `return None` 无痕 | `rejected_pending += 1` + `return None` | `:3127` |
| `Drive::Stalled` | 与 Pending 同臂 `return None` | `rejected_stalled += 1` + `return None`（拆臂） | `:3131` |

统计出口：`pub struct GateBumpStat`（`:3641`，字段 `candidates/script_hits/memo_hits/rejected_*`）、
`pub fn gate_bump_stat()`（`:3671`）、**旧口径投影** `pub fn gate_bump_stats() -> (u64,u64)`（`:3676`，
= `(candidates, script_hits)`，保留给既有判据，禁两套真相）。清零点：`solve_local`（`:296`）与
`solve_layer_with`（`:668`）都改 `GATE_BUMP_STAT.set(GateBumpStat::CLEARED)`。

**「pos 变化」的裁定（二选一）⇒ 降级为具名 `rejected_pos_moved` 计数，不上硬断言。**
理由（写进 `:3624` 起的 struct 文档）：撞门「不移动玩家」是**语义前提**，不是**数据结构不变量**；
数据面若出现 `MOV_0_*`（脚本传送）那是一次合法但本模型不覆盖的转移，硬断言会让求解器在生产
`panic`。改为计数后，可观测性由判据保住：`tests/gate_bump.rs::bump_stat_counts_every_rejection`
断言合成层的四类 `rejected_*` 全为 0，且真 f35 `param=24` 的 `rejected_pos_moved == 0`。
原 commit 里「断言 pos 未变」的表述**在此更正为「守卫跳过 + 具名计数」**。

### 8.2 黄 F3 —— 预筛不变量的定义域写清 + 反例做成判据

- 不变量**精确版**（`solve_local.rs:3029` 起 `bump_only_gate_params` 头注）：
  **在可走格上**，`walk` 能开 ⇒ 撞门臂被走臂支配。
- **定义域之外的反例**（已实测，判据
  `tests/gate_bump.rs::both_arm_gate_on_nonwalkable_cell_is_outside_the_dominance_invariant`）：
  | 层 | param | 格 | `bump` | `walk` | 格类 | 脚本原文 |
  |---|---|---|---|---|---|---|
  | f41 | 13 | (10,2) | true | true | `blocked` | `CES_6_10_2 CES_60_10_2` |
  | f48 | 38 | (4,1) | true | true | `blocked` | `CES_6_4_1 CES_61_4_1` |
  两格**不可走** ⇒ 走臂进不去；而预筛因 `walk=true` 排掉了撞门臂 ⇒ **两臂都没有**。
  两个脚本都是纯 `CES` 陷阱（只生成实体、不改 `pos/hp/flags`）⇒ 今日**零进度损失**，
  但「走能开 ⇒ 走臂支配」在没有可走性前提时**为假**，旧注释与旧判据不承重。
- 正确改法（**未做，属行为变更**）：`if !walk || !self.walkable(tx,ty)`；会改 f41/f48 状态空间，
  不在 A 片（A 片承诺零行为差异）范围。

### 8.3 变异 M5/M6/M7 的判据覆盖

| 变异 | 判据 | 打红机制 |
|---|---|---|
| M5 删 `try_closed_form` 里 `terminal_mismatch` 调用点 | `tests/pure_pickup.rs::closed_form_terminal_mismatch_call_site_is_wired` + `pure_pickup.rs:220` `TERMINAL_CHECKS` 计数器（`:815` 自增） | 删掉调用点 ⇒ 计数器不增 ⇒ 判据红 |
| M6 `StGate::cross_floor_snapshot → true` | `tests/gate_bump.rs::stgate_cross_floor_snapshot_is_false_by_contract` | 合成层 ty83 `param=32`（`flags[7]=0`）在契约下两面皆假；变异后撞面翻真 ⇒ 红 |
| M7 `StGate::predict_hp_loss_vs → 1` | `tests/gate_bump.rs::stgate_predict_hp_loss_is_zero_by_contract` | `param=34` + `hp=1`：契约 `loss=0 ⇒ 0>=1` 假 ⇒ 走面 false；变异 `loss=1 ⇒ 1>=1` 真 ⇒ 走面 true ⇒ 红（对照腿 `hp=0` 恒 true，证明判据非恒假） |

### 8.4 黄 F4 —— `BUMP_MEMO` 键补 `pos` 与门格 + 清理策略

- 新键 `struct BumpKey { floor, param, pos, gate, player, flags, ents }`（`solve_local.rs:3593`），
  `BUMP_MEMO` 类型随之改（`:3612`）。原键缺 `pos`/门格 ⇒ 同一层里玩家站在**不同邻格**撞
  **同 param、同状态**的门时会错误复用同一条缓存（脚本若含 `CES`/`ADD`，生成位置可能与门格相关）。
- 清理策略：`Search::run_root_with` 入口清空 `GATE_MEMO` 与 `BUMP_MEMO`（`:2533`/`:2534`）。
  两张表都是**纯加速**（丢缓存只影响速度，不影响结果）⇒ 每层一次 ⇒ 跨层不串味、内存有界。
- 判据：`tests/gate_bump.rs::bump_memo_key_includes_pos_and_is_cleared_per_layer`
  —— 从门格左右两个不同邻格各撞一次 ⇒ `bump_memo_len()==2`（**修前为 1**，即错误复用）；
  弄脏后再跑一层无门地图 ⇒ `bump_memo_len()==0`（清空生效）。
- 探针：`pub fn bump_memo_len()`（`:3788`）、`pub fn bump_memo_clear()`（`:3793`）。

### 8.5 A 片「零行为差异」的验证口径

A 片除 A2 的计数外**不应有任何可观测差异**，用两条独立证据把住：

1. `tests/gate_bump.rs` 原有 5 条 + 新增 5 条全绿（10 passed）；
2. f51..f54 逐字节回归（§9）。

### 8.6 A1 工具口径（`examples/audit_gates.rs`）

判门 `probe_gate_verdict`（`:3744`，复用 `StGate`）、格类 `probe_cell_class`（`:3766`，复用
`Search::classify`）——两个探针都是 A 片新加的 pub 入口，**目的就是禁止证据工具另写一份
`GateView`**（否则审计表与搜索行为可能漂移）。
