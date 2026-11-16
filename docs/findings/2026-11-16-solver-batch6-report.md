# 批6 报告：求解器击杀金币边（C6a）+ 金币判据（C6b）+ 三查询重跑（C6c）

> 状态：**已实现、已自证、已重跑、未推送**（分支 `modern`，父提交 `911fa94`）。
> 口径提醒：本文件所有三查询数值均为**求解器模型口径，待 oracle 回放**（与
> `docs/findings/2026-11-15-solver-queries.md` 全文一致）。
> 触发来源：批5 对抗复核 §2.4 / §11 建议 1（`docs/findings/reviews/2026-11-16-solver-batch5-review.md`），
> 以及 `2026-11-15-solver-queries.md` §4.5 二次更正 + §5 建议 0（H5）。

## 1. 改了什么（file:line）

### 1.1 C6a 击杀金币边（模型侧，唯一功能改动）

| 位置 | 内容 |
|---|---|
| `crates/game-solver/src/solve_local.rs:961` | 新增 `fn kill_bounty(level: &LevelSpec, player: &PlayerCombat, ety: i32) -> i32`：`let base = level.enemy_base.get((ety - 41) as usize).map(\|e\| e.gold).unwrap_or(0); if player.find_item(25) >= 0 { base << 1 } else { base }` |
| `crates/game-solver/src/solve_local.rs:1536` | 闭包臂 `ClosureCand::Kill`：`ns.player.gold += kill_bounty(self.level, &pc, ety);` |
| `crates/game-solver/src/solve_local.rs:1635` | 走格臂 `Cell::Monster`：`ns.player.gold += kill_bounty(self.level, &player, ety);` |

**与引擎同源的三点**（逐条给出证据，均为 A 级）：

1. **加金语义**：`crates/game-core/src/scene.rs:2301-2309`——
   `let gold = if self.player.find_item(25) >= 0 { self.enemy_base[(t - 41) as usize].gold << 1 } else { self.enemy_base[(t - 41) as usize].gold }; self.player.gold += gold;`
   （`t` = 实体 `type_id`，`41..=75`）。模型 `kill_bounty` 逐项对应，`<< 1` 即双倍。
2. **item25 判定**：`crates/game-core/src/combat.rs:67-74` `find_item` ⇒ 模型直接调用 `PlayerCombat::find_item(25)`（`PlayerSpec::to_combat()` 得到，零额外克隆），**不另写判定**。
3. **型号→表索引**：`crates/game-core/src/combat.rs:24-35`（`idx = t - 41` 稠密下标）⇒ 模型 `level.enemy_base[(ety - 41)]`；`enemy_base` 由 `crates/game-solver/src/bin/solver.rs:66-79` 用 `crates/game-core/tests/fixtures/enemybasegold.txt` + `tables.to_records(&gold)` + `scale_enemy_stats(&base, 1)` 注入（**未改 `data/tables/**`**）。

### 1.2 C6c 诊断指标（可复跑证据，无行为影响）

| 位置 | 内容 |
|---|---|
| `solve_local.rs:465,470` | thread_local `GOLD_PEAK: Cell<i32>`、`SHOP_BUYS: Cell<(u64,u64)>`（`(买臂总数, 授红数)`） |
| `solve_local.rs:372-373` | `solve_layer_with` 入口清零 |
| `solve_local.rs:396-397` | `LAYER_MEMO` 命中时回放（memo 元组 3 元 → 5 元：`(bool, Vec<LayerGoal>, Option<(i32,usize)>, i32, (u64,u64))`） |
| `solve_local.rs:1553-1570` | `expand` 拆 `expand`（采样入口态 + 全部生成态的 `player.gold`）+ `expand_inner` |
| `solve_local.rs:1613-1621` | 买臂 `apply_shop_effect` 成功后计数，`AddKeys{red,..} && red>0` 记授红 |
| `solve_local.rs:492-500` | `pub(crate) fn take_gold_peak() -> i32` / `take_shop_buys() -> (u64,u64)` |
| `solve_global.rs:86-97` | `GlobalResult` 新增 `gold_peak`、`gold_peak_by_floor: Vec<(i32,i32)>`、`shop_buys`、`red_buys` |
| `solve_global.rs:367-374,424-432` | 逐层聚合（`BTreeMap<i32,i32>` 取每层最大；买臂累加） |
| `solve_global.rs` `to_json` | json 增 `"gold_peak"` / `"gold_peak_by_floor"` / `"shop_buys"` / `"red_buys"` |

### 1.3 C6b 金币判据（`crates/game-solver/tests/solve_global.rs`）

| 位置 | 内容 |
|---|---|
| `tests/solve_global.rs` `load_enemies` / `enemies_real_bounty` | 读**真表** `data/tables/enemies.json`（`parse_enemies` 返回 `(Vec<EnemyDef>, Vec<i64>)`，第二元素是 difficulty multiplier，不是未知行） |
| `tests/solve_global.rs` `gold_world(monsters, shop_idx: Option<u8>)` / `gold_run(monsters, shop_idx, with_item25, start_gold)` | 8×3 走廊，仅 `y=1` 可走：`(0,1)` 入场 → 怪槽 → `x=4..=6` ty78 商店 `param=(3<<8)|idx` → `(7,1)` ty7 `param=50` 上行梯 → `floor50_nodoor` 终局 |
| `tests/solve_global.rs:1557` | `solve_global_t击杀给金_普通与item25双倍_余额精确断言` |
| `tests/solve_global.rs:1614` | `solve_global_t连续买两次_余额与买不起分支_逐值断言` |
| `tests/solve_global.rs:1683` | `solve_global_t红钥匙买臂计数_正控`（诊断字段的正控：id10 ¤800 红+1 真表行） |

**旧入口行为变化范围**（必填项）：`solve_global_with(..., shops: &[])`（旧入口 / `solve-global` 子命令）
在真实 55 层数据上**行为会变**——凡击杀怪物的路径，`player.gold` 不再恒 0，`gold` 又是全局去重键 `GKey`
（`solve_global.rs:118-145`，gold 进键）的组成部分，故节点数、`exp`、`goals` 都可能变。
**理由**：这是有意的模型修正——gold 恒 0 时所有 `price>0` 商店边在语义上不可达，模型对「买得起」为**过近似（漏解）**；
C6a 让模型与引擎 `scene.rs:2301-2309` 一致。既有判据因此变红的逐条处理见 §2.3。

## 2. 红 → 绿证据

### 2.1 先红（实现 `kill_bounty` 之前，仅测试落地）

```
$ cargo test --release -p game-solver --test solve_global 击杀给金
---- solve_global_t击杀给金_普通与item25双倍_余额精确断言 stdout ----
thread 'solve_global_t击杀给金_普通与item25双倍_余额精确断言' panicked at crates/game-solver/tests/solve_global.rs:
assertion `left == right` failed: ① 普通击杀余额须精确 = 赏金 100（引擎 scene.rs:2305-2309 非双倍臂）
  left: 0
 right: 100

$ cargo test --release -p game-solver --test solve_global 连续买两次
---- solve_global_t连续买两次_余额与买不起分支_逐值断言 stdout ----
thread 'solve_global_t连续买两次_余额与买不起分支_逐值断言' panicked at crates/game-solver/tests/solve_global.rs:
assertion `left == right` failed: ① 余额集合须 == 连买链 {122,72,22}（入场金 0 ⇒ 全部来自击杀）
  left: {0}
 right: {22, 72, 122}
```

读法：`left` 是模型给出的值（击杀 0 金 ⇒ 余额 0；买不起任何东西 ⇒ 只有 `gold=0` 一个 goal），
`right` 是引擎语义期望值（真表 id47 赏金 100；100+22 杀金后买两次 ¤50 ⇒ `{122,72,22}`）。

### 2.2 后绿（实现后）

```
test solve_global_t击杀给金_普通与item25双倍_余额精确断言 ... ok
test solve_global_t连续买两次_余额与买不起分支_逐值断言 ... ok
test solve_global_t红钥匙买臂计数_正控 ... ok
```

全量：`cargo test --release -p game-solver` ⇒ `7 / 0 / 4 / 24 / 13` passed（另 1 ignored），0 failed。

### 2.3 因 C6a 变红的既有判据（逐条）

实现 + 测试全量跑过 3 轮（含 workspace），**0 条既有判据变红**——原因：既有夹具
`enemies()`/`player()` 的 `gold=0` 且多数世界不带商店，`gold += 赏金` 在「金不进结果」的世界里
不改变 `goals`/`route` 断言面；而 `GKey` 含 gold 只会**增加**节点（不会合并出错），既有断言均为
「存在性/计数值」而非「节点数精确等于」，故未触发期望值改写。
唯一需要改的既有夹具签名是本次**自建**的 `gold_world/gold_run`（`with_shops: bool` → `shop_idx: Option<u8>`、
新增 `start_gold`），属批6 内新增代码，非历史判据。

## 3. 变异自证（3 发，全红；已还原）

| # | 变异 | 打到哪个判据 | 结果 |
|---|---|---|---|
| M1 | 删走格臂 `ns.player.gold += kill_bounty(...)`（`solve_local.rs:1635`） | 击杀给金 ① | **红**：`① 普通击杀余额须精确 = 赏金 100`，`left: 0 / right: 100` |
| M2 | `kill_bounty` 里 `base << 1` 改 `base`（去掉 item25 双倍） | 击杀给金 ②（item25 臂） | **红**：`② item25 双倍余额须精确 = 赏金<<1`，`left: 100 / right: 200` |
| M3 | 删买臂 `ns.player.gold -= price;`（复现批5 复核 §11 的 M1 存活变异） | 连续买两次 ① | **红**：`① 余额集合须 == 连买链 {122,72,22}`，`left: {122} / right: {22,72,122}` |

M3 的意义：批5 复核的 M1（扣金变异仍全绿）在批6 夹具下**已被杀死**——余额断言不再允许「扣金整行消失」。

还原后净空证明：

```
$ git diff --quiet   # 变异已全部还原
$ git status --short crates/game-solver   # 仅本批预期改动
$ cargo test --release -p game-solver    # 全绿（见 §4）
```

## 4. 门禁与测试数字

| 项 | 值 |
|---|---|
| `python3 gates/cli.py check` | **PASS=13 FAIL=0 SKIP=0**（每笔 commit 的 pre-commit 钩子实跑） |
| `cargo test --release -p game-solver` | 7 + 0 + 4 + **24** + 13 passed，0 failed，1 ignored |
| `game_test_ratchet` | 只对「减少」报 FAIL；新增 3 条测试只推高 total：**基线 1075 → total=1084**（`dc96def` 的 pre-commit 实跑输出）⇒ **无需改 ratchet 基线，未动 `data/status/baselines/`** |
| 三查询耗时 | 3m28s ~ 3m35s（`query --all`，release） |

## 5. 三查询：批5 → 批6 对照（含金币新指标）

| 查询 | 指标 | exp（批5→批6） | goals | truncated_layers | node_cap_hit | 金币峰值 | f12 层峰值 | 买臂数 | 授出红钥匙 |
|---|---|---|---|---|---|---|---|---|---|
| Q1 剩余血量最大 | `max_hp` | 68550 → **68148** | 0 → 0 | `[]` | true | **210 → 1861** | **767** | 583 | **0** |
| Q2 最少杀怪 | `min_kills` | 7938 → **8687** | 0 → 0 | `[[6,200000],[7,200000]]` | true | — → **1021** | 未到达 | 5133 | 0 |
| Q3 不拿终极装备 | `reachable` | 641 → **641** | 0 → 0 | `[]` | false | — → **23** | 未到达 | 0 | 0 |

（批5 的「210」来自复核期临时打点，当时的 json 尚无 `gold_peak` 字段；批6 起该指标成为落盘字段，可复跑。）

逐层金币峰值（Q1，`gold_peak_by_floor`）——新增的「金币轨迹」证据：

```
f1 226 → f5 280 → f10 538 → f11 718 → f12 767 → f14 925 → f16 1014 → f17 1149 → f18/f19/f20 1861
f51 5  → f52 1021 → f53 222 → f54 222        # 起点 f51（solver.rs start_floor:51）的另一支
```

### 5.1 关键结论（金币是否成为新瓶颈）

1. **金币不再是稀缺资源**：峰值 210 → 1861（×8.9），逐层峰值总体随深度上升（局部回落源自不同楼梯入口的富集路径不同，非单调）⇒ C6a 的边确实在跑，不是死代码。
2. **红钥匙距离 = 33 金**：真表 id10（`data/tables/shop_goods.json`，`src=1/idx=6/price=800/AddKeys red+1`）
   在 f12；Q1 在 **f12 层内**的金币峰值只有 **767 < 800** ⇒ `red_buys=0` 是「到 f12 时还差 33 金」，
   **不是**搜索取舍——该边在 f12 层内**一次都不可买**（「可买而未买」次数 = 0）。
   相对批5 的 210，「红钥匙距离」从 **590 金压到 33 金**。
3. **但「可买」尚不完整**：Q1 `node_cap_hit=true`（arena 满于 `exp=68148`）⇒ 全局未穷尽；
   33 金是**已探索部分**的缺口上界，不是理论最小缺口。
4. **Q3 的 UNSAT 与金币无关**：`exp=641`、`cap_hit=false`、`trunc=[]` ⇒ 队列自然耗尽（模型内穷尽），
   可达集仅 f51–f53、金币峰值 23（< 最低商品价 50）、买臂 0 次 ⇒ 瓶颈是**结构可达性**（`ban=[36,40]` 下出不去）。
5. **Q2 不可测红钥匙距离**：`trunc=[[6,200000],[7,200000]]` 卡在 10 层以内，**根本没到 f12**（无 f12 层峰值条目）。

⇒ 诚实结论：`goals` 仍 0，但**不是「补边无效」**。金币已从「绝对饿死」变成「33 金之差」，
当前瓶颈从**缺边**转向**搜索覆盖 / `ents` 扇出**（Q1 arena 满、Q2 层截断）。

## 6. 遗留假设与风险

1. **层内去重键不含 `gold`（本批新暴露的近似）**：`solve_local.rs:812-826` 的 `Key` 只有
   `pos/ents/flags/keys/bag/atk/def`；`weakly_ge`（`:831-833`）只比 `hp/kills/steps`。
   补金前 gold 恒 0 ⇒ 完全无害；补金后同层「同键不同金」先到者胜 ⇒ **可能吞掉更富的同键态**，
   即 §5 的 33 金缺口**可能是低估**（真实缺口更小甚至已可买）。全局层 `GKey` 已含 gold
   （`solve_global.rs:121-145` 的 `GKey`，其中 `gold: i32`），且支配只在**同键桶内**比较（`push_state` `by_key.entry(k)`），
   所以全局层没有「高金态被低金态剪掉」的问题——**风险仅限层内**。
2. **Q1 全局未穷尽**（`node_cap_hit`）⇒ 「红钥匙买不到」在未探索区间内**未被证明**。
3. **item25 双倍只按 `find_item(25)>=0` 判**：引擎同源；但模型 `bag` 是 canonical 截断表示
   （`CanonicalBag`），若某态 bag 截断丢掉了 item25 而 hp/atk 恰好等价，判定可能与引擎偏离（未观测到，属假设）。
4. **赏金表只经 `scale_enemy_stats(&base, 1)`**：难度倍率取 1（与批5 口径一致）；若关卡实际有倍率，
   赏金会被低估——本批未核（`enemybasegold.txt` fixture 口径）。
5. **`shop_buys` 是「买臂触发次数」不是「不同商品购买数」**：同一条边在多个入口态下会重复计，
   数值只用于「有没有触发」的方向判断，不可当经济统计。
6. **未处理（按 lead 裁定留待后续）**：商店「一次性移除 vs `flags&1`/`&4` 可重复」分派、商人格走臂改 `Blocked`。
   本批实施中**未发现**推进 C6a–C6c 必须动它们的情况。

## 7. commit 清单（未推送）

| # | hash | 标题 | 文件 |
|---|---|---|---|
| 1 | `d5331af` | `feat(solver): 批6 C6a 击杀给金边（含 item25 双倍）+ gold_peak 诊断` | `solve_local.rs`、`solve_global.rs` |
| 2 | `6f98f8b` | `test(solver): 批6 C6a/C6b 金币判据（击杀给金×item25 双倍、连买两次余额）` | `tests/solve_global.rs` |
| 3 | 见 `git log` | `feat(solver): 批6 C6c 诊断指标（逐层金币峰值/买臂/授红）+ 正控测试` | `solve_local.rs`、`solve_global.rs`、`tests/solve_global.rs` |
| 4 | 见 `git log` | `docs(solver): 批6 三查询重跑落盘 + §4.6 复跑结论 + 批6 报告` | `docs/findings/solver-queries.json`、`2026-11-15-solver-queries.md`、`2026-11-16-solver-batch6-report.md` |

每笔均含 `Evidence:`（引擎行号/真表行/复跑命令）与 `Confidence:`；`gates check` 由 pre-commit 钩子逐笔执行。
`data/status/baselines/`（ratchet）**未改**（本批只增测试）。
