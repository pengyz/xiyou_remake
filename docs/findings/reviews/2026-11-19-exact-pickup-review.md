# 独立对抗性复核：P9 精确拾取 DP（`ad5fece` / `3bc38e2` / `76734f6`）

日期：2026-11-19　复核者：独立复核 agent（只读 + `/tmp` 变异，未改仓库代码）
对象：`ad5fece`（feat exact_pickup）+ `3bc38e2`（coverage example + ratchet）+ `76734f6`（findings），父 `20056e9`
相关：`docs/findings/2026-11-19-exact-pickup-dp.md`、`2026-11-19-f54-state-profile.md`、`2026-11-19-f3-gate27-walk-only.md`

复核纪律：不采信「测试全绿」作为判据；每条判定要么给出**独立复算/变异**的输出，要么给出**代码行级**的
推理链。所有变异都在 `/tmp/x`（`git archive HEAD | tar -x -C /tmp/x` + `assets/raw` 软链 + 独立
`CARGO_TARGET_DIR`）里做，仓库工作区除本文件外零改动（`git status` 仅本 review 新增）。

---

## 0. 判定汇总

| 面 | 判定 | 一句话 |
|---|---|---|
| F1 `4027` 是否 f3 最优 | **黄**（hp=绿 / steps=红） | `4027=3227+800` 的上界论证成立且路线达成；但「106 步、18 件全拿」**不是**最优 |
| F2 回放独立性 | **绿** | 只吃 route token，不读 DP 内部量；篡改预测可被抓（判据 3 实测） |
| F3 闸门健全性 A1–A7 | **黄** | 全线 fail-closed，A1–A7 逐条可指认；但 A6 单点采样 + **witness 链恒传空 shops** 是未披露口径 |
| **F4 支配剪枝** | **红** | 反向 dead 剪枝杀死「同 hp 更少步」的解：**88 步 / 14 拾取** 可达 4027，被告给 106 步 / 18 拾取 |
| F5 f51..f54 逐记录等价 | **绿**（有未披露差异） | 四记录逐字节 IDENTICAL；链路 6/275→8/465 复现；f2 路线也变了（未披露，良性） |
| F6 基线诚实性 | **绿**（一处笔误） | 1231 实测一致、8 条真测且无 `#[ignore]`；但 `SKIP=0` 实际是 `SKIP=1` |
| F7 变异可复现 | **绿** | M1/M4 各自真红，还原后 8 passed（md5 回正） |
| F8 覆盖率口径 | **黄** | 8/54 是上界口径且 5 层平凡；真实链路只放行 **3 层**；f3 在上界口径下反而是 `exit_unreachable` |
| F9 无规则表复制 | **绿** | 硬编码只有 `&[83]` 与测试夹具；A1–A7 全部调用 game-core（行号见 §9） |
| F10 夹带与诚实 | **黄** | 无越界文件；但三处过度声明（未断言的 `steps/states`、「18 件全拿/精确最优」、`SKIP=0`） |

**计数：绿 5 / 黄 4 / 红 1。**

**红只有一条**：`F4` 的支配剪枝不健全（不是「不安全」而是「不精确」——它杀掉的态里有 hp 相同但步数更少的）。
这是 `ad5fece` 唯一**实质性**缺陷；其余黄色都是口径/文案/过度声明，机制本身 fail-closed。

---

## F1 `4027` 真是 f3 最优吗（头号面）

**判定：hp 侧绿（是真最优），steps 侧红（106 不是最优，见 F4）。**

复现（本次实跑）：

```bash
cargo test -p game-solver --release --test exact_pickup exact_dp_f3_reaches_4027 -- --nocapture
```

关键输出：

```
[P9-1] f3 ok=true trunc=false out_hp=4027 exit=Some((11,11)) exact=true reason="adopted"
stat=ExactStat{steps:106, picks:18, doors:5, kills:7, states:130835, replay_steps:106, route_len:106}
1 passed
```

### F1.1 上界论证（我自己复算，不引用被告的结论）

f3 实体 35 个（`F3_FIGHT=1 F3_PILL=1 cargo run --release -q -p game-solver --example f3_bound -- 1000`），
hp 增益面只有两类：

* `ty31`（+50）× 4，cells `(2,2)(11,7)(11,1)(2,9)`；
* `ty32`（+200）× 3，cells `(4,2)(5,1)(6,2)`。

合计 `4×50 + 3×200 = 800`；链态入场 `hp=3227`（f2 出层值，`/tmp/before.json` 与 `/tmp/head.json` 一致）。
⇒ **任何** 模型内可达 hp ≤ `3227 + 800 = 4027`。DP 路线实际达成 4027（`out_player.hp=4027`），
且该路线经 `exact_replay_route` 逐字段回放（`adopted` 的前提）。**上界 + 达成 = 真最优**，
不是「上界恰好可达」的运气。这一点比自述更强：自述说「DP 在全部 (格,掩码) 状态上取到的精确最优」，
其实无需穷举也能证——上界与达成同值即证完。

### F1.2 隐藏 hp 来源逐项排除（每条给证据）

| 通道 | 证据 | 结论 |
|---|---|---|
| 门脚本写 hp（`RES_0` 在 slot 0） | `hp_bound.rs:277 SlotTerm{konst,abs_write,res_write}`、`:288-293 gain()`：`m=max(abs_write,res_write)` 后按「绝对写上限」计入 ⇒ A3 把它当**乐观上界**，只会多算不会少算；且 f3 的 `(2,11) param=0` 在 `exact_gate_effect` 里返回 `None` ⇒ `run()` 把它标成 `XC_BLOCKED`（代码注释自述「f3 的 (2,11) param=0 就是这种」），路线不进入 | 非洞 |
| 门脚本重复触发（slot 32） | `hp_bound.rs` 对每个 ty83 param 调 `gate::gate_slot_is_one_shot`，f3 槽 {0,27} 均一次性 | 已闸 |
| 击杀生成 hp 实体 | `hp_bound::plan` 1b：任一活 cat8 `drop_gain>0` ⇒ `Off(post_kill_hp_spawn)`；f3 7 只怪（41/42/43×2/44/45×2）`predict_engagement(counter=true)` 全 `loss=0`（真引擎），`apply_kill_drops` 净变化 0 | 已闸 |
| 商店买 hp | `hp_bound.rs:512 shop_offer`：`if (param>>8)&2 == 0 { return None }`（未开价）；f3 `ty77 (11,4) param=278=0x116` ⇒ bit9=0；`scene.rs` 同一判据；且链态 `gold=225 < 1000`（AddHp 2000/1000） | 无源 |
| 吃丹回血 | f3 入场背包只有 `item13 火眼金睛`（`OpenPanel EnemyInfo`，非回血）；唯一回血物是 `item19 千年月桂露`（`Heal mul74/div10`），不在背包也不在层内；**且 solver 动作集本就不含菜单** | 模型口径外 |
| 脚本给道具（slot 27 `GIN_0_13`） | 该槽谓词 `!flag(27) && from_walk && flag(0)`（`gate.rs:147`），链态 `flags=0xe337be_00000000` bit0=0 ⇒ `try_run_scene(27,true)=false` ⇒ 不执行（既有 A 级文档 `2026-11-19-f3-gate27-walk-only.md`，我复算 evaluate 表一致） | 不执行 |

**「我改判」**：我最初把 slot 0 的 `RES_0` 当成 A3 的漏网 hp 源（`script.rs:403` 里 RES 在难度 0 下
把玩家重置到 `hp=400`）。读 `hp_bound::plan` 的 `SlotTerm::gain` 后改判：RES 被建模成**绝对写上限**，
对「上界」是保守的；残留风险只是「RES 是重置而非增益」这一点只能靠回放抓——而 f3 的门格是墙、路线不入，
且回放每步驱 `gate_arrive`（见 F2），所以不吃这条风险。

### F1.3 但「106 步 / 18 件全拿」不是最优

F4 的反例：同样 4027，**88 步、14 件拾取**，经 `validate_outcomes` 全字段核对（`adopted=true`）。
⇒ 被告写进 commit/doc 的「18 件全拿」「DP 在全部状态上取到的精确最优」在 steps 目标上**不成立**。

---

## F2 回放独立性

**判定：绿。**

`exact_replay_route`（`crates/game-solver/src/solve_local.rs:4302`）的入参是
`(input, entry, ents, player_in, flags_in, route)`——**只有 route token 与入场态**，没有 `goals`、
没有 `states`、没有 `hp_at/keys_at`。token 三型各自推进：

* `bump ty@x,y` → `exact_bump`（内部 `bump_outcome` = `BattleResolver::resolve_bump` + `predict_engagement`）+ `apply_kill_drops`；
* `door x,y` → `exact_open_door`（`consume_key_for_door`）+ 移除实体 + `gate_arrive`；
* `x,y` → `classify`：`Cell::Pickup` 走 `pickup_step`（`solve_local.rs:3338`：`build_scene` + `try_step` + 8 tick + 逐字段回读）；`Cell::Move` 走 `st.pos=(x,y); st.steps+=1; s.gate_arrive(...)`。

**「我改判」**：我第一遍读 `exact_replay_route` 时把 `Cell::Move` 分支记成「只改坐标、不驱引擎」，
并据此担心「踩门脚本无人验」。重读 4343–4362 行确认 `Move` 分支**调了 `s.gate_arrive`**
（`GateArrive::Pass` 才继续），即回放对每一格都跑门判定；普通走格不模拟 `try_step` 行走动画，
但行走本身无副作用，副作用点（拾取/门/撞怪）全部走引擎。所以 A6 的单点采样只是**覆盖面**问题，
不是健全性洞——任何「入场恒等、后来变非恒等」的门，一旦路线踩上去，回放会真实执行脚本并抓出分歧
（`replay_mismatch`）。

篡改判据实测（`--lib --list` 命中 `exact_dp_corrupted_prediction_is_off`）：

```bash
cargo test -p game-solver --release --lib exact_dp_corrupted_prediction_is_off
# → 1 passed：5×3 走廊 2 枚 ty32(+200)，篡改 goals[0].hp 与 post.hp 后
#   validate_outcomes == Err(("replay_mismatch", _))
```

更强的独立证据：F7 的 M4 变异删掉 A1 的 `monster_damage` 早退后，本应「放行」的层仍被回放抓住，
`reason="replay_mismatch"` ⇒ 回放不是 DP 的自证。

---

## F3 闸门健全性（A1–A7 逐条）

**判定：黄（机制绿；一处未披露口径 + 一处覆盖边界）。**

逐条读码（`crates/game-solver/src/exact_pickup.rs`）：

| 闸 | 实现 | 判定 |
|---|---|---|
| A1 怪零损血 | `bump()` :619 缓存键 `(ty,atk,def)`，`loss!=0 ⇒ Err(("monster_damage",ty))`；`delta!=0 ⇒ kill_spawns_ents` | 绿（M4 实测会红） |
| A2 步臂 | `run()` :185 `hazard_gate(input,entry,root_ents,&[83])`：闭式解**同一函数**，`modeled=[83]` 只把「DP 显式建模的 ty83」排除出 G2 几何可达 | 绿 |
| A3/A4 hp 源 | `run()` :192 `hp_bound::plan`：非一次性槽、脚本 hp 写/生成/未知 op、活怪 `drop_gain` 任一命中即整层 Off | 绿 |
| A5 拾取数 | `pick_ty.len() > 20` ⇒ `too_many_pickups` | 绿（性能护栏，非正确性；doc §3 已注明） |
| A6 门恒等 | `run()` :219–241：逐 `XC_GATE` 调 `exact_gate_effect`（`Search::gate_arrive` + `try_run_scene`）；`None`（Terminal/Dropped）⇒ **该格变墙**；`Some` 且 `(player,flags,ents)` 有变 ⇒ `Off(gate_mutates_state)` | 黄（单点采样，见下） |
| A7 拾取认证 | `exact_layer_view` :4141：`certified_monotone_pickup`（Consumed/六轴非降/gold-tutorial-equip 不变/背包只能合并或+1 槽/严格增益）+ `is_gain` + **`gain.bag` 必须为空**（:4149 `pickup_adds_bag`） | 绿（f5 实测挡住 ty14） |
| 背包槽/`trim_slots` | `pure_pickup::trim_slots` 仅 `fn→pub(crate) fn`（diff 可见），`player_eq` :320 用它推导范形，非零槽仍逐槽比 | 绿（无放宽） |

f3 的实测路线佐证了这两扇门在链上确实「不做事」：106 个 token 里 `(2,11)`（param=0 的门格）出现 **0** 次
（它是入口、且被采样判成墙），`(5,11)`（param=27）出现 **1** 次且该槽谓词 `from_walk && flag(0)` 因
`flags` bit0=0 恒 false ⇒ 不执行脚本（与既有 A 级文档 `2026-11-19-f3-gate27-walk-only.md` 一致）。

**A6 的单点采样**：每扇门只在**入场态**（`player_in/flags_in/root_ents`）采一次恒等。理论上存在
「入场恒等、吃到某拾取物后才变非恒等」的门（如 `gate.rs:34` 依赖玩家 hp）。这条不构成健全性洞，
因为回放逐格驱 `gate_arrive`（F2），一旦真触发就 `replay_mismatch`；代价是**覆盖面**——
这类层会被「先放行、回放再拒」，白烧一次搜索。建议在 doc §3 A6 行补一句「采样在入场态，非入场态
靠回放兜底」，别让读者以为等价于「全状态恒等」。

**真正要写进口径的洞（黄）**：`Cell::Shop` 分支（`solve_local.rs:4157`）

```rust
if !shops.is_empty() { return Err(("shop_present", ty)); }
ExactCell { kind: XC_BLOCKED, ... }   // 空商品表 ⇒ 视作阻挡
```

而 witness 链是：`bin/solver.rs:462` `let (gi, _shops) = solve_global_input(1);` **丢弃 shops**
→ `run_witness` → `solve_local_exit_toward` → `solve_layer` → `solve_layer_with(..., ClosurePolicy::OFF, &[])`。
即 **`solver run` 的整条链路 shops 恒空**，只有 `solver solve-global`（`solve_global.rs:494`）传真实商品表。

我为这条写了一个独立探针（`/tmp/x/crates/game-solver/examples/shop_probe.rs`，同一 f3 链态、同一个
`solve_layer_with`，唯一差异是 `shops` 参数）：

```
[shops=EMPTY]       ok=true goals=2   adopted=true  reason=adopted        out_hp=4027 steps=106 picks=18
[shops=REAL(17行)]  ok=true goals=523 adopted=false reason=shop_present ty=77
```

⇒ **「打穿 f3」只在「商品表未建模」口径成立**；换成仓库里真实 17 行商品表，f3 会被
`Off(shop_present)` 拒掉。这条对**正确性无影响**：f3 的 `ty77 param=0x116` 的 bit9=0，
引擎（`scene.rs` 同判据）根本不会弹购买面，所以它卖不出任何 hp，4027 仍是真最优；
拒绝是**过度保守**（sound but loses coverage）。但口径必须写明，否则「f3 打穿」会被读成
「solve-global 也能打穿 f3」——后者为假。

`trim_slots` 改 `pub(crate)`、`Grid` 改 `pub(crate)`：只看可见性，`cargo test -p game-solver` 里
`pure_pickup` 15 条与 `tests/pure_pickup.rs::f3_is_not_closed_form_g2_first` 均绿；`hazard_gate` 的
`modeled` 参数在闭式解侧传 `&[]`（`modeled.contains(ty)` 恒 false ⇒ 与旧 G2 逐字等价），唯一可观测差异是
「被拒路径也回带 `entry_hazards`」（`no_level` 等早退以前是 0，现在是非零；纯诊断面，commit 已说明）。

---

## F4 支配剪枝（第二号面）——红

**判定：红。`push()` 的反向 dead 剪枝杀死了「hp 相同、步数更少」的状态。**

代码（`exact_pickup.rs:587-613`，`push()` 体）：

```rust
// 前向：已有超集 pm2 ⊇ pm ⇒ 本态被支配（安全）
if front.iter().any(|&(pm2, _)| n.pm & !pm2 == 0) { return Ok(None); }
...
// 反向：本态 pm ⊃ 已有 pm2 ⇒ 后者 dead
for &(pm2, j) in front.iter() { if n.pm & !pm2 != 0 { self.dead[j] = true; } }
front.retain(|&(pm2, _)| pm2 & !n.pm != 0);
```

反例复现（`/tmp/x`：删掉「标 dead + `front.retain`」这 5 行反向标记、`MAX_STATES` 改成 `6_000_000`，其余不动）：

```bash
cargo test --release -p game-solver --test exact_pickup exact_dp_f3_reaches_4027 -- --nocapture
```

```
[MUT] hp=4027 steps=88 len=88
head=["3,11","4,11","5,11","5,10","5,9","5,8","5,7","5,6","5,5","5,4","5,3","5,2","5,1","4,1",
      "4,2","5,2","6,2","6,3","5,3","5,4","5,5","4,5","bump 43@3,5","3,5","2,5","1,5", ...]
[P9-1] f3 ok=true trunc=false out_hp=4027 ... steps:88, picks:14, doors:5, kills:7,
       states:4741000, replay_steps:88, route_len:88
assertion `left == right` failed: 18 件拾取物必须全捡     ← 只有这条测试断言红
```

因为 `adopted=true` 是 `validate_outcomes` 全部通过后才置的，这条 88 步路线是**真引擎逐字段核对过**的：
同 hp（4027）、少 18 步、少 4 件拾取。⇒ 被告的「106 步、18 件全拿」不是最优。

**为什么 hp 仍最优、只有 steps 受损（我自己的论证）**：同 `(cell,dm,km)` 下 `pm1 ⊇ pm2` ⇒ 可达格集相同
（只由 dm/km 定）、hp/atk/def/钥匙逐轴 ≥、gold 只由击杀定、A7 保证每次拾取严格增益且不改背包槽
⇒ 被杀的 `pm2` 的任何延伸在 hp 上都不会更好。但 BFS 是 FIFO 单位步：`pm2` 先入队 ⇒ `steps2 ≤ steps1`，
于是「同 hp 取 steps 最小」的 tie-break 会输给被剪掉的那个子集态。**hp 结论不受影响，steps 结论不成立。**

**修法（二选一）**：
1. 删掉反向 dead 标记，只保留前向剪枝——本层状态 4.74M、实测 3.08 s（vs 现值 130835/0.04 s），
   仍然是可接受的一次性成本，换来真正的 (max hp, min steps)；
2. 把 steps 纳入支配谓词：仅当 `steps1 ≤ steps2` 时才允许 `pm1` 杀 `pm2`（前向剪枝天然满足，反向标记
   需要记录入队步数）。方向 1 更简单、更不易再错。

无论选哪个，doc §4 的支配引理必须补一句：「本引理只证 hp 不劣；steps 需额外条件，否则不可反向剪枝」。

---

## F5 f51..f54 逐记录等价

**判定：绿（自述成立），但有一处未披露的良性差异。**

复现（`/tmp/x` 里 build 父版 `20056e9` 与 HEAD，各自 `solver run --out`）：

```bash
git archive 20056e9 | tar -x -C /tmp/x-before && ln -s $PWD/assets/raw /tmp/x-before/assets/raw
CARGO_TARGET_DIR=/tmp/xt-before cargo run --release -q -p game-solver --bin solver -- run --out /tmp/before.json
cargo run --release -q -p game-solver --bin solver -- run --out /tmp/head.json
```

stderr 对照（与自述逐字一致）：

```
before: status=capped floors=6 total_steps=275 blocked=floor 3 reason=truncated
head:   status=capped floors=8 total_steps=465 blocked=floor 5 reason=truncated layer_ok=Some(false) layer_truncated=Some(true)
```

逐 floor 记录比（提取 `floors[i]` 原文）：

| floor | before | head | 结论 |
|---|---|---|---|
| 51 | route 69 步 | route 69 步 | 记录**逐字节** IDENTICAL |
| 52 | route 102 步 | route 102 步 | 记录**逐字节** IDENTICAL |
| 53 | route 25 步 | route 25 步 | 记录**逐字节** IDENTICAL |
| 54 | route 63 步 | route 63 步 | 记录**逐字节** IDENTICAL |
| 1 | route 6 步 | route 6 步 | 全字段 SAME |
| 2 | route 首 token `1,2`，10 步 | route 首 token `1,3`，9 步 | **DIFF**（out_hp 均 3227） |
| 3 / 4 | 不存在（blocked@3） | 新增（106 / 85 步） | 新增层 |

⇒ 自述「f51..f54 逐记录 sha256 IDENTICAL」**成立**；但 **f2 的路线也变了**（exact 接管，`states=13`，
同 hp 少一步），commit 只说「f51..f54 不变 / 通用搜索路径零触碰」，没提 f2 也换了路线。差异良性
（f2 本来就被 exact 精确接管），但按 AGENTS §9「有意变更须写明」应补一句。

---

## F6 基线诚实性

**判定：绿（一处口径笔误）。**

```bash
python3 gates/cli.py status   # 实测尾部
# PASS=13 FAIL=0 SKIP=1 TOLERATED=0
# [- SKIP] visual-diff — 视觉比对器未就绪（P4 启动后启用）
# [✓ PASS] test-ratchet — total=1231(基线1231) 无回退
cargo test -p game-solver --release --test exact_pickup -- --list   # 8 个名字
cargo test -p game-solver --release --lib -- --list | wc -l        # 62（含 lib 单测）
grep -c '#\[ignore\]' crates/game-solver/tests/exact_pickup.rs      # 0
```

`data/status/baselines/rust-test-counts.json` 的 `exact_pickup(test): 8` / `game_solver(lib): 62` 与实测一致，
无 `#[ignore]` 占名额。唯一不实：`3bc38e2` 的 message 写「`gates/cli.py check` PASS=13 FAIL=0 **SKIP=0**」，
实测 `SKIP=1`（visual-diff 未就绪，属既知 SKIP）。低危，但按 AGENTS §5「SKIP 必须显式打印」应写真数。

---

## F7 变异可复现

**判定：绿。两发都复现了「真红 → 还原后绿」。**

**M1（支配方向反）**：把 `push()` 的前向判定改成 `pm2 & !n.pm == 0`（把超集当被支配）：

```
[P9-1] f3 ok=true trunc=false out_hp=3227 exit=Some((1,11)) ... steps:1 picks:0 doors:0 kills:0 states:263
assertion `left: 3227 right: 4027` → FAILED
```

与 doc §4 记载的「第一版方向写反 ⇒ 3227 / 263 states / 门全未开」逐字吻合。

**M4（A1 不查损血）**：删掉 `if loss != 0 { return Err(("monster_damage", ty)); }`：

```
[P9-1] ... reason="replay_mismatch"
assertion `left: "replay_mismatch" right: "monster_damage"` → FAILED
```

值得记一笔：A1 删掉后，回放防线仍然兜住（层被 `replay_mismatch` 拒），只是**理由字面**变了 ⇒ 判据
「断言具名理由」的价值被这条变异证明（否则测试可能只是绿着放行）。

还原后：`exact_pickup.rs` md5 `bce0ac41ff6b589dc694b06228c0ceda` 与 HEAD 一致，
`cargo test --release --test exact_pickup` = **8 passed / 0 failed / 0 ignored（7.88 s）**。

**未覆盖**：M2（门 token）、M3（撞怪计步）、M5（MAX_PICKUPS）我没复跑；判据存在性与自述一致，但我
不把它们当已证。

---

## F8 覆盖率口径诚实性

**判定：黄（口径已声明，但「8/54」几乎不携带链路信息）。**

`cargo run --release -q -p game-solver --example exact_coverage`（口径写在文件头：hp/atk/def=9999、
`flags=0`、`entry=层 start` ⇒ **上界口径**）：

* `adopted = 8/54`：f2、f6、f12、f23、f24、f27、f28、f36；
* 其中 **5 层是平凡层**（`picks=0 doors=0 kills=0`、`out_hp=9999`、10 步走到楼梯）：
  f2/f23/f24/f27/f28，全是「入口走到楼梯、什么都不捡」的空层；
* 剩余三层：f6（5/2/9，`out_hp=10149`）、f12（2/0/2，10199）、f36（2/0/4，10799）；
* **f3 在上界口径下是 `exit_unreachable`**（原表行 `3 false exit_unreachable 0 0 0 0 0 2`，
  即 out_hp=0、states=2），根本不在 8 层里。

真实链路口径（在 `/tmp/x` 的 `solve_local.rs:787` 插 `[EXACT]` 打印，跑 `solver run`）：

| floor | entry | exact | reason | rty | out_hp | picks/doors/kills | states |
|---|---|---|---|---|---|---|---|
| 51 | (1,11) | Off | `gate_mutates_state` | 47 | 0 | — | 0 |
| 52 | (2,11) | Off | `pickup_uncertified` | 13 | 0 | — | 0 |
| 53 | (6,10) | Off | `too_many_dims` | 21 | 0 | — | 0 |
| 54 | — | 未走 exact | 被 `pure_pickup` 先接管 | — | 3227 | — | — |
| 1 | (6,11) | Off | `step_arm_hazard_on_entry` | 83 | 0 | — | 0 |
| 2 | (1,2) | **adopted** | — | 0 | 3227 | 0/0/0 | 13 |
| 3 | (2,11) | **adopted** | — | 0 | 4027 | 18/5/7 | 130835 |
| 4 | (11,10) | **adopted** | — | 0 | 4327 | 7/5/6 | 377280 |
| 5 | (2,11) | Off | `pickup_adds_bag` | 14 | 0 | — | 0 |

⇒ **真实链路只放行 3 层（f2/f3/f4），且 f3 还依赖「空商品表」口径（F3）**；f2 是平凡层。
所以「54 层里 8 层被精确解接管」这句只在 doc §6 的口径声明下成立，而 §1 把它与「f3 打穿」并列时，
读者极易读成「链路能力 8 层」。建议 §1 改成「上界口径 8/54（含 5 个平凡层），真实链路 3 层」。
这不是造假（§6 明确写了保留），是**信息摆放**问题 ⇒ 黄。

---

## F9 无规则表复制

**判定：绿。**

```bash
grep -nE 'ty *== *(1|5|13|14|19|26|28|29|30|31|32|60|61|62|76|77|78|83)\b|\b(12|13)\b.*slot|price' \
  crates/game-solver/src/exact_pickup.rs
# 仅 :185 `hazard_gate(..., &[83])` 与 lib 单测夹具 ty32（:784-785）
```

826 行里没有 ty 表、没有槽号表、没有价格/商品表。A1–A7 的调用点（行号实测）：

| 闸 | 调用点 | 被调实现 |
|---|---|---|
| A1 | `exact_pickup.rs:619`（`bump()`） | `solve_local.rs:1139 bump_outcome` → `BattleResolver::resolve_bump` + `predict_engagement` |
| A2 | `exact_pickup.rs:185` | `pure_pickup.rs hazard_gate`（本片抽出的共用函数） |
| A3/A4 | `exact_pickup.rs:199` | `hp_bound.rs:630 plan` → `gate::gate_slot_is_one_shot` + `parse_slot` |
| A6 | `solve_local.rs:4243 exact_gate_effect` | `solve_local.rs:3110 gate_arrive` → `scene.rs:915 try_run_scene` |
| A7 | `solve_local.rs:4141` | `effect_registry.rs:536 certified_monotone_pickup` |
| 回放 | `solve_local.rs:4302` | `exact_bump` / `exact_open_door`（`combat.rs consume_key_for_door`）/ `pickup_step` / `gate_arrive` |

`exact_bump`（:4213）与 `bump_outcome` 的差异只是「返回 loss/gold/ents 三元组」，战斗本身仍是同一个函数；
没有第二套伤害公式。

---

## F10 夹带与诚实

**判定：黄（无夹带，但三处过度声明 + 两处未披露）。**

`git show --name-status` 三笔：

```
ad5fece  A crates/game-solver/src/exact_pickup.rs
         M crates/game-solver/src/lib.rs
         M crates/game-solver/src/pure_pickup.rs
         M crates/game-solver/src/solve_local.rs
         A crates/game-solver/tests/exact_pickup.rs
3bc38e2  A crates/game-solver/examples/exact_coverage.rs
         M data/status/baselines/rust-test-counts.json
76734f6  A docs/findings/2026-11-19-exact-pickup-dp.md
```

无越界文件、无顺手改。`pure_pickup.rs` 的改动我逐 hunk 看过：`Grid`/`trim_slots` 只加 `pub(crate)`；
`hazard_gate` 是**原样抽出**（G2′ 逐字不变，G2 只多 `modeled.contains(ty)` 且闭式解传 `&[]`）；
`try_closed_form` 把 `st.entry_hazards` 提前到 `hazard_gate` 返回值——**唯一可观测差异**是「早退路径
（`no_level` 等）的 `entry_hazards` 从 0 变成非零」，纯诊断面，commit message 已写明。无放宽断言。

过度声明（按严重度）：

1. **「`exact_dp_f3_reaches_4027` 断言 … steps=106 … states=130835」**——测试里**没有**这两条断言
   （只有 `out_hp/exit/picks/doors/kills/replay_steps==steps/route_len>0`）。断言「值」与「跑出值」
   是两件事，commit 把日志当断言写。
2. **「18 件全拿」「DP 在全部状态上取到的精确最优」**——被 F4 的 88 步 / 14 件反例推翻（hp 最优仍成立）。
3. **「PASS=13 FAIL=0 SKIP=0」**——实际 `SKIP=1`（visual-diff）。

未披露（良性但应写）：

4. **witness 链丢弃 shops**（`bin/solver.rs:462`）⇒ f3 的 adopted 依赖空商品表（F3/F8）；
5. **f2 路线也变了**（10→9 步，F5）。

`76734f6` 的 doc 本身质量不错：§3 的 A1–A7 表、§4 的支配引理正反两面、§6 的三点口径保留都成立；
需要就地更正的是 §1（106/18/全拿）与 §4（引理缺 steps 条件）。

---

## Confidence

* **对本复核：88/100。** F4 反例、shops 探针、f5 探针、M1/M4、before/head 对比、coverage 全表都是本次实跑，
  命令与输出都在文中，可独立重放。扣分项：M2/M3/M5 未复跑；未验证「74 金样 sha 不受影响」；
  未跑 `solve-global` 全链路（只验证了 `solve_layer_with` 同一函数在两种 shops 下的分叉）；
  f5 的「+50 是否可达」只给了上界与探测入口，未证死。
* **对 `ad5fece` 作为机制实现：78/100。** hp 精确、fail-closed、逐条复用 game-core、回放防线真实有效，
  这些是硬功夫；扣分全部来自 (a) 支配剪枝对 steps 不健全（红），(b) 声称的「精确最优/106 步」不实（黄），
  (c) shops 口径未披露（黄）。

---

## 两句话

1. **能不能合并**：**不能按现在的 message 合并**——先修 F4（删反向 dead 剪枝或把 steps 纳入支配谓词，
   然后把「精确最优 / 106 步 / 18 件全拿」改成「hp 最优；88 步 14 件亦可达」），并在 doc/commit 里
   写明「witness 链 shops 恒空、真实商品表下 f3 会被 `shop_present` 拒」；机制本身（闸门/回放/复用）
   没有再需要改的地方，上述三条改完即可合。
2. **f5 的 `truncated` 该先查什么**：先查 f5 被 exact 拒掉的 `ty14 生死簿`——它只是加背包槽的**非 hp 物**
   （`pickup_adds_bag`），而 f5 唯一 hp 拾取是 `ty31@(3,9)` 的 +50（上界 4377，truncated 报的 4327 至多差 50）；
   所以第一步是确认「`(3,9)` 的 +50 是否被 200k 节点截断挡住、以及能否绕开 `(4,9)` 的生死簿到达它」：
   能 ⇒ 给 exact 加背包维/放宽非 hp 加槽物，拿 4377；不能 ⇒ 4327 就是本层真最优，卡点在 f6 而不是 f5。
