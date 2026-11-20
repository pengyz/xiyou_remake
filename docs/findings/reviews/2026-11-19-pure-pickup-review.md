# 2026-11-19 纯拾取层闭式解（`0a244ff`）对抗复核

> 复核者：独立对抗复核（只读源码 + 跑测试/门禁/CLI；变异实验一律在 `/tmp` 副本内）
> 被复核 commit：`0a244ff feat(solver): 纯拾取层闭式解（fail-closed）+ 真引擎回放，f54 层界打穿`
> 父 commit：`cabb1b3`（`certified_pure_hp_gain` 由父引入，本 commit 未动 `game-core`）
> 基线：`data/status/baselines/rust-test-counts.json`（`total 1164 → 1183`）

## 0. 方法与前提

实现者自述**不作为证据**；测试全绿**不作为判据**。所有结论按 AGENTS.md §3 分级：

- **A 级**：两条独立路径给出同一数字（自写回放 vs 引擎 `pickup_step`；旧二进制 vs 新二进制；变异体 vs 原型）。
- 关键手法：① 用 `git archive` 建 `cabb1b3` / `0a244ff` 两份**独立编译**的快照；② 在 `/tmp` 副本里自写回放循环（`build_scene` → `try_step` → 8 tick 落格 → 手读 player/ents/flags），**不调用** `Search::pickup_step`、**不调用** `replay_pickup_route`，以免用被测代码验证被测代码；③ 四个变异体（A/B/C/O）逐条对照自述主张。

```text
仓库：/home/peng/workspace/xiyou_remake   HEAD=0a244ff   branch=modern
副本：/tmp/pprev-old (cabb1b3)  /tmp/pprev-new (0a244ff)
      /tmp/pprev-mutA|mutB|mutO（由 pprev-new 派生，仅 /tmp 内改动）
注意：assets/ 被 .gitignore（只有 assets/README.md 入库），副本必须
      `ln -s $REPO/assets/raw /tmp/pprev-*/assets/raw`，否则 tests/pure_pickup.rs:47 全 panic。
```

---

## 1. 判定摘要

| 面 | 判定 | 一句话证据（详见 §2） |
|---|---|---|
| F1 `3227` 是不是真的 | **黄** | 自写回放独立复现 `hp=3227 pos=(7,1) ents=19 steps=62 flags 不变`（A 级）；但闭式把**(7,10) 下行梯出口整条丢弃**，出口最优性未证明 ⇒ 口径收缩记黄 |
| F2 闸门能否被绕过 | **绿** | 7 个夹具：同格双药→`pickup_cell_shared`、入场宝箱→`step_arm_hazard_on_entry`、邻格陷阱→`step_arm_hazard_reachable`、真不可达陷阱→放行；G2 与「陷阱是邻接判定」同构，不会漏 |
| F3 回放是否不可替代 | **绿** | 变异 B（跳回放+不接出口段）：**6 passed / 1 failed**，只有判据 2 红、判据 1/7 全绿；变异 O 证明 `replay_flags_changed` 是活代码 |
| F4 非纯层是否不变 | **绿** | 旧/新两个独立二进制 `solver run` 的 f51/f52/f53 **整个 floor 对象逐字节相同**；变异 C（强制关闭闭式）新二进制输出与旧二进制**全 JSON 逐字节相同** |
| F5 基线诚实性 | **绿** | 12 例 = `game_core(lib)+3 / game_solver(lib)+8 / solve_local(test)+1`，实测 412/46/25 与代码一致（旧基线记 409/38/24）；78 个二进制、`total 1183`、MISSING/DIFF 均空 |
| F6 确定性 | **绿** | 跨进程两次 `solver run` 输出 `BYTE-IDENTICAL`；`pure_pickup.rs` 无 `HashMap/float`（仅注释提到），memo 热冷结果相同（`order_dependent=false`） |
| F7 可观测性缺口 | **红** | `solver run` 的 floor 对象只有 15 个键，**没有** `pure_pickup`/`reason`（`witness.rs:157-192`、`:331-360` 丢弃 `got.pure_pickup`）⇒ P5 无法从产物判定某层是否闭式 |
| F8 静默失败残留 | **绿** | 17 个 `return give_up(...)` 调用点（14 个不同 reason）+ 11 处 `note_reason`；`attempted:true` 处 `:433` 写 stat；3 个裸 `?`/`return None`（`:239/:299/:315`）都在 `Grid` 纯函数内、无 stat 语义；`hp_overflow` 与 `geo_reach` 两处已修 |
| F9 引擎侧改动 | **绿** | `git diff --stat cabb1b3 0a244ff -- crates/game-core` **为空**；门禁 `state-baseline` 实跑 `74 场景 sha 全同`；`certified_pure_hp_gain` 只操作局部 `&mut player`，无 static/thread_local |
| F10 门禁/夹带/并发 | **绿** | `python3 gates/cli.py check` → **`PASS=13 FAIL=0 SKIP=0`**（含 `rust-tests` 全绿、`test-ratchet total=1183` 无回退）；`reflog` 单条 `commit:`；`--name-status` 7 个文件全部属于本片；`git status --short` 仅他人 review 文档 |

**计数：绿 8 / 黄 1 / 红 1**（F1 黄、F7 红，其余 8 面绿。F2/F3/F8 带有已记录的保守点与模型边界，见各面「残留」）。

---

## 2. 每面详证

### F1 `3227` 是不是真的（黄）

**① 独立回放（不碰被测函数）**——`/tmp/pprev-new` 追加 `#[cfg(test)] mod pprev_probe`：

```text
solve: ok=true trunc=false out_hp=3227 exit=Some((7,1)) kind=Some(Stair) route=63
       pure_pickup=true reason=adopted pickups=18 sum=2400 orders=0 replayed=3227
own_replay: hp=3227 pos=(7,1) ents=19 steps=62 flags_unchanged=true nonzero_deltas=18
drugs=18 distinct_cells=18 visited_drug_cells=18 all_covered=true
reachable hazard cells (excl entry) = []   route lands on hazard cells = []
```

18 个非零增量逐格列出：`("5,9",50) ("4,8",200) ("4,6",200) ("4,4",200) ("4,2",200) ("5,3",50)
("9,3",50) ("10,2",200) ("10,4",200) ("9,5",50) ("10,6",200) ("9,7",50) ("10,8",200)
("9,9",50) ("10,10",200) ("5,7",50) ("5,5",50) ("4,10",200)` = `8×50 + 10×200 = 2400`，
`827 + 2400 = 3227` 一字不差；hp 轨迹 63 点单调不降；`flags` 全程未变（与回放内 `replay_flags_changed` 独立互证）。

**② 出口清单与 (7,10) 丢解**（`pure_cell_kinds` + `level.exits` 直读）：

```text
[X] exits.len()=2
[X] exit kind=Stair cell=(7,1) index=1
[X] exit kind=Stair cell=(7,10) index=2
[X] stair ent ty=7 at (7,1) param=1  (param&0xff=1)      ← 上行梯，目标层 1
[X] stair ent ty=8 at (7,10) param=53 (param&0xff=53)    ← 下行梯，回 f53
[X] grid 13x13 hazards=[(7,9,83)]
[X] y= 9 ####.P.E.P.##
[X] y=10 ####P##D##P##
```

`(7,10)` 的四个正交邻格里只有 `(7,9)`（入场格）可走 ⇒ 收集 18 颗药之后再从 `(7,10)` 出层，
在几何上**必须再踩一次入场格**。`pure_pickup.rs:186-194 grid.step_ok` 的 `cell != entry`
把这条路整条禁掉，于是 G4 只剩 `(7,1)`。

旧搜索**能**走到 `(7,10)`（`/tmp/pprev-old`，默认 `SOLVER_LAYER_LIMIT=200000`）：

```text
[OLD] f54 isolated(None): ok=false trunc=true out_hp=2227 exit=Some((7,10)) route=26 6.11s
[OLD] f54 toward=Some(1)（witness 口径，memo 命中）: out_hp=2027 route=23
```

⇒ 同一层旧搜索的 goal 集是 `{(7,10)→2227, (7,1)→2027}`，闭式只回 `{(7,1)→3227}`。
**这不是血量损失**（3227 逐点支配两个旧目标），但**是 goal 集收缩**：任何需要「从纯层下撤」
的消费方（`solve_global` 的回撤/复访）在纯层上会看不到下行边。对上行主线（witness P4）无影响。
自述承认「未证明 (7,10) 方向最优」——复核确认这是真的未证明，且比自述更具体：**下行边不是"没验证"，
而是被闭式结构性丢弃**。

**残留**：`LayerGoal.kind` 在成功分支硬编码 `ExitKind::Stair`（`pure_pickup.rs:680` 附近），
本层两个出口恰好都是 `Stair` 所以无害；但若将来 `level.exits` 混入其它 kind 而 G4 的 filter 被改，
这里会静默贴错标签。

### F2 闸门能否被绕过（绿）

synth 夹具：`5×3` 走廊，仅 `y=1` 可走，入口 `(0,1)`，药 `ty32@(1,1)(2,1)`，梯 `ty7@(4,1)`，
`run_synth → solve_local_exit_toward(..., None)`。逐条打：

| 夹具 | 构造 | 实测 reason / ty | 判定 |
|---|---|---|---|
| P2 同格双药 | `ty32@(2,1)` 重复 | `pickup_cell_shared` / 32 | G3 挡住 |
| P3 入场格宝箱 | `ty5@(0,1)` | `step_arm_hazard_on_entry` / 5 | G2a 挡住（非 83 一律拒） |
| P4 入场格正上方陷阱 | `ty62@(0,0)` | `step_arm_hazard_reachable` / 62 | G2 挡住邻接伤害 |
| P5 药与陷阱同格 | `ty61@(2,1)` | `step_arm_hazard_reachable` / 61 | 被 G2 先拦（G3 的 `pickup_cell_not_pure` 是第二道） |
| P6 可达非入场格宝箱 | `ty5@(3,1)` | `step_arm_hazard_reachable` / 5 | **G2 独占地盘**：`ty5` 不是 cat8，G1 覆盖不到 |
| P7 负控（真不可达） | 7×3 走廊 + `ty5@(6,0)` 四面不可走 | `adopted` / 3227→500 | 不误杀 |

**为什么 G2 不会漏**（机制取证，A 级代码锚）：`crates/game-core/src/scene.rs:3889-3948
apply_step_cell_effects` 的 60/61/62 伤害是**落地格的邻接判定**（61 需左右或上下两侧都有、62 任一
邻格、60 任一邻格；`armor40` 免疫 61/62 且 60 不计数，`:3900-3902`）。而 `pure_pickup.rs:186-194
geo_reach` 的 BFS 把 hazard 格**本身**当可走节点 ⇒「某格落地会挨打」⟺「该 hazard 格 BFS 可达」，
两条判据同构，不存在「邻接可站但 hazard 判不可达」的缝。这条正是自述 claim 2 里说的修复点，
实测（P4/P6）与代码一致。

**残留（保守，非漏洞）**：G2 不利用 `armor40` 免疫 ⇒ 穿 40 号甲的玩家本可安全穿过陷阱走廊，
闭式仍然拒绝并回落通用搜索（丢掉本可命中的闭式快路）。同理 61 的单侧不算伤害也被保守拒绝。
方向安全（fail-closed），代价是解集偏小。

### F3 回放是否真不可替代且不可绕过（绿）

变异 B（跳过 `replay_pickup_route`，直接伪造 `out`，且不拼接「最后一颗→出口」段）：

```text
f54: out_hp=3227 ok=true truncated=false exit=Some((7,1)) route=51 pure_pickup=true
     stat{reason=adopted, ..., replay_steps=50}
chain: status=blocked floors=4                     ← 整条 witness 仍然"成功"
test result: FAILED. 6 passed; 1 failed
panicked at tests/pure_pickup.rs:304:  left: (4, 10)  right: (7, 1)
```

⇒ **判据 1（3227）/判据 3/4/5/6/7 全绿，唯一红的是判据 2**，与自述 claim 3 逐字吻合
（自述给的 `pos=(4,10) steps=50` 我也逐字复现）。回放确实是唯一防线。

变异 O（`flags ^= 1`，验证 `replay_flags_changed` 不是死代码）：

```text
reason=replay_flags_changed, pure_pickup=false, out_hp 回落 2027/truncated
```

⇒ `:637` 的分支可执行且 fail-closed 生效。

变异 A（漏算一颗药，`pickups=17 sum=2200`）：

```text
reason=replay_pickup_left, hp_expected=3027, pure_pickup=false, out_hp 回落 2027/truncated
```

⇒ `:651` 的 `replay_pickup_left` 活，且**不会把 3027 当答案交出去**（与自述一致）。

**「隐性代价」检查**：`hp_expected = hp_in + Σdelta` 是**闭式上界**，任何中途掉血都会让终值
小于上界 ⇒ `replay_hp_mismatch` 抓住；`steps` 自洽检查（`:661`）抓住「同格来回凑步」；
九字段副作用（`:624-633`）+ flags + 实体多重集抓住一切模型内非 hp 变化。

**残留（模型边界，已在证书侧兜底）**：回放只能看到 `PlayerSpec` 的 10 个字段（hp + 9 个副作用字段），
`tutorial_seen / equipped_weapon / equipped_armor / exp` **不在 `PlayerSpec` 里**，回放看不见。
唯一守卫是 `certified_pure_hp_gain` 的探针（`effect_registry.rs:391-470` 断言这 3 个字段不变）。
它是**单状态采样**，不是证明：若某 HpUp 药的 `pickup` 分支依赖玩家状态（当前探针固定
hp100/atk11/def12/equip33,40），探针覆盖不到。当前 HpUp 族无此类分支（f54 实测 18 颗全部只加血），
风险低但应记为**证书强度上限**。

### F4 非纯层是否真的不变（绿）

**① 两个独立编译的二进制的整链对比**（不做字段挑选，直接比整个 floor 对象）：

```text
OLD status: capped floors: 3 total_steps: 196 blocked: {floor:54, reason:"truncated", out_hp=2027}
NEW status: capped floors: 6 total_steps: 275 blocked: {floor:3,  reason:"truncated", out_hp=3827}
 floor 51: IDENTICAL=True     ← 含 in_player/out_player/in_flags/out_flags/ents_in/ents_out/route
 floor 52: IDENTICAL=True
 floor 53: IDENTICAL=True
```

`196 → 275`、撞界点 `f54 → f3` 与自述 claim 4 一致；f51/f52/f53 **逐字节相同**（比自述的
「六字段」更强）。

**② 变异 C（同一新二进制、把闭式强制关掉）**——`pure_pickup.rs:436 if policy != OFF` 改成 `if true`：

```text
cmp /tmp/run-mutC.json /tmp/run-old.json  →  MUT-C == OLD: BYTE-IDENTICAL
```

⇒ 本 commit 对非纯层的**接线**（`ExitSolve.pure_pickup` 字段、memo 元组 6→7、闭式分支里的
`LAYER_TRUNC/GOLD_PEAK/SHOP_BUYS` 复位）在关闭闭式后**零侧效应**，整链输出与父 commit 完全一致。
这是「有意变更可追溯、无意外行为漂移」的最硬证据。

**③ f1 两侧对比的边界**：旧 `solver run` 撞在 f54，**从未推进到 f1/f2**，所以 f1 不存在
「旧链值 vs 新链值」的可比口径（新链 f1 的 `in_hp=3227` 来自闭式，旧链根本走不到）。
可判定的是：f1 的闭式入口被 G1 拒（`monster_ents / ty=41`，`tests/pure_pickup.rs` 判据 6 实测），
拒绝路径在 `pure_pickup.rs:508-512` 早退、不写任何搜索状态；加上 ② 的构造性等价，
f1 的行为不可能因本 commit 改变。

### F5 基线诚实性（绿）

`cabb1b3` 的基线文件记 `game_core(lib)=409 / game_solver(lib)=38 / solve_local(test)=24 /
total=1164`；但 `cabb1b3` 的**代码实际**是：

```text
old(cabb1b3) gc lib: 412   new(0a244ff) gc lib: 412
old          gs lib: 46    new           gs lib: 46
old          solve_local test: 25   new: 25
```

⇒ 12 例 = `+3 +8 +1` 是**父 commit 遗留下来、当时没 ratchet-up 的真实测试**，本 commit 把它们
补进基线（`total 1164 → 1183`，`+12 +7(pure_pickup test)`）。逐键 diff 无任何键减少或消失，
`sum(binaries)==total==1183`；在 `/tmp/pprev-new` 复现门禁同款清点
（`cargo test --workspace --release --no-run --message-format=json` + 每 exe `--list`）
⇒ **bins=78 / total=1183 / MISSING 空 / DIFF 空**。

**风险（不是本片的，但必须记）**：`gates/cli.py:266-304` 的 ratchet 是**全 workspace** 口径
（78 个二进制含 `game-desktop`/`android` 等），单跑 `-p game-solver -p game-core` 覆盖不了整条。
未来若某个别的 crate 的测试被删，本片配方不会发现——只有全量 `gates check` 会拦。
本次 78/78 全部存在且计数逐一相符，无「基线里有本片跑不到的幽灵二进制」。

### F6 确定性（绿）

```text
/tmp/pt-new/release/solver run > run1.json ; solver run > run2.json
cmp run1.json run2.json  →  BYTE-IDENTICAL
```

跨进程（两次独立启动）逐字节相同。源码侧：`pure_pickup.rs` **无** `HashMap`/`HashSet`/`f32`/`f64`
（`:64` 注释明确「不依赖 HashMap 迭代序」）；`DIRS` 是固定数组 `:65`；`grid.hazard`/`grid.pickup`
是 `BTreeMap`；出口 `sort_unstable + dedup` 与 `norm` 的 `sort_unstable` 都是全序键
`(ty,x,y,param)`，不含兜底比较。

同进程 memo 冷/热对比（旧侧 p9）：`f54 first 6.109s/2227/26` vs `after-chain 0.011s/2227/26`
⇒ `order_dependent=false`；`memo_key` 含 `node_limit`（`solve_local.rs:700-712`），
不会出现「先小界后大界」串味。

### F7 可观测性缺口（红）

`gates` 之外的直接取证——`solver run` 产物：

```text
top keys: ['status','floors','final','blocked','total_steps','meta']
floor keys: ['entry','ents_in','ents_out','exit','floor','in_flags','in_hp','in_player',
             'kind','ok','out_flags','out_hp','out_player','route','truncated']
grep -c pure_pickup run1.json  →  0
```

代码侧根因：`crates/game-solver/src/witness.rs:157-192` 的 `WitnessFloor` **没有**
`pure_pickup`/`reason` 字段；`:351-352` 逐字段搬运 `got.*` 时也没有 `got.pure_pickup`；
`bin/solver.rs:446/504/508` 只做 `w.to_json()`。⇒ **`ExitSolve.pure_pickup` 在 `record_floor`
处被丢弃**，JSON 与 P5 回放都**无法区分**某层来自闭式还是通用搜索。

危害具体化：本片 f54 的 `out_hp=3227 / route=63` 与通用搜索产物的形态**完全一样**
（同一套 route token 词表、同一个 `ok/truncated` 契约）。P5 若拿 `run1.json` 回放，
只能说「这条路走通了」，无法回答「这是闭式上界还是搜索上界」，也就无法回答
「闭式是否在某层静默改变了语义」。自述在 §? 承认了这个缺口——**承认不等于无害**：
判据 1 的 `pure_pickup=true` 断言只存在于单测（`tests/pure_pickup.rs` test1 第 6 个实参 `toward=Some(1)`
定向到上行梯，并直接断言 `pure_pickup` 与 `stat.reason`）——**产线产物没有等价物**：
同一条 route 从 `solver run` 出来时，谁也看不出它是闭式还是搜索。

### F8 静默失败的残留（绿）

穷举 `pure_pickup.rs` 里所有非成功出口：

- `return give_up(...)` 共 17 处调用点、14 个不同 reason（`policy_not_off / no_level×2 / entry_out_of_bounds / entry_hp_not_real /
  step_arm_hazard_on_entry / step_arm_hazard_reachable / monster_ents / pickup_cell_shared /
  pickup_on_entry_cell / pickup_cell_not_pure / pickup_not_certified×2 / no_certified_pickup /
  hp_overflow / no_stair_exit_reachable / ...`），每处都在 `:358-364` 写 stat（`adopted=false` + reason + ty）。
- `:428-433` 在函数入口就 `attempted: true` 并 `pure_pickup_stat_set(st)` ⇒ 任何早退都留下
  「本层问过、答案是 X」的快照，不会停在上层的旧快照。
- 其余裸 `?`/`return None` 只有 `:239`（`Grid::pure` 邻域边界）、`:299`/`:315`（`pick_sequence`
  内部无解），全部是 `Grid` 纯函数、不碰 stat；调用方用 `note_reason(...)` 落账。
- `hp_overflow`（`:549`）用 `checked_add + give_up`，**不是** `?` —— 自述说的第一处修复确认存在。
- `geo_reach`（`:186-194`）把 hazard 格计入可走 —— 第二处修复确认存在，且 P4/P6 夹具反向证明它真的在起作用。
- `route_construct_failed` 是 `last` 的默认值（`:572`），只有在「一次 `note_reason` 都没触发且
  goals 为空」时经 `:693` 露出 ⇒ 可见，非静默。

`give_up` 14 个 + `note_reason` 10 个，并集 **23** 个具名 reason；加 `not_run`(CLEARED) 后 **24** 个
（与自述 claim 2 的「24 个具名」逐字相符），再加 `route_construct_failed` 默认值共 25 个可能值。
脚本清点见 `python3` 一行：`distinct give_up: 14 / distinct note_reason: 10 / union: 23 / +not_run: 24`。

### F9 引擎侧改动（绿，构造性）

```text
git diff --stat cabb1b3 0a244ff -- crates/game-core   →  （空输出）
```

⇒ 两个 commit 之间 `game-core` **逐字节相同**：不论 74 金样测什么，它们的输入/被测代码都没变，
不可能因本 commit 变红或变绿。

归属澄清：`certified_pure_hp_gain` 是**父 commit `cabb1b3`** 引入的（`cabb1b3 --name-status` 含
`M crates/game-core/src/effect_registry.rs`），本 commit 对 `game-core` 一行未改。
自述「game-core 一行未改」对 `0a244ff` 成立，但 F9 的「只读性」证据必须挂在父 commit 上。

只读性读证（`effect_registry.rs:391-470`）：探针构造局部 `PlayerCombat`（hp100/atk11/def12/
三色钥 3,4,5/gold7/item_types[13,19,15]/uses[2,3,4]/stack3，`tutorial_seen=true`，装备 33/40），
调唯一分发链 `ItemResolver::pickup`，要求 9 字段逐一相等 + `tutorial_seen`/两件装备不变 +
`ItemOutcome::Consumed{hp_popup:Some(d)}` 且 `d==delta*floor_tier(floor)` 且 `hp==100+d`。
全程只操作局部变量，**无 static/thread_local/IO** ⇒ 纯函数语义成立。

### F10 门禁/夹带/并发（绿）

```text
git reflog: 0a244ff HEAD@{2026-11-19 05:12:27} commit: ...   （单条，无 amend/rebase）
            cabb1b3 HEAD@{2026-11-19 03:41:05} commit: ...   （单条）
git show --name-status 0a244ff:
  M crates/game-solver/src/lib.rs
  A crates/game-solver/src/pure_pickup.rs
  M crates/game-solver/src/solve_local.rs
  A crates/game-solver/tests/pure_pickup.rs
  M crates/game-solver/tests/witness.rs
  M data/status/baselines/rust-test-counts.json
  A docs/findings/2026-11-19-pure-pickup-closed-form.md
git status --short（复核收尾）:
  ?? docs/findings/reviews/2026-11-18-boss-drops-p2-review.md
  ?? docs/findings/reviews/2026-11-18-exit-solve-p3-fix-review.md
  ?? docs/findings/reviews/2026-11-18-exit-solve-p3-review.md
  ?? docs/findings/reviews/2026-11-19-witness-p4-review.md
  ?? docs/findings/reviews/2026-11-19-pure-pickup-review.md   ← 本次唯一写入
```

7 个文件全部属于本片，无夹带；4 个他人 review 文档是并行 agent 的未跟踪产物，未被本 commit 触碰。
本仓 `crates/game-solver/tests/witness.rs` 的 3 处 `pure_pickup: false` 是**结构性**（`ExitSolve`
新增字段必须补齐），不构成对既有断言的放宽。

门禁汇总（`CARGO_TARGET_DIR` 隔离到 `/tmp/pt-gate`，避免与并行 agent 抢 `target/`）：

```text
$ CARGO_TARGET_DIR=/tmp/pt-gate python3 gates/cli.py check
[✓ PASS] original-integrity — 1 个归档校验通过
[✓ PASS] assets-manifest — 与基线一致 {'entries': 138, 'classes': 2, 'maplv': 55, 'sprite': 55}
[✓ PASS] reference-seed-integrity — 8457 行，sha256 对照基线通过
[✓ PASS] knowledge-format — 64 条目合规
[✓ PASS] docs-links — 全部相对链接有效
[✓ PASS] naming-ledger — 全部记录带证据指针
[✓ PASS] rust-tests — cargo test --release 全绿
[✓ PASS] test-ratchet — total=1183(基线1183) 无回退
[✓ PASS] table-content-hash — 保真表全集 sha256 对照基线通过（逐值锚 220 宿主）
[✓ PASS] table-evidence — 机械提取值（等值）
[check] ✓ anim_defs/sheet_index/sprite_defs.json 复跑同哈希 + 等值全绿
[✓ PASS] state-machine-anchors — 锚 6 + 边数对照；全命中
[✓ PASS] asset-truth-manifest — entries=138，art 三态核通过
[✓ PASS] state-baseline — [GREEN] 金样逐字节等价：74 场景 sha 全同
== 汇总 ==
PASS=13 FAIL=0 SKIP=0
```

`test-ratchet total=1183(基线1183)` 与我独立复现的 `bins=78 / total=1183 / MISSING 空 / DIFF 空`
互证；`state-baseline` 的「74 场景 sha 全同」把 F9 的构造性论证升级为**实跑证据**。

注：本片复核期内本仓有另一 agent 并行写，故门禁跑在隔离 `CARGO_TARGET_DIR`；
上面是隔离目录下的真实输出，未改任何门禁参数。

---

## 3. 变异实证表

| 变异 | 改法（仅在 /tmp 副本） | 实测结果 | 证明什么 |
|---|---|---|---|
| **A** 漏一颗药 | 靶向表里删一颗 `ty32` | `replay_pickup_left`，`pickups=17 sum=2200 hp_expected=3027`，`out_hp` 回落 2027/truncated | 回放抓漏捡；**不把 3027 交出去** |
| **B** 跳过回放 | 不调 `replay_pickup_route`，伪造终态，且不接「最后→出口」段 | `6 passed / 1 failed`，唯一红：判据 2 `pos=(4,10)`；判据 1 `out_hp=3227` 仍绿，判据 7 整链 `status=blocked floors=4` 仍绿 | **回放是唯一防线**（与自述 claim 3 逐字吻合） |
| **C** 关闭闭式 | `pure_pickup.rs:436` → `if true`（等价 `policy_not_off`） | 新二进制 `solver run` 输出与 `cabb1b3` 二进制 **BYTE-IDENTICAL** | 接线零侧效应；非纯层与旧链完全等价 |
| **O** flags 翻转 | `flags ^= 1` 后再回放 | `replay_flags_changed`，`pure_pickup=false`，回落 2027 | `:637` 是活代码，不是死分支 |
| **P1** 独立回放 | 自写 `try_step` 循环（不用 `pickup_step`） | `hp=3227 pos=(7,1) ents=19 steps=62 flags 不变` | 3227 不是被测代码自己算出来的 |
| **P2-P7** 闸门夹具 | 见 §F2 表 | 6 种攻击面全部具名拒绝 + 1 个负控放行 | G2/G2a/G3 不可绕过 |

关键对照：**变异 B 的 6/7 绿**说明「测试全绿」在本片恰好是危险信号——除判据 2 外没有任何断言
能发现「路线根本没走到出口」；**变异 C 的逐字节相同**说明「非纯层不变」不是靠测试抽样，
而是结构性等价。

---

## 4. 自述与实际不符处

| # | 自述 | 实际 | 等级 |
|---|---|---|---|
| 1 | commit message「`pure_pickup.rs`（692 行）」 | `wc -l` = **696** 行 | 极轻（记账） |
| 2 | 「旧 f54 = `out_hp=2027` / 23 waypoint / **5.82s**」 | 2027/23 是 `toward=Some(1)`（witness 口径、memo 命中）的值，我测 **0.014s**；默认 `toward=None` 200k 同一层是 **2227/26/(7,10)/6.11s**（真玩家 6.30s）。**数值与耗时来自不同口径** | 轻（结论不受影响：两条口径都是 truncated，闭式 3227 逐点支配） |
| 3 | 「`certified_pure_hp_gain` … 除 hp 外 **10 字段**零变化」 | 代码实际断言 9 个状态字段 + `tutorial_seen` + 两件装备 = **12 项**（`effect_registry.rs:391-470`） | 轻（实际更严，非更松） |
| 4 | 「24 个具名 reason」 | `give_up`14 + `note_reason`10，并集 23；+`not_run` = **24**（相符），另有 `route_construct_failed` 默认值共 25 个可能值 | 相符 |
| 5 | 「回放是唯一防线」（claim 3） | 逐字复现（变异 B：6 绿 1 红，红在判据 2） | 相符 |
| 6 | 「f51/f52/f53 六字段全等」 | 比自述更强：**整个 floor 对象逐字节相同**（含 route/ents/player/flags） | 相符（更强） |
| 7 | 「baseline 含上一片遗留 12 例」 | 逐项核实 `+3/+8/+1`，代码实测 412/46/25 vs 旧基线 409/38/24 | 相符 |

**没有发现**：虚报测试、放宽断言、绕过门禁、夹带他人改动、amend/rebase 痕迹。
唯一实质缺口是自述**自己承认**的 F7（`solver run` 无 `pure_pickup` 字段）——按本片任务判据记红。

---

## 5. 未验证清单

1. **`policy_not_off`(G6) 无反例夹具**：只有代码读证（`:436`），没有「Q1/Q2/Q3 下闭式必须不 adopt」
   的测试。风险低（早退必然 `give_up` 且 stat 可见），但缺测试覆盖。
2. **`(7,10)` 出口在 `solve_global` 里的实际影响未测**：我只证明了 goal 集收缩（旧有两条边、闭式一条），
   没有构造「必须从纯层下撤才能拿到更优全局解」的反例。若存在这种层，`solve_global` 会少一条边。
3. **证书的「单状态采样」强度未做对抗**：没有构造「HpUp 药在特定玩家状态下有额外副作用」的夹具
   （`effect_registry.rs` 的探针玩家是硬编码的一组非零值：hp100/atk11/def12/equip33,40）。
4. **跨机器/跨 `--test-threads` 的确定性**：只做了同机跨进程两次 `solver run`（单线程）；
   未做多线程并跑多层共享 `thread_local` 的竞争测试（`pure_pickup_stat` 是 thread_local）。
5. **f54 之外是否存在其它「纯层但入口被夹」**：只逐层看了 f51..f54 + f1/f2；未全量枚举 55 层。
6. **变异 B/C 的产物未入库**：变异体只在 `/tmp` 跑过，仓库里没有可复跑的脚本（复现需照 §3 手工打补丁）。

---

## 6. 两句话结论

**这条闭式解能不能留在主干？** —— **能留**：3227/位置/实体数/flags 已由不碰被测函数的独立回放复现，
非纯层经「关掉闭式后与父 commit 逐字节相同」的构造性证明保证零侧效应，回放被变异实证为唯一且有效的
防线，门禁实跑 `PASS=13 FAIL=0 SKIP=0`；但**落主干前必须补一件事**：把 `pure_pickup`/`reason` 写进
`WitnessFloor`（F7 红——否则 P5 与产线都分不清闭式与搜索），并把「(7,10) 下行边被结构性丢弃」写进
`pure_pickup.rs` 的 G4 注释（现在只有 `step_ok` 的 `cell != entry` 隐含它）。

**f3 的撞界该往哪查？** —— 往**「闭式没覆盖到的层到底在上界内做了什么」**查：f3 是本片唯一因
`node_limit` 撞界（`out_hp=3827` 不可消费）的层，先看它的 `LayerGoal` 集为何没被剪枝收敛（搜索面
`classify` 的 `Move/Pickup/Other` 三分类 + `Key` 不含 `hp/kills/steps` 导致同态膨胀），再判断它是否
本该有闭式（无怪 + 无危险步臂 + 纯药）却被某道闸门误拒——用 `pure_pickup_stat().reason` 打一次即知，
若 `reason` 是 `monster_ents` 就得先解战斗量化，若是 `no_stair_exit_reachable` 则是出口几何问题。
