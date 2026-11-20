# 定向对抗复核：P9 整改三笔（`127a9cb` / `fc22c44` / `fa30c94`）

日期：2026-11-20　复核者：独立对抗性定向复核 agent（只读 + `/tmp/y` 变异，未改仓库代码/commit/push）
对象：`127a9cb`（fix solver：正向支配 + shops 透传 + 护栏判据）、`fc22c44`（docs 就地更正）、
`fa30c94`（ratchet 1231→1237），父 `76734f6`
上一轮：`docs/findings/reviews/2026-11-19-exact-pickup-review.md`（红 F4；黄 F3/F8、F6、F10）
相关：`docs/findings/2026-11-19-exact-pickup-dp.md`、`2026-11-19-hp-bound-p8b-spawn-gate.md`

复核纪律：不采信实现者自述；每条判定给出**独立复算、变异或代码行级**证据。所有变异在
`/tmp/y`（`git archive HEAD | tar -x -C /tmp/y` + `assets/raw` 软链 + `CARGO_TARGET_DIR=/tmp/yt`）
里做，做完即还原；仓库工作区除本文件外零改动。

---

## 0. 判定汇总

| # | 面 | 判定 | 一句话 |
|---|---|---|---|
| G1 | 红 F4 真修了吗 | **绿** | 反向 dead 段整段删除且无复活（全文件无 `dead[..]=true`/`retain`）；f3 实测 88/14/4,741,000 与复核要求逐字一致 |
| G2 | 支配剪枝现在健全吗 | **绿** | 正向支配（子集后到被超集剪）在「FIFO 步数非降 + 六轴单调 + 撞怪必须零伤」三前提下健全；不健全的是**目标函数**（见 G6），不是剪枝 |
| G3 | 88/14 真可行、真 4027、少捡的 4 件 | **绿** | 回放**真驱动引擎**（变异 `pickup_step` 臂 ⇒ `replay_failed` + 红）；少捡恰为 3 黄钥 + 1 def；4027 的 800 全部来自被捡的 4×ty31 + 3×ty32，与弃件无关 |
| G4 | shops 口径真透传了吗 | **黄**（机制绿 / 归因措辞错） | 透传调用链与判据都真实存在、机制判据与 `scene.rs` 一致、未建模边界已如实披露；但「8/54 高估的根因是 shops 表被丢弃」这条**归因可证伪**（新判据下空表与真表结果完全相同） |
| G5 | 黄 F6 判据真能抓静默吗 | **绿** | 把 `state_cap` 改成 `Ok(None)`、`gate_budget` 改成 `break` ⇒ 两条判据各自变红（`exit_unreachable` ≠ 具名理由） |
| G6 | 链路退回 floors=7 与归因 | **黄**（现象绿 / 数量错） | `floors=7 blocked=floor 4 reason=no_exit_toward` 独立复现属实、归因方向成立；但「f4 需要 3 把黄钥匙」是错的，**最小是 2 把**（DP + 独立 BFS + 手算三重一致）；结论不变（实际余量 0 < 2） |
| G7 | 夹带与诚实 | **绿** | 三笔零越界（9 个 game-solver 文件 / 2 个 docs / 1 个基线）；docs 更正**零删除行**（§3.6 满足）；无新增 `#[ignore]`；无净测试删除；ratchet 1237/bins 81 实测一致；`check`=13 门 SKIP=0 / `status`=14 门 SKIP=1 的撤回经静态核对成立 |

**计数：绿 5 / 黄 2 / 红 0。**

**红 0**：上一轮唯一的红 F4 已真修，且修法比「把 steps 并入谓词」更彻底（删反向段）；我用 M1 复原旧段
能让判据回到 106/18，说明这不是「换个数字糊弄」，而是行为真变了。

**两处黄都是「口径」，不是机制**：G4 是文档归因句与机制不符（需补一行更正）；G6 是一个数字
（3 把 → 2 把）与一个**已披露**的遗留项（逐层 `(hp, steps)` 目标不含跨层钥匙盈余）。

---

## G1 红 F4 真修了吗

**判定：绿。**

### G1.1 反向段确已删除、且没有换个地方复活

```bash
git show 127a9cb -- crates/game-solver/src/exact_pickup.rs | sed -n '/fn push/,/^@@/p'
git show 127a9cb -- crates/game-solver/src/exact_pickup.rs | grep -E "^-" | grep -v "^---"
grep -n "dead\[.*\] = true\|front.retain\|\.retain(" crates/game-solver/src/exact_pickup.rs
grep -n "dead" crates/game-solver/src/exact_pickup.rs
```

删除的正是反向段本体：

```
-        // 反向：本态 pm ⊃ 已有的 pm₂ ⇒ 后者失效（标记 dead，不再展开/参评）
-        let front = self.front.entry(kk).or_default();
-        for &(pm2, j) in front.iter() {
-            if n.pm & !pm2 != 0 {
-                self.dead[j as usize] = true;
-            }
-        }
-        front.retain(|&(pm2, _)| pm2 & !n.pm != 0);
-        front.push((n.pm, i));
```

当前 `crates/game-solver/src/exact_pickup.rs:661-684` 只剩：`index` 精确去重 → 正向 `front.iter().any(|&(pm2,_)| n.pm & !pm2 == 0) ⇒ Ok(None)` → `cap_reached() ⇒ Err(("state_cap", len))` →
`front.entry(kk).or_default().push((n.pm, i))` / `states.push` / `dead.push(false)`。
`grep dead` 只剩 `:464`（字段声明）、`:518`（`dead: Vec::new()`）、`:680`（`dead.push(false)`）、`:296`/`:746`（两个**读点**）；
`grep "= true\|retain"` **零命中** ⇒ 没有任何地方再把 `dead` 置真，`if dp.dead[i]` / `if self.dead[i]` 恒 false。
`:58` `MAX_STATES: usize = 6_000_000`（2M→6M 属实）。

> 附注（不阻塞）：`dead: Vec<bool>` 现在是**残留字段**（永假、永不写），建议后续清理时删掉，
> 否则下一个人会以为还有一条「dead 剪枝」在跑。

### G1.2 f3 实测值与复核要求逐字一致

```bash
cargo test --release -q -p game-solver --test exact_pickup exact_dp_f3_reaches_4027 -- --nocapture
```

```
[P9-1] f3 ok=true trunc=false out_hp=4027 exit=Some((11,11)) exact=true reason="adopted" stat=ExactStat{out_hp:4027, steps:88, picks:14, doors:5, kills:7, states:4741000, replay_steps:88, route_len:88}
test result: ok. 1 passed; finished in 2.79s
```

`88 / 14 / 4,741,000 / 4027` —— 与上一轮复核 F4 用「删掉反向段」探针得到的值**完全一致**
（不是被告自己另立的口径）。

---

## G2 支配剪枝现在健全吗（本次最重要）

**判定：绿**（对**已实现的目标函数**健全）。这里给证明 + 三个前提的代码位点，再指出
「真正不健全的是目标函数」这一残余风险（G6 会用到）。

### G2.1 现在的剪枝只有一条，且是正向

`crates/game-solver/src/exact_pickup.rs:661-672`：

```rust
fn push(&mut self, n: SNode) -> Result<Option<u32>, (&'static str, i32)> {
    let k = self.pack(&n);
    if self.index.contains_key(&k) { return Ok(None); }      // 同 (pos,pm,dm,km)：first-come 即最少步
    let kk = self.pack_no_pm(&n);                            // 键 = pos|dm|km（不含 pm）
    if let Some(front) = self.front.get(&kk) {
        if front.iter().any(|&(pm2, _)| n.pm & !pm2 == 0) {   // 新态 pm ⊆ 已入队 pm₂ ⇒ 剪
            return Ok(None);
        }
    }
    ...
```

`front` 只追加（`:679` `entry(kk).or_default().push(...)`），从不删；`index` 是 `(pos,pm,dm,km)` 全键。

### G2.2 健全性论证（四个前提，逐个落到代码）

要证明「剪掉 `pm ⊂ pm₂` 的后到态不丢失 `(hp 最大, steps 最小)` 的最优解」，需要：

1. **步数非降（FIFO）**：`search()` 用 `VecDeque`，`:744` `q.pop_front()` / `:817` `q.push_back`；
   所有转移共用同一前缀 `nn.steps = n.steps + 1`（撞怪臂 `:745-751` 显式写 `nn.steps = n.steps + 1`
   且不移动——注释「撞怪：不移动，只加一步 + 置杀位」）。⇒ 任何已在 `front` 里的态
   `steps₂ ≤ n.steps`。这正好补上反向剪枝缺的那条不等式。
2. **六轴单调**：`pm₂ ⊇ pm ⇒ hp₂ ≥ hp₁`（`hp_at:542 = player0.hp + Σ_{i∈pm} gains[i].hp`，
   增益恒非负由 `certified_monotone_pickup` 认证，`crates/game-core/src/effect_registry.rs:593`
   「④ 六轴必须非降（任一轴下降 ⇒ 不是单调增益 ⇒ fail-closed）」）、`atk/def_at:552`、
   `keys_at:573` 同理逐轴 ≥；`dm` 相同 ⇒ 门位图相同 ⇒ **可走格集相同**。
3. **撞怪零伤（这是最容易被忽略的一条）**：`bump():686` 用 `atk_def_at(n.pm)` 算损血，
   `:704 if loss != 0 { return Err(("monster_damage", ty)) }` ⇒ **精确 DP 只在「所有实际要撞的怪零伤」
   的层里接管**，hp 因此是 `pm` 的纯函数（与拾取顺序无关）。于是 `atk₂ ≥ atk₁ ⇒ 损血₂ ≤ 损血₁ = 0`，
   共有后续路线在超集上仍然合法。（若允许非零损血，`hp_at(pm)` 纯函数假设就会破，剪枝立刻不健全——
   这条约束是 O(cost) 的，不可去掉。）
4. **拾取物不挡路、实体表由掩码唯一决定**：拾取格是 cat2/4（可走、踩即拾），`ents_at:608`
   按槽位从 `(pm,dm,km)` 确定性重算 ⇒ 同一延伸在超集上「少几个待捡物、门/怪相同」，终态
   `hp' = hp0 + Σgains(pm₂ ∪ future) ≥ Σgains(pm ∪ future)`、`steps' = steps₂ + L ≤ steps_n + L`。

⇒ 被剪掉的态，其**任意**后续都有超集态的同形延伸，两目标（hp 越大越好、同 hp 步数越小越好）都不劣
⇒ 剪枝健全。

### G2.3 反面证据仍在（M1：把旧段贴回去 ⇒ 判据立刻红）

在 `/tmp/y` 复原反向段后：

```bash
cargo test --release -q -p game-solver --test exact_pickup exact_dp_same_hp_fewer_steps_survives exact_dp_f3_reaches_4027
# exact_dp_same_hp_fewer_steps_survives ... FAILED  left: 10 right: 6
#   报错：同 hp 下必须取最少步：6 步；10 步是绕路捡 ty29 的解
# exact_dp_f3_reaches_4027 ... FAILED  left: (106, 18) right: (88, 14)
#   stat=ExactStat{steps:106, picks:18, states:130835, ...}
```

⇒ 判据 1b 与 1 都能真正抓回退，而且 106/18/**130,835** 正是旧版原值（说明复原精确）。

### G2.4 残余风险（**不属剪枝**，属目标函数）

剪枝对「层内 `(hp 最大, steps 最小)`」是健全的，但这个目标**看不见跨层钥匙盈余**：
f3 的 88/14 最优解出门时黄钥匙余量 **0**（实测 `out_player.yellow = 0`），而 f4 上行最少要
**2** 把（G6.3）。这不是剪枝错了，是「层内最优 ≠ 全链可行」的结构冲突，文档已列为遗留项
（`docs/findings/2026-11-19-exact-pickup-dp.md:27`「逐层 `(hp, steps)` 目标与跨层钥匙需求冲突，本片未修」）。

---

## G3 88/14 路线真可行、真 4027、少捡的 4 件是什么

**判定：绿**（三项子问题全部实证）。

### G3.1 回放**真驱动引擎**（不看自述，用变异证明）

`exact_replay_route`（`crates/game-solver/src/solve_local.rs:4374`）逐 token 走三臂：`bump`→`exact_bump`、
`door`→`exact_open_door`+`gate_arrive`、普通格→`s.classify` 后 `pickup_step`/`Move`。
变异：把 `Cell::Pickup` 臂的 `st = s.pickup_step(&st, x, y)?;` 换成「只改坐标 + 步数+1」（模拟「回放没真跑引擎」）：

```bash
cargo test --release -q -p game-solver --test exact_pickup exact_dp_f3_reaches_4027 -- --nocapture
```

```
exact_pickup: floor=3 entry=(2, 11) replay_failed/0 —— DP 预测与真引擎回放不一致，本层 Off
[P9-1] f3 ... exact=false reason="replay_failed" stat=ExactStat{adopted:false, reason:"replay_failed", ...}
thread 'exact_dp_f3_reaches_4027' panicked ... FAILED
```

⇒ 采用（`adopted`）是被**真引擎回放**卡着的：回放不跑引擎，f3 立刻掉到 `Off(replay_failed)`。
`run()` 尾段顺序也印证：先逐出口取优，再 `validate_outcomes(...)`（任何不一致 ⇒ `give_up`），
`replay_steps` 由回放实际步数写入，判据断言 `st.replay_steps == st.steps`（本次 88 == 88）。

### G3.2 少捡的 4 件 = 3 黄钥匙 + 1 def，且不影响 4027

用 `/tmp/y` 的 `f3_route_probe` 把 88 token 还原成访问格集合，与全部实体比对：

```
ty=26 ( 6, 1) taken=false     ← 黄钥匙
ty=26 ( 4, 3) taken=false     ← 黄钥匙
ty=26 ( 1, 1) taken=false     ← 黄钥匙
ty=30 ( 2, 1) taken=false     ← def+1（gain Def(1)）
其余 14 件 taken=true：4×ty31(+50hp) + 3×ty32(+200hp) + 5×ty26 + ty29(+atk) + ty28(蓝钥)
```

`4027 - 3227 = 800 = 4×50 + 3×200` —— **全部来自 ty31/ty32**，被弃的 3 黄钥 + 1 def 对 hp 零贡献。
⇒ 「少捡导致 4027 掉了」不成立；少的 def 只体现在出门态 `def 731 → 730`（旧 18 件路线捡了 ty30）。

### G3.3 但「少捡」的代价落在**链路上**：出门黄钥匙余量 0

`/tmp/y_run.json` 的 f3 记录：`in = hp 3227 / atk 730 / def 730 / yellow 0 / blue 0 / gold 225` →
`out = hp 4027 / atk 731 / def 730 / yellow 0 / blue 1 / red 0 / gold 250`。

88 步路线开 5 扇黄门、路上捡 5 把黄钥 ⇒ **余量 0**；旧 18 件路线捡 8 把、开 5 扇 ⇒ 余 3。
这正是 G6 的根因，也说明「88/14 是层内最优」与「全链可行」是两件事。

---

## G4 shops 口径真透传了吗

**判定：黄**（透传与机制绿；文档里那条**归因**可证伪，需就地补一行更正）。

### G4.1 调用链逐段核（真实存在，不是摆设）

```bash
grep -n "shops" crates/game-solver/src/bin/solver.rs crates/game-solver/src/witness.rs | head
```

- `bin/solver.rs:462-482`：`let (gi, shops) = solve_global_input(1);` → `run_witness(..., &shops, &cfg)`
  （旧版此处是 `let (gi, _shops)`，确实改对了）；
- `witness.rs:325` `type LayerSolver = ... &[ShopGoodDef]`；`:493` `run_witness_with` 委托
  `layer(input, entry, &cur_player, cur_flags, ents_here.as_deref(), next_up, shops)`；
- `solve_local.rs:399 solve_layer_shops` / `:563 solve_local_exit_toward_shops` → `solve_layer_with(..., shops)`
  → `:825` 传给 `exact_layer_view`/`:853 search.shops = shops`，并进 memo 键 `shops_key`（`:757`）；
- 判据 `tests/pure_pickup.rs::witness_forwards_real_shop_table_to_layer_solver` 用记录式层解器断言
  **长度 17 + 首地址**就是调用方那片表（不是复制、不是空表）。

### G4.2 机制判据与引擎一致

`solve_local.rs:4122`：

```rust
fn shop_can_buy(param: i32) -> bool { let flags = (param >> 8) as i8; let idx = param as i8; idx > 0 && flags & 2 != 0 }
```

对照 `crates/game-core/src/scene.rs`：
`:2522 fn apply_merchant_offer(&mut self, buy: bool) -> bool`、`:2537-2538 if idx <= 0 { return false }`、
`:2542 if flags & 2 != 0 {` 才进购买面（查 `data/tables/shop_goods.json` 同 src/idx 行、扣金/发效果）、
`:2592-2612` 收尾在 `flags&2` 门内（`flags&4==0 ⇒ param ^= 512`）、
`:2629 } else if flags & 2 == 0 && flags & 1 != 0 {` 拒绝臂 ⇒ `param ^= 256` 且 `flags&4==0 ⇒ entity::remove`。
⇒「只有 `flags&2` 开价、`idx<=0` 早退」与实现一致；可买 ⇒ `Off("shop_present")`（fail-closed，因为买臂会改
hp/钥/金、位图推不出）；不可买 ⇒ 阻挡格（引擎撞商家格 `allowed=false`）。

### G4.3 未建模边界：披露属实（且方向是安全的）

文档 §8 原文（`docs/findings/2026-11-19-exact-pickup-dp.md`）：

> **未建模边界（如实披露）**：`flags&2==0 ∧ flags&1!=0` 的告示格，关面板后 `param ^= 256` 且实体被移除
> ⇒ 事后可通行；DP 没有「告示已读」维，仍按阻挡处理（只会低估可达性，不会高估收益）。

与 `scene.rs:2629-2634` 相符。方向也确实是「只低估」：把可通行格当墙，只会漏解，不会凭空造出收益。

### G4.4 但「根因是 shops 表被丢弃」这条归因是**错的**（可证伪）

文档 §6 写：「新增的 8 个 `shop_present`……⇒ **旧的 8/54 高估了 4 层**，根因是 `shops` 表被丢弃（见 §8）」；
commit message 也写「同链态改传真表即 `adopted=false reason=shop_present`——即『打穿 f3』只在空表口径成立」。

**在修好后的代码上，这个因果不成立**：精确 DP 的商店判定已经**只读 `param` 机制位，不读商品表**。
我直接把同一链态分别喂空表与真表：

```bash
# f3（ty77@(11,4) param=0x116 ⇒ flags=1 不可买）
EMPTY_SHOPS=1 cargo run --release -q -p game-solver --example f3_route_probe   # &[]     ⇒ adopted 88/14/4741000/4027
                cargo run --release -q -p game-solver --example f3_route_probe   # si.shops ⇒ adopted 88/14/4741000/4027
# f2（ty78@(11,7) param=0x201 ⇒ flags=2 可买）
EMPTY_SHOPS=1 cargo run --release -q -p game-solver --example f2_shop_probe    # &[]     ⇒ reason="shop_present" reason_ty=78
                cargo run --release -q -p game-solver --example f2_shop_probe    # si.shops ⇒ reason="shop_present" reason_ty=78
```

两组**完全相同**。原因在代码里写着：`run()` 注释「层视图不再吃商品表：商店格的判定读实体 `param`
的机制位（`shop_can_buy`），表只喂 `hp_bound::plan` 与 `Dp` 的 `Search` 面」，
`solve_local.rs:4235-4245` 的 `Cell::Shop(slot, ty)` 臂也只 `st.ents.get(slot).param`。

**真正的根因**是判据从「表非空 ⇒ 拒」的 **ty 盲判**换成了 `shop_can_buy(param)`：
旧实现（`git show 76734f6:crates/game-solver/src/solve_local.rs` 的 `:4168`）是

```rust
Cell::Shop(_, ty) => { if !shops.is_empty() { return Err(("shop_present", ty)); } ... }
```

⇒ 旧版「8/54 覆盖」之所以高估，是**测量时传了空表 + 判据本身 ty 盲**这两件事叠加；
仅把表接上（不改判据）会让 f2/f3 **一起**变 Off，f3 反而解不了。

**要说清的一点**：这不影响本次整改的实质结论（旧 8/54 确实高估；真实表下 adopted=4/54；
f3 该放行、f2 该拒），只是**归因句与机制不符**。按 AGENTS §3.6「证伪更正」纪律应补一行：

> 【追加更正】「根因是 shops 表被丢弃」应改为「根因是旧判据 `表非空 ⇒ 拒` 的 ty 盲判 + 测量用空表」；
> 新判据下商店格的 `Off/放行` **与商品表无关**（实测空表与 17 行表同结果），表只影响
> `hp_bound::plan`、`Search` 买臂与 memo 键。

---

## G5 黄 F6 的判据真能抓「静默截断」吗

**判定：绿。**

判据（`crates/game-solver/src/exact_pickup.rs:912` / `:935`）用 `#[cfg(test)] thread_local`
把预算压到 1（生产 `:176`/`:195` 恒 `MAX_STATES`/`MAX_GATE_CALLS`），断言「撞界 ⇒ `Off(具名 reason)` 且
不携带半成品统计」。我自己做两次变异：

```bash
# M5：state_cap 改静默
-        if self.cap_reached() { return Err(("state_cap", self.states.len() as i32)); }
+        if self.cap_reached() { return Ok(None); }
# M6：gate_budget 改静默
-        if gate_calls > gate_budget() { return give_up(st, "gate_budget", gate_calls as i32); }
+        if gate_calls > gate_budget() { break; }
cargo test --release -p game-solver --lib -- exact_dp_state_cap exact_dp_gate_budget
```

```
exact_dp_state_cap_is_off_not_truncation ... FAILED  必须是具名拒绝，实测="exit_unreachable"
exact_dp_gate_budget_is_off_not_truncation ... FAILED 必须具名，实测="exit_unreachable"
```

⇒ 两条判据都能抓静默（静默后理由退化成 `exit_unreachable`，与「预算」不可混淆）。

---

## G6 链路退回 floors=7 是否属实、归因是否正确

**判定：黄**（现象与方向绿；「3 把」这个数字错，应为 **2 把**）。

### G6.1 现象独立复现（仓库 HEAD 与 `/tmp/y` 两份各跑一次，输出逐字相同）

```bash
cargo run --release -q -p game-solver --bin solver -- run --out /tmp/y_run.json
```

```
status=blocked floors=7 total_steps=363 blocked=floor 4 reason=no_exit_toward layer_ok=Some(true) layer_truncated=Some(false)
```

JSON 的 `blocked.detail`：

```
层 4 的目标集里没有「出层边指向 f5」的解；规范回落解 kind=Stair exit=Some((11, 11)) edge=Some((3, true))
```

成功前缀：f51(69) / f52(102) / f53(25) / f54(63) / f1(6) / f2(10) / f3(88)。⇒ `floors=8 → 7`、卡在 **f4**、
理由 **no_exit_toward** 全部属实（不是 `exit_unreachable`——f4 层本身可解，只是解不指向 f5）。

### G6.2 归因：方向成立

f3 出门黄钥匙余量 0（G3.3）；f4 在 0/1 把时无解、≥2 把有解 ⇒ 链路断在 f4 是**正确的下游后果**。
旧链路能过 f4 靠的是 106/18 那条**非最优**路线多剩的 3 把黄钥——即「旧 8 层的可解性建立在 bug 上」。

### G6.3 但「f4 需要 3 把黄钥匙」是错的，最小是 **2 把**

三条独立路径给出同一个数：

1. **DP**（`/tmp/y` 的 `f4_probe`，同链态、只改 `yellow`）：

```
yellow=0 → ok=true trunc=false hit=false out_hp=4027 exit=Some((11,11)) route_len=1   （回落解：原地上下梯）
yellow=1 → hit=false route_len=1
yellow=2 → hit=true  exit=(1,11) route_len=92 adopted steps=88 picks=7 doors=6 kills=7 out_hp=4327
yellow=3 → hit=true  exit=(1,11) route_len=81 steps=77 picks=5 doors=5 out_hp=4327
```

2. **独立 BFS**（我自己写的 python，只用 f4 地形 + 门/钥匙模型，不调 solver 任何代码）：
`initial yellow=0 或 1 ⇒ 上梯 (1,11) 不可达`；`yellow=2 ⇒ 可达`（`blue=0` 时同样可达 ⇒ 蓝钥非必要）。

3. **手算机制**（与 1、2 一致）：f4 的 entry `(11,10)` 落在右下孤岛 `x=11` 列（y9/y10 的 `x=10`、y11 的 `x=6`
   都是墙），唯一出路是 `y8` 的竖排黄门 `(11,8)` ⇒ **第 1 把**；进中层后通往上层的 `y4` 只走 `x=2,6,10`，
   其中 `(2,4)`/`(10,4)` 又是黄门 ⇒ **第 2 把**；上层捡到 `(3,2)/(9,2)` 两把后，再开 `(1,8)` 才能进
   `x=1` 孤岛到上梯 `(1,11)`。⇒ 入场最少 **2** 把。（层内确实有 5 把黄钥，但全在门后，0 把时够不着。）

⇒ **我改判**：归因（f3 余量 0 ⇒ f4 无解 ⇒ `no_exit_toward`）**成立**；但数量应写 **2 把**，不是 3 把。
结论不变（0 < 2）。文档正文只写「少 3 把黄钥匙」（这是 f3 **旧路线余量**，正确），
**没有**写「f4 需要 3 把」——所以这是自述口径问题，不是文档错误；建议后续叙述统一成「f4 上行最少要 2 把」。

### G6.4 附带核对：f2 的「9 vs 10 步」确属记录口径

`crates/game-solver/src/solve_local.rs:3609 route_of` 里 `Act::Start => format!("{},{}", pos.0, pos.1)`
⇒ Search 系层的 `route[0]` 是**入场格 token**；精确 DP 的 `route_to` 不含（实测 f3 的 88 token 从 `3,11`
开始，而 entry 是 `(2,11)`）。⇒ 「f2 steps 9→10」是口径差，不是物理路径变化，文档这句属实。

---

## G7 夹带与诚实

**判定：绿。**

### G7.1 三笔范围（零越界）

```bash
for c in 127a9cb fc22c44 fa30c94; do git show --name-status --format="%h %s" $c; done
```

```
127a9cb fix(solver): 精确 DP 只保留正向支配（F4 红）+ witness 透传商品表并按机制判商店（F3/F8 黄）
M  crates/game-solver/{examples/exact_coverage.rs, examples/profile_floor.rs, src/bin/solver.rs,
   src/exact_pickup.rs, src/solve_local.rs, src/witness.rs, tests/exact_pickup.rs,
   tests/pure_pickup.rs, tests/witness.rs}
fc22c44 docs(findings): P9 整改就地更正（F4/F10）…
M  docs/findings/2026-11-19-exact-pickup-dp.md, docs/findings/2026-11-19-hp-bound-p8b-spawn-gate.md
fa30c94 chore(gates): 测试计数基线 1231 → 1237（P9 整改新增 6 条判据）
M  data/status/baselines/rust-test-counts.json
```

一个逻辑变更加文档更正加基线更新，三者分开提交（AGENTS §9「小步提交」+「门禁改动单独成 commit」满足）。
`git diff 76734f6..fa30c94 | grep '^+.*#\[ignore'` → **无新增**。

### G7.2 docs 更正保留原主张（§3.6）

```bash
git show fc22c44 -- docs/ | grep -E "^-" | grep -v "^---"     # 输出为空
```

⇒ **零删除行**，全部是 `> 【2026-11-20 更正】…` 原地追加（§1 事实口径、§3 状态数、§4 支配引理、§6 覆盖口径、§8 整改记录），
读者仍能看到「曾经错在哪」。

### G7.3 无净测试删除（顺带抓「用删测试换绿灯」）

```bash
git diff 76734f6..HEAD -- crates/ | grep -E "^[-+]\s*(fn |#\[test\]|#\[ignore\])"
```

新增 7 个 `#[test]`、删除 1 个 —— 被删的 `exact_dp_corrupted_prediction_is_off` **只是搬家**：
现在在 `crates/game-solver/src/exact_pickup.rs:887`（lib 单测），仍在跑。净新增 6 条判据，
与 ratchet `1231 → 1237` 吻合。

### G7.4 ratchet 实测（我自己复刻 `collect_test_counts`）

用 `cargo test --workspace --release --no-run --message-format=json` 取每个 test 可执行文件跑
`--list --format terse` 数 `: test`：

```
total= 1237 baseline_total= 1237 binaries= 81 baseline_binaries= 81
DIFF: {}
exact_pickup(test)=11  game_solver(lib)=64  pure_pickup(test)=16  witness(test)=9
```

### G7.5 SKIP 撤回（静态核对，顺手省掉 11 分钟）

`gates/cli.py`：`cmd_check` 的列表有 **13** 门（…`state_machine_anchors`、`state_baseline` 收尾），
`cmd_status` 的列表有 **14** 门（多 `reference`/`trace`/`visual`，不含那两门）⇒
「`check` 是 13 门 SKIP=0、`status` 是 14 门 SKIP=1（visual 未就绪）」**成立**，实现者撤回「SKIP=0 写错」是对的。

我另外把总闸真跑了一遍（不是引用自述）：

```
[✓ PASS] original-integrity / assets-manifest / reference-seed-integrity / knowledge-format / docs-links
[✓ PASS] naming-ledger / rust-tests / test-ratchet（total=1237 基线1237）/ table-content-hash /
         table-evidence / state-machine-anchors / asset-truth-manifest / state-baseline（74 场景 sha 全同）
== 汇总 ==
PASS=13 FAIL=0 SKIP=0
```

---

## Confidence

**对「红/黄已真修」：90。**

- 红 F4：**95**。反向段确已删除（零残留 + 全域无 `dead=true`/`retain`），f3 数值与复核探针逐字一致，
  且复原旧段能让两条判据同时红到旧值（106/18/130,835）⇒ 修复是真的、判据是真的。
- 黄 F3/F8/F6/F10：**85–90**。F6 两条护栏判据变异各自红；透传链与机制判据代码级可指认；SKIP 撤回正确。
  扣分点是 G4 的归因句（机制没缺陷但解释错了）与 G6 的「3 把」（应为 2 把）——都是口径，不是行为。
- 全链可行性：**未修**（已披露）。`(hp, steps)` 层内最优与跨层钥匙盈余不耦合，这层意思文档写了遗留项，
  但**没有任何判据**钉住「跨层钥匙守恒」——这是本次复核唯一想推荐的新护栏（见末句）。

---

## 两句话

**1）这三笔能不能合并：能合并。** 红 F4 已真修（88/14/4,741,000 复现、复原旧段可回红 106/18），
F3/F6 的修法在代码级站得住，三笔范围零越界、docs 零删除行、无 `#[ignore]`、ratchet 1237/81 与
`check` PASS=13 FAIL=0 SKIP=0 都实测一致；两处黄只是文档口径（把「根因是 shops 表被丢弃」改成
「旧判据 ty 盲 + 测量用空表」、把「f4 需要 3 把」改成「2 把」），建议在下一笔 docs commit 里就地补，
**不必阻塞合并**。

**2）链路退回 floors=7 该不该接受：接受，但只当作「正确基线」而不是「修好了」。**
它是修复的**必然下游**——旧 8 层依赖的是 106/18 那条非最优路线多剩的 3 把黄钥（bug 的产物），
修对之后 f3 余量 0 而 f4 上行最少要 2 把、必然断；但它同时暴露了**目标函数的结构缺陷**：
逐层 `(hp 最大, steps 最小)` 与跨层钥匙盈余不耦合，所以以后每修一次「层内更优」都可能这样掉一层。
建议下一片加一条跨层不变量（例如「进入 fN 时黄钥匙 ≥ 该层上行最小需求」的守恒/警告），
把这类退层从「事后才发现」变成「层内就算得出来」。
