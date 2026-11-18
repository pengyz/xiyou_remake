# 纯加血拾取的子集支配（D 面）—— 健全性考证 + 实现 + 负结果

日期：2026-11-19 　分支：`modern`（基线 `cf051c4`）　范围：`crates/game-solver/src/solve_local.rs`、
`crates/game-core/src/effect_registry.rs`、`crates/game-solver/examples/profile_floor.rs`、
`crates/game-solver/tests/solve_local.rs`

## 0. 结论摘要

1. **条件 5（"单次 / 固定加血 / 不挡路 / 无其它副作用"）在 f54 上成立，但"固定"要按层读**：
   拾取增益是 `delta × floor_tier(floor)`，**不是**常量 `+50/+200`。`floor_tier` 只取决于
   层号（`(floor-1)/10` 钳位、`floor>50 ⇒ 1`），**同一次单层搜索内恒定** ⇒ 层内每颗药增益
   确实是固定值（f54 ⇒ tier=1 ⇒ `+50/+200`，与既有观测 `Σ=+2400` 吻合）。见 §1.1。
2. 拾取物**不挡路**：独格拾取物所在格在拾取前是 `Cell::Pickup`、拾取后是 `Cell::Move`，
   两者都放行、都 `+1` 步 ⇒ 移除拾取物既不打开也不关闭任何格子。见 §1.2。
3. 拾取臂的**玩家面**副作用恰为「hp += 增益 + 实体移除」；`steps` 增量与普通移动相同。
   但**踩格还有其它臂**（宝箱 5 / 陷阱 60,61,62 / 组门 76 / 场景门 83），其中 5/76/83 会
   改实体表或跑脚本 ⇒ 必须 fail-closed 挡掉。见 §1.3、§3。
4. 规则已实现（`solve_local.rs` 的 D 面，见 §4），5 条判据 + 2 条变异自证全绿（§5）。
5. **负结果（本片最重要的产物）**：本规则在 f51–f54（以及 f1）**一律 fail-closed 关闭**，
   原因 `step_arm_hazard_reachable:83`（每层都有可达的场景门，其脚本可任意写 hp 且输出对
   hp 不保证单调）；**即使**把这道闸拆掉，f54 上它也**原理性剪 0 条**（§7 的定理：无独立 hp
   源时 `hp ≡ hp₀ + Σ已捡`，条件 ③+④ 联立逼出"已检集合相同"，被既有精确键先吃掉）。
   ⇒ f54 的截断**不能靠子集支配解决**，必须换攻法（闭式上界 / 最佳优先）。实测数字见 §6。

## 1. 条件 5 核查（A 级：代码直读）

### 1.1 "固定加血"是否成立 —— 增益随 `floor_tier` 缩放，但层内恒定

`crates/game-core/src/effect_registry.rs`：

- 拾取面真源在 `PICKUP_ARM_TABLE`（:101-143）：ty31 = `("HpUp", EffectParams::HpUp{delta:50})`、
  ty32 = `("HpUp", …{delta:200})`（:129-130）。
- `HpUpEffect::apply`（:262-283）逐字：

  ```rust
  let mut d = delta * floor_tier(ctx.floor);
  if ctx.player.hp <= -d { ctx.player.hp = 1; d = ctx.player.hp - 1; }
  else { ctx.player.hp += d; }
  let popup = if d > 0 { Some(d) } else { None };
  ItemOutcome::Consumed { overlay_tier: None, hp_popup: popup }
  ```

- `floor_tier(floor)`（:157-166）：`t = (floor-1)/10` 钳 0、`floor > 50 ⇒ t = 0`，返回 `t+1`
  ⇒ floor 1..10 = 1、11..20 = 2、…41..50 = 5、>50 = 1。

**裁决（决定规则外形）**：增益**不是**全局常量，`31/32` 的 `×floor_tier` 是真实的。但
`floor_tier` 的自变量**只有层号**，而一次 `solve_layer` 只跑一层 ⇒ 层内 `tier` 恒定
⇒ **层内每颗药的增益是固定常量**。因此规则**不需要跨层归一**，只需把"固定"的口径定为
"同层固定"，并在计划阶段用 `certified_pure_hp_gain(ty, floor)` 把本层常量算出来写进计划。

给这一类增益赋值的机制也**只有** `ItemResolver::pickup` 一条链
（`crates/game-core/src/subsystem/item_service.rs:25`）：
`pickup_kind_for_type(t)` → `pickup_params_for(t)` → `builtin_effect(kind).apply(ctx, &params)`。
`ItemCtx` 的 `equip_types/equip_bonuses` 是**按值复制**，效应体拿不到外层 ⇒ 不可能有隐藏副作用。

> 补充（易混淆）：`crates/game-core/src/item_effect.rs` 的 `ITEM_EFFECTS` 是**背包使用面**
> （`use` 道具，例如 ty19 = `Heal{mul:74,div:10}` 按 `(atk+def)` 算），与地面拾取无关；
> 两条面并存，勿互相代入。

### 1.2 "不阻挡通行"成立

- 通行判定：`solve_local.rs:1563 fn walkable` 只看 `level.terrain`；`classify`（:1595-1665）
  先看同格实体：`cat2|4` 记 `pickup=true`，最后 `(None,true)` ⇒ 若该型不在 `policy.ban` 里
  则 `Cell::Pickup`，否则 `Blocked`。
- `expand` 的 `Cell::Pickup` 臂（:2072-2081）与 `Cell::Move` 臂走同一套 `pickup_step`/`move_step`，
  都产生 `+1` 步的 token。
- 移除拾取物后同格分类从 `Pickup` 变成 `Move`（或反方向）——**两者都放行**。
  ⇒ 「移除一颗独格药」不改变任何格子的可通行性，也不改变步数代价。

### 1.3 "拾取无其它副作用"成立，但**踩格**有其它臂

`crates/game-core/src/scene.rs:3889 apply_step_cell_effects` 顺序（A）：

1. 陷阱（邻格判定）：ty61 两侧都有 ⇒ `apply_hp_delta(-(hp>>1))`；ty62 邻格 >0 ⇒ `-100`；
   ty60 邻格 >0 ⇒ `-200` **且** `step_triggered=true`，若上邻格也是 60 则
   `fire_trap_climb_up(...)`（**改实体表**）。
2. 本格实体槽倒序：`cat1` 的 `6`（喷泉 bob）/ `83`（`try_run_scene` + `load_level_script`，
   **跑脚本**）/ `7,8`（换层）/ `5`（宝箱，第二轮扫描里 `remove_slot_at` + `try_open_chest`）/
   `76`（组门：`open_group.clear()` + 把同组实体收进 `open_group` + **该格全实体 removed**）。
3. `cat2|4` ⇒ `pickup_item_type(t)` + `entity::remove`。
4. `walk_phase != 4 && !step_triggered` ⇒ `handle_field_input()`——`pickup_step` 里
   `game_mode=Map`、`key_value=0`、`key_held=0` ⇒ 该函数**纯 no-op**（:3134 的 `_ => return`）。

`pickup_item_type`（:4123）三步：`menu.set_overlay_kind_item`（**menu 态，不被 `pickup_step`
读回** ⇒ 搜索不可见）→ `ItemResolver.pickup` → 弹字/文案（纯 UI）。
`pickup_step`（`solve_local.rs:2496-2584`）读回的只有 player 全字段 + `readback_ents` +
`slots.script_line_flags`，并 `ns.steps += 1`。

⇒ 拾取臂的可见副作用恰为 **hp 与药实体**。但同一格上若还有 **5/76/83**，`Pickup` 与 `Move`
两条路径的副作用集**不同**（`Move` 不进 `pickup` 分支，但同样会跑 1./2. 的臂——注意 2. 的臂
与 pickup 无关，两条路径都会触发；真正的差别在**药实体是否被移除**）。为避免纠缠，闸门 G4
直接要求这些"踩格副作用臂"实体**不可达**，否则整层退回通用搜索。

> 保守度声明：60/61/62 的伤害对 hp **单调**（`-常量`、`-⌊hp/2⌋`，且下限钳到 1 ⇒
> `max(1, x-d)` 关于 x 单调不减），按本片的分析其实可放行；但 60 会改实体表、且本片未单独
> 证明"陷阱格位与药位不相交"，故与 5/76/83 一并挡掉（fail-closed 优先）。

## 2. 规则与健全性证明

**桶键** `PillKey = key_of_ents(st, 非药实体)`：把"药维实体"从 `ents` 里剥掉，其余
（`pos / flags / 三钥 / 背包 / atk / def`）照旧。**剩余药**用 `u32` 位图 `rem` 表示
（bit i = 第 i 颗入场药还在，按 `root.ents` 顺序静态编号）。

**谓词**（`solve_local.rs:1281`）：

```
pill_dominates(a, a_rem, b, b_rem) =
     hp(a) ≥ hp(b) ∧ kills(a) ≤ kills(b) ∧ steps(a) ≤ steps(b) ∧ gold(a) ≥ gold(b)
   ∧ (b_rem & !a_rem) == 0        // rem(a) ⊇ rem(b)：a 捡得更少
```

**断言**：满足上述条件时 `a` 不劣于 `b`，`b` 可剪。

**证明（对 b 的动作序列做逐步模拟）**。同桶 ⇒ 比较时刻 `pos/flags/三钥/背包/atk/def/
非药实体` 全等。设归纳假设 `hp(a) ≥ hp(b)` 且 `rem(a) ⊇ rem(b)`，其余桶内字段相等：

- **移动**（`Move`）：两边同格、同分类、同 `+1` 步；hp 不变 ⇒ 假设保持。
- **拾取**：b 走到某药格 P（`P ∈ rem(b)`）时，a 的同一格分类也是 `Pickup`（非药实体全等、
  药位图身份静态）⇒ a 也拾到该药，得 `g_P > 0`；`rem(a) ⊇ rem(b)` 且 `P ∈ rem(b)`
  ⇒ `P ∈ rem(a)` ⇒ a 也移除它 ⇒ `rem(a)\{P} ⊇ rem(b)\{P}`；hp 差 `(g_P - g_P) = 0`
  ⇒ `hp(a) ≥ hp(b)` 保持。若 b 站在已被自己拾过的药格上（无事发生），a 那边即使还有这颗药
  也只会**多拿** hp（`g>0`）⇒ 不等式只会更松。
- **战斗**：拒战判据是 `loss ≥ hp`（`crates/game-core/src/subsystem/battle.rs:121-123`），
  且 `predict_engagement`（:85-103）只用 `atk/def/敌表`、**不用 hp** ⇒ 同一 action 两边
  `loss` 相同；`hp(a) ≥ hp(b)` ⇒ b 打得动 ⇒ a 也打得动，胜负与 `loss` 相同 ⇒
  `kills` 同增、hp 同减 ⇒ 假设保持。
- **门 / 商店 / 墙 / 梯 / 终止**：只看 flags/keys/gold/非药实体 ⇒ 两边同判。
- **`rem` 位图合法性**依赖「药实体身份静态」：实体不移动、只减不增，且**药格不被其它实体共享**
  （闸 G3）、掉落不会落在药格或生成药（闸 G5）⇒ 成立。

终态：a 到达 b 所到出口时 `hp(a) ≥ hp(b)`、`steps(a) ≤ steps(b)`（两边步数增量相同）
⇒ 按取解序 `Cost::HpLoss`（hp 大 → steps 小 → route 字典序，`solve_local.rs:1281-1296`）
a 的解不劣于 b ⇒ 剪 b 安全。∎

**方向不可反**（判据 2 钉的就是这个）：只要求 `hp(a) ≥ hp(b)` 或只要求 `rem(a) ⊇ rem(b)`
都**不**成立——b 可以"回去把 a 已经花掉的血补回来"。

## 3. fail-closed 闸门（任何一条不满足 ⇒ 整层退回通用搜索，并留下具名原因）

`plan_pill`（`solve_local.rs:2288`）按顺序判定，返回 `PillDom::On(ord)` 或
`PillDom::Off(reason, reason_ty)`：

| 闸 | 条件 | 原因串 |
|---|---|---|
| G0 | 层里至少有一颗"被背书的纯加血拾取物" | `no_certified_pill` |
| G1 | 每个 `kind == "HpUp"` 的拾取型都必须拿到 `certified_pure_hp_gain(ty, floor)`（本层固定 `g>0`） | `hp_gain_not_certified` |
| G2 | 该型不得出现在 `policy.ban` 里（禁 ⇒ `Blocked`，a 进不去而 b 能 `Move`） | `pill_banned_by_policy` |
| G3 | 药实体数 ≤ 32（位图容量） | `too_many_pills` |
| G4 | 药格必须**独占**（层静态表 ∪ 入场活体按 `EntRec` 去重后，该格只有这颗药） | `pill_cell_shared` |
| G5 | 踩格副作用臂实体（`STEP_ARM_HAZARD_TYPES = [5,60,61,62,76,83]`）必须**不可达**（`classify` 返回 `Blocked`） | `step_arm_hazard_reachable` |
| G6 | 怪物掉落（`boss::event_of` / `post_victory_event_of` → `event_spawns`）不得生成 hazard 型或药型 | `drop_breaks_pill_identity` |

`certified_pure_hp_gain`（`effect_registry.rs:391`）不是查表，而是**运行时探针**：
在一个非默认探针玩家上用 `ItemResolver::pickup` 真跑一次拾取臂，要求「除 `hp` 外 10 个字段
零变化、`tutorial_seen/weapon/armor` 不变、`ItemOutcome` 的 `hp_popup` 恰为
`delta × floor_tier(floor)`、`hp` 恰增那么多」才发证。任何未来新增的副作用都会让它自动变成
`None` ⇒ 触发 G1 关闭规则（而不是带着错误假设继续剪）。

## 4. 实现落点与可观测

| 位置 | 内容 |
|---|---|
| `solve_local.rs:1158` `key_of_ents` | 从 `key_of` 拆出（`key_of` = `key_of_ents(st, st.ents.clone())`，既有语义零变化） |
| `solve_local.rs:1233-1310` | `PillGains` / `PillDom` / `PillOrd{bits,gains}` + `split` + `pill_dominates` + `pill_scan` |
| `solve_local.rs:1321` | `STEP_ARM_HAZARD_TYPES` |
| `solve_local.rs:1325` | `PILL_CMP_BUDGET = 4_000_000`（比较预算：超了只停止剪枝，不影响正确性/确定性） |
| `solve_local.rs:1330-1380` | `PillDomStat` + `thread_local PILL_DOM` + `pub fn pill_dom_stat()` |
| `solve_local.rs:2288` | `Search::plan_pill`（§3 的闸门） |
| `solve_local.rs:2408` | `run_root`（转发，注入真证书） |
| `solve_local.rs:2433` | `run_root_with(..., cert: &dyn Fn(i32,i32)->Option<i32>)`（可注入假证书 ⇒ 测试能造"未背书"） |
| `solve_local.rs:2677` | D 块本体：**在既有精确键去重块之后**，桶 `HashMap<Key,Vec<(u32,u32)>>` |
| `solve_layer_with` | 入口清零 stat；`LAYER_MEMO` 值加一维 `PillDomStat`，**memo 命中时回放**（禁静默） |
| `profile_floor.rs` | 目标层隔离重放后打印 `pill_dom: enabled/reason/reason_ty/pill_types/pills/states/cmp/pruned_new/killed_old/budget_exhausted` |

既有精确键逻辑**未改**：`weakly_ge`/`dominates`/`key_of` 的调用与注释一字未动，D 块在其后
另起（D 天然看不到被精确键吃掉的候选）。`ok`/`truncated`/`ExitSolve` 契约、默认层界、
`SOLVER_LAYER_LIMIT` 语义均未改。

## 5. 判据实跑

`cargo test -p game-solver --lib pill_subset_dominance` ⇒ **8 passed**（`solve_local.rs:3480`）：

| # | 判据 | 结果 |
|---|---|---|
| 1 | `pill_subset_dominance_prunes_dominated_state`（hp 高且剩药多 ⇒ 剪新态） | ok |
| 2 | `pill_subset_dominance_keeps_better_hp`（hp 低但药多 / hp 高但药少 ⇒ 都不剪） | ok |
| 2' | `pill_subset_dominance_needs_subset_not_just_count`（不可比 ⇒ 不剪） | ok |
| 2'' | `pill_subset_dominance_respects_step_budget`（血多步少药多 ⇒ 剪） | ok |
| 3 | `pill_plan_fails_closed_when_gain_not_certified`（假证书 ⇒ `Off(hp_gain_not_certified,31)`；真证书 ⇒ `On` 且 `gains[31]=50`） | ok |
| 3' | `pill_plan_fails_closed_when_step_arm_hazard_reachable`（ty61 可达 ⇒ `Off(step_arm_hazard_reachable,61)`） | ok |
| 4a | `pill_dominance_off_matches_off_path_field_by_field`（同层 On vs Off 逐字段差分） | ok |
| 4b | `pill_subset_dominance_does_not_change_exact_results`（**真实 f1**：out_hp/route 与既有 `solve_local` 一致 + memo 回放 stat） | ok |

**变异自证**：

- **M1**：`pill_dominates` 的 `hp >=` 改成 `hp >` ⇒ **仍 8 passed**。等价理由：唯一失去的是
  「hp 相等 + 严格 `rem` 超集」这一角，而增益全为正 ⇒ 严格子集的 Σ 不可能相等 ⇒ 该角在引擎内
  不可达。
- **M2**：`rem` 条件反转（`(b_rem & !a_rem)==0` → `(a_rem & !b_rem)==0`）⇒ **红 3 条**
  （`prunes_dominated_state` / `keeps_better_hp` / `respects_step_budget`），其余 5 条仍绿
  ⇒ 方向被钉住。
- **M3（误伤对照，意外收获）**：第一次改错了函数（把 `weakly_ge` 的 `hp >=` 改成 `>`，
  `solve_local.rs:1189`）⇒ 判据 4a/4b(单测7) 立刻红（10k 界下 On 腿只出 1 个 goal，Off 腿 12 个）
  ⇒ 说明这两条差分测试**对搜索剪枝强度敏感**，不是空过。

## 6. 实跑对照（改动前后）

### 6.1 `profile_floor 54 200000`

| 指标 | 改动前 | 改动后 |
|---|---|---|
| 目标层隔离重放 | `ok=false truncated=true out_hp=2027 waypoints=23 exit=(7,1) kind=Stair **6.12s**` | 同上，**6.04s** |
| 整例 real | 9.513s | 9.516s |
| `pill_dom` | （无字段） | `enabled=false reason=step_arm_hazard_reachable reason_ty=83 pill_types=0 pills=0 states=0 cmp=0 pruned_new=0 killed_old=0` |

⇒ **f54 仍截断，数字完全不变**：本层因可达的场景门（ty83）而 fail-closed，D 块根本没参与。

### 6.2 `solver run --out`

见 §6.3 的对照（`status=capped floors=3 total_steps=196`，前后一致）。

### 6.3 真实层的闸门现状（`profile_floor` 逐层）

| 层 | `ents_ty_hist` 摘要 | 药 | `pill_dom` |
|---|---|---|---|
| f1 | 含 ty83 | ty31×2 | `Off(step_arm_hazard_reachable,83)` |
| f52 | 含 ty83×4 | ty32×3 | `Off(step_arm_hazard_reachable,83)` |
| f53 | 含 ty83×1 | ty32×2 | `Off(step_arm_hazard_reachable,83)` |
| f54 | ty6×16 ty7 ty8 ty31×8 ty32×10 **ty83×1** | 18 | `Off(step_arm_hazard_reachable,83)` |

⇒ 「场景门（ty83）在真实层普遍可达」是本规则无法上场的**唯一**原因；其脚本可任意写 hp，
且输出对 hp 不保证单调，故不能从"实体集相同"推出"两边脚本效果可比"。

## 7. 负结果：即使拆掉 G5，f54 也剪 0 条（定理）

f54 **无怪、无陷阱、无商店买血**（§6.3 的 `monster_ents=0`，且实体表里没有 60/61/62），
拾取是**唯一**的 hp 写入 ⇒ 对任一状态 `hp = hp₀ + Σ_{已捡} gain`。设同桶两态 `a,b` 满足
`rem(a) ⊇ rem(b)`（即已捡(a) ⊆ 已捡(b)）且 `hp(a) ≥ hp(b)`：
`Σ已捡(a) ≥ Σ已捡(b)` 且 `gain > 0` ⇒ 已捡(a) = 已捡(b) ⇒ 药维实体相同 ⇒ `ents` 全等
⇒ **已被既有精确键块处理**（在 D 块之前），D 块看不到这对。

代码化证据：`pill_dominance_is_vacuous_without_independent_hp_source`（13×3 走廊 + 两颗药 +
出口，真证书）⇒ `enabled=true, pill_types=2, cmp>0, pruned_new=0, killed_old=0`。

⇒ **工程结论**：f54 的 195174 个唯一键里，"药维"这条维度不能靠子集支配压掉——它在
"无独立 hp 源"的层上是**守恒量**（`hp` 与"剩余药价值"此消彼长）。要压掉它必须换攻法：
(a) 对"纯拾取 + 纯走位"的子层求**闭式上界**（可达药的价值和）来剪枝；(b) **最佳优先**
（按 `hp + 剩余可达药价值` 的乐观势函数）替代 BFS，让撞界前先取到高血解。

## 8. 假设 / 未查清

- **假设（B 级）**：`policy.ban` 之外不存在"某些层禁拾取"的运行时开关。已核 `classify`/
  `expand` 的 `Cell::Pickup` 臂都不看 `policy.pickup`，`solve_layer` 传 `ClosurePolicy::OFF`
  ⇒ 判据成立；若将来引入运行时禁拾取，G2 需扩展。
- **未证明（保守挡住）**：陷阱 60/61/62 的伤害对 hp 单调，因此**理论上**可以放行；本片未做
  「陷阱格位与药位不相交」的单独证明，故一并 fail-closed。
- **未证明**：场景门 83 / 宝箱 5 / 组门 76 的脚本**对 hp 盲目**（若某层的这些脚本既不含 hp
  读写、也不引用药实体，则 G5 可以收窄到"该实例"）。这是把本规则救回真实层的唯一低成本路径，
  需要脚本 DSL 的静态"只读校验"（本片未做）。
- **未测**：`PILL_CMP_BUDGET` 的实际耗时——真实层全部 fail-closed，`cmp` 恒 0，预算未被消耗；
  在 `Off` 层上 D 块零开销（只多一次 `plan_pill` 的层扫描）。

## 9. 复现

```bash
cargo test -p game-core --lib pill_cert                                  # 证书 3 条
cargo test -p game-solver --lib pill_subset_dominance                     # 判据 8 条
cargo test -p game-solver --test solve_local pill_subset_dominance_does_not_change_exact_results
cargo test -p game-core && cargo test -p game-solver                      # 全量
cargo run --release -p game-solver --example profile_floor -- 54 200000   # 含 pill_dom 行
python3 gates/cli.py check
```
