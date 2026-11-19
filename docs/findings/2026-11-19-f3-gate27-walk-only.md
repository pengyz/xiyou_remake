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
| f35 | `param=24@(4,6) (5,6) (6,6)` | **撞门专用参数**（`!from_walk`）⇒ 真能力缺口 |
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
- 真收益面：f35 的 `param=24`（撞-only 臂）在真层数据上**真触发**（判据
  `gate_bump_fires_on_real_floor35`）；该层不在当前链上（链在 f3 截断），故未进 `run` 输出。

结论（更正后）：撞门在引擎里是**真路径**、求解器原来确实没有这条动作；P7 已补。但它
**救不了 f3**（27 号门只认走），也**不会**改 f51..f54。

---

## 6. 未查清 / 假设

1. **f3 真最优是多少**：未算。1M 仍截断；要闭式化需要新的支配关系（例如「拾取单调 + 战斗免费」的
   层判据，或按 `ents` 子集做松弛支配）。这是**下一片**的事，本文档不下结论。
2. **`flags[0]` 为什么是 0**：链上 f3 入场 `flags=0xe337be0000000000` 的 bit0 为 0，而 `param=27` 的门
   要求 `flag(0)`。推测（**C 级**）：`flags[0]` 是「脚本槽 0 已播」标记，而链上从 `(6,11)` 换轮进入 f3
   时并没有播过槽 0 ⇒ 该门这条分支在本局里永远不成立。未做逐 tick 取证。
3. **撞门在全 55 层里是否还有别的 `param`**：扫描只覆盖 `flags=0`、difficulty=0 的初态；带 flags 的
   组合未穷举（`param=32` 需要 `cross_floor_snapshot(23,12)`，本仓实现恒 false ⇒ 未出现）。
