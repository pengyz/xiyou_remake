# S6a 修复复核：`eb2359b`（`fbecc72` / `e42c26a` / `5792e01` / `d130206`）

**结论：有前提可合并**——F1 的精确 DP 终态判据现在**真的**能打红「floor 翻盘 hp」；F8 的 `steps` 有牙但 `picks/states` 钉成了枚举顺序指纹；F3 的具名诊断不误报可达、`ok/adopted` 未动，但边界与入口钥项无判据，且诊断会附着在 `give_up` 结果上；F6 两处标记都有牙、旧 boss 入口 `floor=None` 同路径，但 witness JSON 根本不落该字段、`Option` 的 `None` 语义会被 S6b 误读；F9 的 gold 1231 同值同源、ratchet 无掩盖删测，但 example 模块 doc 仍留旧值 225。**0 红 / 4 黄 / 1 绿（面级）；Confidence: 90/100**。

- 复核对象：`HEAD=eb2359b`；复核开始时主树 `git status --short` 空，落笔后仅新增本文件（`?? docs/findings/reviews/2026-11-23-s6a-fix-review.md`）。
- 复核基线（HEAD）：`exact_pickup` 16 passed、`objective` 13、`solve_local` 31、`outer_witness` 15、`witness` 9；报告落笔前在副本重跑 `exact_pickup` → `16 passed; 0 failed`（8.43s）。
- 变异纪律：全部在 `/tmp/s6arck`（`rsync` 副本，`CARGO_TARGET_DIR=/tmp/s6arck-target`）内做，原文件备份在 `/tmp/s6abackup2/`，用 `cp` 还原；**未用 `git checkout`/`--amend`，未在变异下提交**；主树只新增本文件。
- 除特别标注「lead 已复现、本复核未重跑」的 F1 终态序变异外，本文所有红绿均为本复核实跑原始输出。

---

## 1. 逐面判定表

| 面 | 判定 | 关键实证 | 合并前必须处理 |
|---|---|---|---|
| **F1** `s6a_exact_dp_hp_beats_key_floor` | **绿**（附 1 条黄级备注） | 终态序改 floor→hp→steps 会选出 A `hp500/keys1`（lead 已复现）；本复核 M1/M1b 证明 A/B 两终态真互斥、自证段是 A 可达的前提守卫；`atk900→730` 后结果逐字段不变 | `atk900` 不是真链观测值（真链最大 atk731、f3 入场 atk730）；建议改 730 或在注释声明合成属性 |
| **F8** `s6a_exact_dp_key_floor_f3` | **黄** | 反转 steps tie-break → `steps85/picks17`，`steps==83` 断言红；但旋转枚举顺序 `DIRS`（语义无关）→ `picks15/states3_556_732`，`picks==16` 红、`states==4_739_833` 红 | `picks==16` 改为 `>=15`（或 `>=14` + 满足 floor）；`states` 改区间 + 单独 `states(None)==states(Some)` 不变量，精确值若留须降级为「枚举顺序指纹」 |
| **F3** `key_floor_reason=Some("key_floor_unreachable_in_layer")` | **黄** | 恒报错变异 M5 被 `Some([2,0,0])⇒None` 打红（说明「不误报」有牙）；但 `<`→`<=` 的隐蔽变体 M6b、去掉入场钥项的 M7 都让 shipped 测试全绿 | 补 `[8,0,0]⇒None` 与「入场钥非零 + floor=上界⇒None」；`key_floor_reason` 只在 `adopted=true` 时可作精确解结论（或 `give_up` 时清空） |
| **F6** `key_floor_not_consulted` + `solve_local_boss_toward_shops_floor` | **黄** | 删 pure/boss 两处标记均红（M8/M9）；旧入口委托 `None`，`old == new(None)` 全字段相等（本复核 scratch）；改传 `Some` 后仅 marker 差异即被打红 | `Option<&'static str>` 的 `None`（消费了 floor / 没给 floor）对 S6b 有歧义；witness JSON 无此字段（schema 仍 v3）⇒ S6b 读 witness 看不到 boss 未消费；doc 声称的 `key_floor_not_consulted_by_path:<值>` 统计键全仓不存在 |
| **F9** gold 1231 / ratchet 1310→1318 | **黄** | 示例函数与判据同为 `gold=1231`，其余字段逐项相同，真链 `prefix_floors` index6/index10 的 `in_player.gold` 均 1231；ratchet 测试数只增不减、`#[ignore]` 全为 0、总闸 PASS | `crates/game-solver/examples/f3_bound.rs:4` 模块 doc 仍写 `gold=225`，与同文件 1231 自相矛盾 |

> 计数口径：面级 0 红 / 4 黄 / 1 绿；F1 的 `atk900` 是额外的一条黄级备注，不改变 F1 面级绿。

---

## 2. 变异记录（变异前 → 变异后）

| 变异 | 攻击点 | 变异前 | 变异后 | 说明 |
|---|---|---|---|---|
| M1 删 A 自证四断言 | F1 自证是否承重 | 绿 | **绿（主断言）** | 自证对终态序变异零贡献 |
| M1b 删 A 夹具的 K | F1 自证活性 | 绿 | **红（自证）** | 证明自证是 A 可达的前提守卫 |
| M2a atk 900→730 | F1 属性真实性 | 绿 | 绿（逐字段同） | 真链值可用 |
| M2b atk 900→10 | F1 属性下限 | 绿 | **红（adopted）** | 低 atk 撞旁路 cat ⇒ DP give_up |
| M3 反转 steps tie-break | F8 steps 牙 | 绿 | **红（steps85）** | 目标变异被抓住 |
| M4 旋转 DIRS | F8 picks/states 脆性 | 绿 | **红（picks15）** | 语义无关改动即红 |
| M5 恒报错 | F3 不误报 | 绿 | **红（[2]⇒Some）** | `[2]` 判据有牙 |
| M6 `<`→`<=` | F3 边界 | 绿 | 红（shipped，零轴 0<=0） | 粗变异被抓 |
| M6b `f[i]>0 && upper<=f` | F3 等值边界 | 绿 | **绿（漏）** | `[8]`、entry 等值未覆盖 |
| M7 去掉入场钥项 | F3 入口钥 | 绿 | **绿（漏）** | f3 入场钥 0，零覆盖 |
| M8 删 pure 标记 | F6 标记牙 | 绿 | **红** | 有牙 |
| M9 删 boss 标记 | F6 标记牙 | 绿 | **红** | 有牙 |
| M10 give_up 保留 reason | F3 诊断附着 | — | 诊断残留 | `adopted=false` 仍带 Some |
| 旧 boss 入口改传 Some | F6 旧入口 | eq=true | **eq=false**（marker 差异） | shipped 测试只查 marker/hit |

### F1：判据有牙，但自证段不承重

基线：

```
[S6a-F1] ok=true hp=700 keys=(0,0,0) steps=18 doors=1 picks=2 stat=ExactStat { attempted: true, adopted: true, reason: "adopted", reason_ty: 0, out_hp: 700, steps: 18, picks: 2, doors: 1, kills: 0, states: 102, replay_steps: 18, route_len: 18, key_floor_reason: None }
```

- **M1 删除 A 自证段的四条断言**（`adopted/hp500/yellow1/doors0`，保留主断言）→ **主断言全绿**，输出与基线逐字段相同。⇒ 自证段对「floor→hp 变异」的检测**不承重**，它不是主判据的一部分。
- **M1b 把 A 夹具的 ty26 K 拾取物删掉**（自证段保留）→ 主断言仍绿；自证红：

```
自证断言失败：A 终态带 1 把黄钥（满足 floor）
  left: 0
 right: 1
```

⇒ 自证段是「A 路线真的存在」的**前提守卫**，有活性；删掉它后，未来夹具退化（A 不可达）会静默，但当前它并不给主断言加牙。建议保留，并可加一条把前提与主断言绑定的对照（`ex_a.out_hp + 200 == ex.out_hp`、`ex_a.yellow == ex.yellow + 1`）。

- **M2a `ps.atk 900→730`**（真链 f3 入场 atk=730）→ 全绿，`ExactStat` 与 atk900 逐字段相同（`hp700/steps18/doors1/picks2/states102`）。⇒ atk900 不承重，可用真链值 730。
- **M2b `ps.atk 900→10`** → `ok=true` 但 `adopted=false reason="monster_damage" ty=41 out_hp=0 states=0`，红在：

```
接管断言失败：合成夹具必须真正落到精确 DP，reason="monster_damage"
```

机制：ty41 cat8 在 (6,4)，与出口 (6,3) 相邻；DP 对可达怪格的 `bump` 是 fail-closed（`loss!=0` ⇒ Err），低 atk 一撞就整层 `give_up`。所以夹具要求「能零损杀 cat」，否则连 DP 都不接管。真实链 f3 入场 `atk=def=730`，`atk=900` 高于所有真链观测（最大出层 atk=731）。

`picks==2` 口径与代码一致：`crates/game-solver/src/exact_pickup.rs:418-421` 是 `st.picks = pm.count_ones()`，`pm` 是终态拾取位图，一件最多一位，所以它数的是「终态捡过几件」，不是「拾取动作次数」；测试注释也写明黄钥花在门上但位仍置位。`doors==1` 排除绕路：地图 `row1 = S.K.DP#`，P(5,1) 上方、右方、下方分别是 `#`/`#`/`#`（`row2` 的 x5 是墙），唯一入口是 D(4,1)，而 hp700 只能来自 P 的 +200 ⇒ 选 B 必然开门；全层又只有这一扇门，故 `doors==1`。

> **附加观察（B 级代码推理，未另造夹具实测）**：`exact_pickup.rs:405-407` 的 `main = max_by_key((g.hp, -g.steps))` 是**跨出口**取统计投影，而 `solve_local` 随后用 `pick_exit_floor`（hp→gold→floor→steps）重挑要返回的 goal。多出口且 hp 相同时，`ExactStat.picks/doors/steps` 可能描述的不是最终 `ExitSolve` 的出口；F1/F8 的断言把 `st.picks` 与 `ex.out_hp` 配对，只在单出口（或恰好同出口）时成立。S6b 不能用 `ExactStat` 反推返回出口；建议让 `main` 与 `pick_exit_floor` 共用比较器，或给统计加 `exit` 字段并明确它只是「本层最高 hp 终态」的投影。

### F8：steps 有牙，picks/states 过脆

基线：

```
[S6a] f3 floor=Some([2,0,0]) hit=true ok=true trunc=false hp=4027 gold=1256 keys=(2,1,0) steps=83 exit=Some((11, 11)) exact=true reason="adopted" states=4739833 stat=ExactStat { ... adopted: true, steps: 83, picks: 16, doors: 5, kills: 7, states: 4739833, key_floor_reason: None }
```

- **M3 反转 steps tie-break**（`exact_pickup.rs:365-367` 的 `.then(sb.cmp(&n.steps))` → `.then(n.steps.cmp(&sb))`）→ **红**：

```
hp=4027 gold=1256 keys=(3,1,0) steps=85 ... picks: 17, states: 4739833
panicked at crates/game-solver/tests/exact_pickup.rs:883:
key_floor 只许精确 +2 步（81 = 无 floor 的最短；83 = 多捡 2 件的代价）
  left: 85
 right: 83
```

⇒ `steps==83` 确实能抓住这条目标变异（旧窗口 81..=88 会漏，实现者说法属实）。

- **M4 旋转 `DIRS`**（`exact_pickup.rs:223` `[(0,-1),(0,1),(-1,0),(1,0)]` → `[(1,0),(0,1),(0,-1),(-1,0)]`，纯枚举顺序改动）→ **红**：

```
hp=4027 gold=1256 keys=(2,1,0) steps=83 ... picks: 15, states: 3556732
panicked at crates/game-solver/tests/exact_pickup.rs:887:
floor=Some([2,0,0]) 必须比无 floor 多捡 2 件（14 → 16）
  left: 15
 right: 16
```

⇒ `steps83 / hp4027 / gold1256 / kills7` 稳定，但 `picks==16` 与 `states==4_739_833` 是**搜索枚举顺序指纹**，不是语义不变量。测试注释「states 是 DP 状态图规模（只由层决定、与 floor 无关）」前半句对、后半句「确定性可精确断言」在当前实现下等价于「任何 DIRS/剪枝改动都必须同步改测试」。

- 单独验证「states 与 floor 无关」这一半（scratch `tmp_states_floor_invariance`，f3 同层）：

```
[tmp-st] none ok=true hp=4027 steps=81 picks=14 states=4739833 | some ok=true hp=4027 steps=83 picks=16 states=4739833
```

`assert_eq!(states_none, states_some)` 通过（23.43s）⇒ 这一半成立，但 `picks` 的「必须 +2」不成立（M4 下 83 步只多 1 件仍满足 floor）。

### F3：不误报守住了「2」，没守住「8/入场钥」

基线 `Some([9,0,0])`：

```
[S6a-F3] ok=true trunc=false hp=4027 yellow=0 steps=81 key_floor_reason=Some("key_floor_unreachable_in_layer") stat=ExactStat { ... adopted: true, steps: 81, picks: 14, doors: 5, kills: 7, states: 4739833, key_floor_reason: Some("key_floor_unreachable_in_layer") }
```

上界代码（`exact_pickup.rs:311-324`）：`upper = [player_in.yellow + s[0], blue + s[1], red + s[2]]`，`s = dp.key_sums[last]`（全层 certified 拾取钥和），任一轴 `upper[i] < f[i]` 即报 `Some`，且在 `dp.search()` 之前设置。

**上界是否过保守（把可达但绕路误报成不可达）？不会误报；它是松上界，会漏报。** `upper` 是「入场钥 + 全层所有合法拾取钥」的**超集**：开门只减钥、撞怪要求 `delta==0`（掉钥立即拒）、门脚本 `mutate` 拒、商店能买钥时 DP 直接 `shop_present` 放弃（`solve_local.rs:4520-4528`）⇒ 不存在被上界漏计的钥源，所以不会把「可达但需绕路」判成不可达；它只会**漏报**真正不可达（例如钥都在 8 扇门后，上界仍够）。这对诊断可接受，但 S6b **不能**把 `None` 当作「已满足/可达」——`None` 只表示「没证明不可能」。

- **M5 恒报错**（`if (0..3).any(...)` → `if true`）→ **红**：

```
「不误报」断言失败：可达 floor 不许被误判成 unreachable_in_layer
  left: Some("key_floor_unreachable_in_layer")
 right: None
```

⇒ 对 `[2,0,0]` 的「不误报」有牙。

- **M6 `<`→`<=`** → shipped 测试红（blue/red 需求 0，`0<=0` 也触发）；scratch 里 `[8,0,0]`、entryY1+`[9,0,0]` 都被误报 `Some`。
- **M6b 隐蔽变异** `if (0..3).any(|i| f[i] > 0 && upper[i] <= f[i])` → shipped 测试**全绿**；scratch：`[8,0,0]→Some`（正确 None）、entryY1+`[9,0,0]→Some`（正确 None）。
- **M7 去掉入场钥项** `let upper=[s[0],s[1],s[2]]` → shipped 测试**全绿**；scratch：entryY1+`[9,0,0]→Some`（正确 None）。
- scratch 正确实现基线：`[8,0,0]→None`、`[9,0,0]→Some`、entryY1 `[9,0,0]→None`。

⇒ 「不误报」判据只覆盖了 `[2,0,0]` 这个远离边界的点；等值边界 `[8,0,0]` 与「入场钥非零」项都是零覆盖。f3 入场黄钥为 0，所以 M7 这种删掉入场项的退化在 shipped 测试里完全静默。

- **M10 `give_up` 残留诊断**（5×3 走廊中间 ty41、atk10、floor`[9,0,0]`）：

```
[tmp-gu] ok=true adopted=false reason="monster_damage" ty=41 key_floor_reason=Some("key_floor_unreachable_in_layer")
```

原因：`key_floor_reason` 在 `dp.search()` 之前设置（`exact_pickup.rs:314-324`），`give_up` 只写 `reason/reason_ty`、不清该字段（`exact_pickup.rs:144-148`）。⇒ 诊断会附着在 fallback 的 `ExitSolve` 上；消费方必须同时看 `adopted`，否则会把「DP 因撞怪放弃」误读成「floor 不可达」。

**`ok/adopted` 语义是否动了？** 未动。`fbecc72` 对该函数的 diff 只新增字段、`if` 赋值和 doc，没有改 `ok/reached_goal/truncated/adopted` 的控制流；`3c79018` 与 HEAD 在 `[9,0,0]`、f3 夹具上同为 `ok=true / adopted=true / hp4027 / y0 / steps81`（`3c79018` 记录见上一轮报告）。

### F6：标记有牙、旧路径同，但 witness 不落账

基线：

```
[S6a-F6] pure ok=true hp=700 pure=true reason=Some("adopted") marker=Some("pure_pickup")
[S6a-F6] boss hit=true ok=true marker=Some("boss")
```

- **M8 删 pure 标记**（`solve_local.rs:667`）→ **红**：

```
pure 标记断言失败：闭式不读 floor ⇒ 给了 floor 时必须显式标记
  left: None
 right: Some("pure_pickup")
```

- **M9 删 boss 标记**（`solve_local.rs:723`）→ **红**：

```
boss 标记断言失败：boss API 不读 floor ⇒ 给了 floor 时必须显式标记
  left: None
 right: Some("boss")
```

- **旧 boss 入口是否逐字节同路径（`floor=None`）**：函数体确实是搬移——旧 `(materialize_exit(...), hit)`，新 `let mut ex = ...; if key_floor.is_some() { ex.key_floor_not_consulted = Some("boss"); } (ex, hit)`，其余语句逐字相同；旧入口 `solve_local_boss_toward_shops` 现在委托 `..._floor(..., None)`。本复核 scratch 在 ty75 boss 夹具上直接比较 `ExitSolve`（derive `PartialEq`）：

```
[tmp-boss] hit true/true eq=true old_marker=None new_marker=None hp=500 route=4
```

把旧入口误改成传 `Some([1,0,0])` 后：

```
[tmp-boss] hit true/true eq=false old_marker=Some("boss") new_marker=None
```

⇒ 旧入口当前确实传 `None`；但 shipped 测试只查 `hit/ok/marker`，**没有全字段相等断言**，除 marker 外的漂移（hp/route/out_player）不会被这条测试抓住。

- **字段语义 / S6b 误读风险**：`ExitSolve::key_floor_not_consulted: Option<&'static str>`（`solve_local.rs:513-517`），doc 明写 `None` = 「消费了 floor，**或调用方没给 floor**」。S6b 如果按 `marker.is_none()` 判「已满足」，在「没给 floor」与「消费了 floor」两种情况下都会读错。字段名是否定式（`not_consulted`），尚可让 `is_some()` 判「未消费」，但没有表达「有没有给 floor」。
- **witness JSON 缺口（最重）**：`witness.rs:182-233` 的 floor 记录只有 `pure_pickup/pure_pickup_reason` 等，`witness.rs:435` 也只落这两个；`tests/witness.rs:475-480` 的字段序清单同样没有 `key_floor_not_consulted`，schema 仍是 `xiyou/witness/v3`。⇒ S6b 从 witness JSON 根本看不到 boss/pure 的「未消费」标记；boss 路径由 `outer_chain.rs:269` 走旧入口时尤其不可见。
- **doc 假声明**：`solve_local.rs:513-514` 说该字段「等价于统计面里的 `key_floor_not_consulted_by_path:<值>`」，但全仓 `grep key_floor_not_consulted_by_path` 只命中这行 doc，没有任何实现/落盘。
- **`outer_witness.rs:720` 的 `None` 是否正确定义**：当前正确且无害——该 fake 只被 `witness::exit_edge`（`witness.rs:379-385`，只读 `kind/exit/cross_edge`）消费；语义上「这个调用没给 floor」也确实是 `None`。但若将来 witness 传播该字段，这个 `None` 会被读成「消费了 floor」。建议改成显式构造（枚举/构造函数），而不是依赖默认 `None`。

### F9：gold 同值同源；ratchet 无掩盖

- 同值：`crates/game-solver/examples/f3_bound.rs:81` 与 `crates/game-solver/tests/exact_pickup.rs:124` 均为 `gold: 1231`；两处 `f3_chain_player` 其余字段逐项相同（`hp3227/atk730/def730/keys0/item_types[0]=13/item_uses[0]=255/stack1`、`F3_ENTRY=(2,11)`）。
- 同源：`docs/findings/outer-chain-real.json` 的 `prefix_floors` 里 index6 与 index10 的 `in_player.gold` 均 1231；index10 明确 `in hp3227 atk730 def730`，out `hp4027 atk731 blue1 gold1256`。`5792e01` 只改示例 `gold 225→1231` + 函数 doc（6 增 1 删）。
- **但** `crates/game-solver/examples/f3_bound.rs:4` 模块 doc 仍写 `链 51→…→2 的出层态 = … gold=225`，与同文件函数 1231 矛盾；旧值未清，F9.1 的「两处不许各说一套」只修了函数体，没修模块头。
- ratchet 测试数（`[#test]` 实测）：

| revision | exact_pickup | objective | solve_local | 基线 total |
|---|---:|---:|---:|---:|
| `fbecc72^` | 13 | 13 | 31 | 1310 |
| `fbecc72` | 13 | 13 | 31 | 1310 |
| `e42c26a` | 16 | 13 | 31 | 1310 |
| `5792e01` | 16 | 13 | 31 | 1310 |
| `d130206` | 16 | 13 | 31 | **1318** |

`d130206` 的 diff 只改 `data/status/baselines/rust-test-counts.json` 4 行：`exact_pickup 11→16 / objective 11→13 / solve_local 30→31 / total 1310→1318`；`#[ignore]` 在四个提交的所有 tests 文件里均为 0；没有任何文件的测试数下降。`python3 gates/cli.py status` 实测：

```
[✓ PASS] test-ratchet — total=1318(基线1318) 无回退
总 PASS13 FAIL0 SKIP1
```

⇒ 「借对齐之名掩盖删测」不成立；`e42c26a` 对 F8 是**加强**（`81..=88`→`==83`、`<6e6`→`==4_739_833`，新增 `picks/kills`），没有删/放宽既有断言行（diff 其余是 rustfmt 展开）。

---

## 3. S6b 接线前必须补的判据

1. **F6 / witness（阻断级）**：把 `key_floor_not_consulted` 落进 witness JSON，并升 schema（v3→v4）+ 补 `tests/witness.rs` 字段序断言。建议直接序列化三态而不是只在 `Some` 时给字符串：
   ```json
   "key_floor": { "requested": true, "consultation": "consumed" | "pure_pickup" | "boss" | "not_requested" }
   ```
   若暂不升 schema，至少加 `key_floor_requested: bool` + `key_floor_consultation: Option<&'static str>`；S6b **不得**用 `marker.is_none()` 推断「已满足」。
2. **F6 / boss 入口**：S6b 的 boss 目标必须走 `solve_local_boss_toward_shops_floor(..., Some(floor))`，并在 witness 落账；`outer_chain.rs:269` 目前仍走旧入口（`None`），接线后要有一条「boss 层带了 floor 且 marker 不再出现」的判据。旧入口保行为测试补 `assert_eq!(old, new_floor(None))` 全字段相等（`ExitSolve: PartialEq` 已具备）。
3. **F6 / API**：把 `Option<&'static str>` 换成显式枚举（`Consumed / NotConsulted("pure_pickup"|"boss") / NotRequested`），删掉/实现 `key_floor_not_consulted_by_path:<值>` 的 doc 声明；`outer_witness.rs:720` 的 fake 用 `NotRequested` 构造。
4. **F3 / 边界与入口**：补三条诊断判据——`[8,0,0]⇒None`（上界等值）、`[9,0,0]⇒Some`、入场黄钥=1 时 `[9,0,0]⇒None`；再补「`adopted=false`（撞怪 give_up）时 `key_floor_reason` 不得被消费方当作精确解结论」的判据，或在 `give_up` 时清空该字段。
5. **F8 / 去脆**：`picks==16` 改 `>=15`（或 `>=14` 且 `keys` 满足 floor）；`states` 改区间（如 `3_000_000..=5_500_000`）+ 单独断言 `states(None)==states(Some)`；`steps==83 / hp4027 / gold1256 / kills7` 保留。精确 `4_739_833` 若要留，必须改注释为「枚举顺序指纹，任何 DIRS/剪枝改动都要同步」，不能写「只由层决定」。
6. **F1 / 属性真实性**：`ps.atk` 改 730（真链 f3 入场值，实测逐字段等价），或在该测试注释明确「属性为合成、判据只证逻辑序位」。附带建议：主测试补 `exact_stat().key_floor_not_consulted.is_none()` 与 `key_floor_reason.is_none()`，把 F1 和 F3/F6 的「消费」语义绑在一起；自证段加 `ex_a.out_hp + 200 == ex.out_hp`、`ex_a.yellow == ex.yellow + 1` 的对照。
7. **F9 / 文档**：`examples/f3_bound.rs:4` 的 `gold=225` 改 1231（或删旧值改指向 `outer-chain-real.json`）；example 不参与门禁，无需加测试。
8. **ExactStat / 出口一致性（B 级观察）**：多出口场景补一条判据，断言 `ExactStat` 的 `picks/doors/steps` 与最终返回的 `ExitSolve` 来自同一出口（例如双出口、hp 相同但拾取件不同的夹具）；否则至少把 `ExactStat` 的字段 doc 写成「本层最高 hp 终态的统计」，禁止 S6b 用它反推出层钥匙/门数。

---

## 4. 明确回答：F1 那条判据现在是否真守住「key_floor 不许牺牲 hp」

**是——但范围仅限精确 DP 的终态比较器（`exact_pickup.rs:365-367`），不是全链路。**

理由（按证据强弱）：

1. **两个终态真互斥且都可达**：B（开门捡 P，`hp700`、黄钥 0、不满足 floor）由主断言选中并证明可达；A（不捡 P，`hp500`、黄钥 1、满足 floor）由第二夹具自证可达（M1b 证明：删掉 K 后自证立即红）。B 必须开门才能拿到 P（P 三面是墙，唯一入口是 D），A 必须不开门才能保钥 ⇒ 不存在「又保钥又拿 +200hp」的第三态。
2. **变异确实翻盘**：终态序改成 `floor→hp→steps` 后选出 A `hp500/keys1`，主断言 `hp700` 红（lead 已独立复现；本复核遵嘱未重跑该条，但 M1/M1b 独立证明了夹具结构，比较器顺序也已逐行核对）。
3. **判据断言的是「结果」而不是实现细节**：`hp700` + `yellow<1` + `doors1` + `picks2` 同时成立，只有「hp 压过 floor」一种解释；`picks==2` 是终态拾取位图口径（`exact_pickup.rs:418`），不是动作次数。
4. **不现实属性不构成反例**：`hp500/gold0/keys0` 是合成入场态，但判据要证的是**序位**；`atk900` 虽高于真链最大 731，实测 `atk730` 逐字段等价，且低 atk 失败的原因是 DP 对旁路 cat 的 fail-closed，不是判据依赖 900。

**不覆盖的部分**：boss 路径（`solve_local_boss_toward_shops_floor` 明确不读 floor，只置 `Some("boss")`）与纯拾取闭式（置 `Some("pure_pickup")`）都没有「floor vs hp」的取舍，S6b 不得把「有结果」当作「floor 已满足」；Search 回落路径的 floor 择优（`pick_exit_floor`）也没有一条 hp/floor 互斥夹具。若要宣称全链路守住，还需在 S6b 接线后补 boss/Search 两条同类判据。
