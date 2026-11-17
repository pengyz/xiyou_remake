# 批6 对抗复核：击杀金币边 + 金币判据 + 三查询重跑

> 复核范围：`git log --oneline ac179f9..HEAD` = 9 笔（含批5 的 `f8cfd49`/`eec959a`/`911fa94`，
> 批6 的 `d5331af`/`6f98f8b`/`dc96def`/`a3099d0`/`16db0f5`，以及夹在中间的 `a5431dd`=ROI 设计定稿）。
> 被复核对象：`docs/findings/2026-11-16-solver-batch6-report.md`（**不默认其为真**，逐条自证）。
> 复核方式：只读 + 自跑测试/门禁/三查询 + 自建临时探针（评审后删除）；**未改任何交付代码**。
> 时间：2026-11-16。判据来源：任务书 F1–F10 + 追加 F11（架构/纪律面）。

---

## 1. 结论

| 项 | 值 |
|---|---|
| **可否推送** | **不可**（红 = 1；推送门槛是红 = 0） |
| **红项** | F2：层内去重键/支配不含 `gold` ⇒ **健全性缺陷**（可把有解判成无解），本批把它从「休眠近似」激活成「活的漏洞」 |
| **Confidence** | **88 / 100** |
| 说明 | 代码面（F1/F4/F5/F6/F7/F8/F10）实测全绿，报告绝大多数数字可逐字复现；**唯一阻断项是 F2**，它同时污染本批的头条结论（「差 33 金」「Q3 穷尽无解」）的**逻辑地位**（不是数值本身） |

一句话：**批6 干的事基本是对的、可复现的；但它补进去的 `gold` 让一个批0就存在、此前无害的剪枝近似变成了会漏解的真漏洞，而本批报告把它写成「可能低估」——量级判错了。**

---

## 2. 面子表

| 面 | 判定 | 一句话要点 | 证据（命令 / file:line） |
|---|---|---|---|
| **F1** 击杀给金语义同源 | **绿**（2 黄注） | 赏金公式与引擎两处内联逐字一致（`find_item(25)>=0 ? gold<<1 : gold`）；无双计 | 引擎 `scene.rs:2300-2309`（钹路径）、`subsystem/battle.rs:187-193` + `scene.rs:3778-3785`（撞怪路径）；模型 `solve_local.rs:961`、`:1536`、`:1635`；边界见 §4.1 黄1/黄2 |
| **F2** 层内去重/支配健全性 | **红** | `Key` 无 `gold` + `weakly_ge` 只比 hp/kills/steps ⇒ 同键「更富但入桶晚」的态被删 ⇒ **假 UNSAT**（同一张图只换入口格就翻转） | `solve_local.rs:805-827`（Key）、`:830-845`（weakly_ge/dominates）、`:1875-1893`（同键桶剪枝）；探针见 §3 |
| **F3** 诊断指标可信度 | **绿**（2 黄注） | 线程局部三量在每层入口清零、memo 命中按缓存回放、`expand` 覆盖全部生成路径；`red_buys=0` 由「唯一红商品 ¤800」+「峰值 < 800」结构性推出 | `solve_local.rs:465-476`（定义）、`:370-372`（清零）、`:380-397`（回放）、`:1553-1570`（采样）；`shop_goods.json` id10 唯一红行 ¤800 |
| **F4** 变异自证真伪 | **绿** | M1/M2/M3 我自改自跑，**三发真红**且断言文本与报告逐字一致；还原后 `git diff --quiet` CLEAN | §5 三发实测输出 |
| **F5** 判据强度 | **绿**（1 保留） | 三条钉子都有存在性闸 + 逐值断言 + 真表（enemies.json 35 行/4839、shop_goods id8 ¤50/id10 ¤800） | `tests/solve_global.rs:1557`、`:1614`、`:1683` |
| **F6** 三查询可复现 | **绿** | 我独立跑两遍，**两遍与落盘 json 三者完全相同**（含全新诊断字段）；`truncated_layers`/`cap_hit` 代码零改动 | `cargo run --release -p game-solver --bin solver -- query --all --out …`（3m57s / 5m54s）；`python3 -c` 比对 = True |
| **F7** 有无静默调参 | **绿** | 无 `bound`/`node_cap`/排序/剪枝收紧；唯一的剪枝改动（`landmark_dead` 计商店）是**放宽** | `git diff ac179f9..HEAD -- crates/game-solver/src` 逐 hunk 分类见 §6 |
| **F8** 门禁与基线 | **绿** | `check` = PASS 13 / FAIL 0 / SKIP 0；ratchet `total=1084(基线1075) 无回退`；`data/status/baselines/` **0 文件改动** | `python3 gates/cli.py check`（§7 原样输出） |
| **F9** 文档诚实性 | **黄** | 原地更正做对了（历史保留 + 日期注），但三处措辞与实现不符，且把 F2 的**健全性缺陷**降级为「低估」 | §8 |
| **F10** 红线夹带 | **绿** | 11 文件全部在 `crates/game-solver/**` + `docs/**`；`original/` `reference/` `crates/game-core/` `data/tables/` `gates/` 零改动 | `git diff ac179f9..HEAD --stat`；`git diff --name-only … \| grep -c "data/status/baselines"` = 0 |
| **F11** 架构/纪律（追加） | **黄** | 未违反「禁止求解器重写战斗算术」（伤害仍直调 `predict_battle_hp_loss`/`bump_outcome`），但赏金公式现在是**三处副本**（引擎×2 + 求解器） | `combat.rs:80-91`、`gain_gold` `combat.rs:191` 只做 `gold += amount`，无法覆盖赏金计算 |

---

## 3. 红项详述：F2 层内去重键/支配不含 `gold`

### 3.1 机制（A 级，代码路径）

1. 层内去重键 `Key { pos, ents, flags, keys, bag, atk, def }`——**不含 hp/kills/steps/gold**（`solve_local.rs:805-827`）。
2. 同键桶内的支配只看三项：

```rust
// crates/game-solver/src/solve_local.rs:831-845（原文）
fn weakly_ge(a: &St, b: &St) -> bool {
    a.hp >= b.hp && a.kills <= b.kills && a.steps <= b.steps
}
```

3. `run_root` 对新生成态 `ns` 遍历同键桶：`weakly_ge(e, &ns)` ⇒ **丢弃 `ns`**；`dominates(&ns, e)` ⇒ 淘汰 `e`（`solve_local.rs:1875-1893`）。
4. 补金前 `gold` 在层内恒 0（模型没有金币来源），这条近似**无害**；C6a 之后 `gold` 是真实状态量，而**商店价是硬门**（`gold >= price` 才出买臂，`solve_local.rs:1594-1620`）⇒ 「金多但入桶晚」的态被吞掉时，**买钥匙这条唯一通路被切断** ⇒ 可解判成不可解。

### 3.2 最小反例（我自建，实测，A 级）

同一张 4×3 图、同一组实体、同一玩家，**只换入口格**（决定 BFS 同深度的入队序）：

```
可走格：(0,1)(0,2)(1,1)(1,2)(2,1)(3,1)(3,0)
(0,1)=幸运币 ty25   (1,1)=怪 ty41（hp7/atk0/def5，赏金 800）
(2,1)=商店 ty78 param=(3<<8)|6   (3,1)=红门 ty2   (3,0)=上行梯 ty7
玩家 hp100 atk10 def10；自定义商品 src1/idx6 price=900 AddKeys{red:1}
（怪 atk0 ⇒ 两序都不掉血 ⇒ 两态 hp 相同，唯一差异是 gold）
```

实跑（探针源码见 §3.3）：

```
$ cargo test --release -p game-solver --test zz_review_f2_probe -- --nocapture
entry(1,2) start0 with item25 -> reachable=true  goals=2 Some((700, 6, ["1,2", "bump 41@1,1", "0,1", "1,1", "2,1", "door 3,1", "3,0"]))
entry(1,2) start0             -> reachable=false goals=0 None
entry(0,2) start0             -> reachable=true  goals=1 Some((700, 5, ["0,2", "0,1", "bump 41@1,1", "2,1", "door 3,1", "3,0"]))
entry(1,2) start101           -> reachable=true  goals=2 Some((1, 4, ["1,2", "bump 41@1,1", "2,1", "door 3,1", "3,0"]))
test result: ok. 1 passed
```

读法：

| 列 | 含义 |
|---|---|
| ① `with item25` ⇒ **SAT**（gold 700） | **同一个 (1,2) 世界有解**：先持 25 再撞怪 ⇒ 1600 金 ⇒ 买红。⚠ 注意这一行里路线仍是「先 bump 后捡 25」——因为 25 是预置在 bag 里的，所以语义等价于「先捡后撞」 |
| ② `entry(1,2)` ⇒ **UNSAT** | 假 UNSAT：穷态（先撞怪，800 金，hp100，steps3）先入桶，富态（先捡25，1600 金，hp100，steps3）**同键同 hp** ⇒ `weakly_ge` 真 ⇒ 富态被删 ⇒ 只剩 800 < 900 |
| ③ `entry(0,2)` ⇒ **SAT** | 同一张图换个入口，富序先入桶 ⇒ 富态存活并淘汰穷态 ⇒ 1600-900=700 |
| ④ `start101` ⇒ **SAT** | 正控：证明商店/红门/出口接线无误，②的失败**唯一**来自那 800 赏金没被计入 |

**为什么 ② 是假 UNSAT 而不是「世界本来就无解」**：②与③是同一张图；③的路线 `["0,2","0,1","bump 41@1,1","2,1","door 3,1","3,0"]` 在②的世界里就是
`["1,2"]`（左→`0,2`）再接同一后缀——合法走法存在（先捡 25 再撞怪拿 1600）。①也独立证明同世界 SAT。⇒ **引擎语义可解，求解器报不可解**。

剔除「精确键去重」这一替代解释：`Key` 不含 `gold` ⇒ 若只做**精确键去重**，两态会并存（gold 不同不进键）⇒ 商店可买 ⇒ SAT。实测却是 UNSAT ⇒ 差异只能来自 `weakly_ge`/`dominates` 把富态**支配删除**。

### 3.3 探针源码（复核自建，**非交付物**，评审后已删除）

```rust
// crates/game-solver/tests/zz_review_f2_probe.rs（临时）
use game_core::combat::{scale_enemy_stats, CombatTables, PlayerCombat};
use game_data::defs::{EnemyDef, ShopEffect, ShopGoodDef};
use game_solver::solve_local::{
    solve_layer_with, ClosurePolicy, EntSpec, ExitKind, ExitSpec, LevelSpec, PlayerSpec, SolveInput,
};

fn nums(text: &str) -> Vec<i32> {
    text.lines().map(|l| l.trim()).filter(|s| !s.is_empty())
        .map(|s| s.parse().expect("夹具整数")).collect()
}
fn enemy_tables() -> (Vec<EnemyDef>, Vec<EnemyDef>) {
    let tables = CombatTables {
        enemy_base_hp: nums(include_str!("../../game-core/tests/fixtures/enemybasehp.txt")),
        enemy_base_atk: nums(include_str!("../../game-core/tests/fixtures/enemybaseatk.txt")),
        enemy_base_def: nums(include_str!("../../game-core/tests/fixtures/enemybasedef.txt")),
        difficulty_multipliers: nums(include_str!("../../game-core/tests/fixtures/difficulty_multipliers.txt")),
    };
    let gold = nums(include_str!("../../game-core/tests/fixtures/enemybasegold.txt"));
    let base = tables.to_records(&gold);
    let scaled = scale_enemy_stats(&base, 1);
    (base, scaled)
}
fn player(hp: i32, atk: i32, def: i32, gold: i32) -> PlayerCombat {
    PlayerCombat { hp, atk, def, yellow_keys: 0, blue_keys: 0, red_keys: 0, gold,
        item_types: Vec::new(), item_uses: Vec::new(), item_stack_size: 0 }
}
fn world(entry: (i32, i32), start_gold: i32) -> SolveInput {
    const W: i32 = 4;
    let mut terrain = vec![false; (W * 3) as usize];
    for (x, y) in [(0,1),(0,2),(1,1),(1,2),(2,1),(3,1),(3,0)] { terrain[(y * W + x) as usize] = true; }
    let entities = vec![
        EntSpec { ty: 25, x: 0, y: 1, param: 0 },
        EntSpec { ty: 41, x: 1, y: 1, param: 0 },
        EntSpec { ty: 78, x: 2, y: 1, param: (3 << 8) | 6 },
        EntSpec { ty: 2,  x: 3, y: 1, param: 0 },
        EntSpec { ty: 7,  x: 3, y: 0, param: 50 },
    ];
    let (mut base, mut scaled) = enemy_tables();
    base[0] = EnemyDef { id: 41, hp: 7, atk: 0, def: 5, gold: 800, trait_flags: 0 };
    scaled[0] = base[0].clone();
    SolveInput { floor: 1, difficulty: 0, level: Some(LevelSpec {
        cols: W, rows: 3, terrain, start: entry, entities,
        exits: vec![ExitSpec { x: 3, y: 0, kind: ExitKind::Stair, index: 1 }],
        player: PlayerSpec::from_combat(player(100, 10, 10, start_gold)),
        enemies: scaled, enemy_base: base, flags: 0,
    }) }
}
fn goods() -> Vec<ShopGoodDef> {
    vec![ShopGoodDef { id: 99, src: 1, idx: 6, price: 900,
        effect: ShopEffect::AddKeys { yellow: 0, blue: 0, red: 1 }, fail_hint: String::new() }]
}
fn run_with(entry: (i32, i32), start_gold: i32, pre_item25: bool)
    -> (bool, Vec<game_solver::solve_local::LayerGoal>)
{
    let input = world(entry, start_gold);
    let mut pc = player(100, 10, 10, start_gold);
    if pre_item25 { pc.item_types = vec![25]; pc.item_uses = vec![1]; pc.item_stack_size = 1; }
    let p = PlayerSpec::from_combat(pc);
    solve_layer_with(&input, entry, None, p, 0, ClosurePolicy::OFF, &goods())
}
#[test]
fn f2_同图只改入口_假UNSAT与SAT() {
    let (poor_ok, _) = run_with((1, 2), 0, false);   // 穷序先入桶 ⇒ 期望（错误地）UNSAT
    let (rich_ok, _) = run_with((0, 2), 0, false);   // 富序先入桶 ⇒ SAT
    let (ctrl_ok, _) = run_with((1, 2), 101, false); // 正控：起金 +101
    let (i25_ok, _)  = run_with((1, 2), 0, true);    // 正控：同世界先持 25
    assert!(rich_ok && ctrl_ok && i25_ok);
    assert!(!poor_ok, "穷序先入桶 ⇒ 富态被剪 ⇒ 假 UNSAT");
}
```

### 3.4 修复方向（我不改代码，只给方向）

- **最小修法（推荐）**：把 `gold` 加入**支配维度**而不是 Key——`weakly_ge` 改为
  `a.hp>=b.hp && a.kills<=b.kills && a.steps<=b.steps && a.gold>=b.gold`（`dominates` 同步）。
  理由：gold 是「只减不增于买、只增于杀」的**单调资源**，三个目标（max_hp/min_kills/reachable）都不奖励金，
  ⇒ 金多态可无损模拟金少态（后继仍是金多 ≥ 金少，商店门 `gold>=price` 也只在金多侧开），支配成立且**不丢剪枝力**。
- **备选**：`gold` 进 `Key`（绝对安全但状态数暴涨，需重测 Q1 的 200k 层上限与整轮耗时）。
- **必须配套**：把 §3.3 那份夹具固化成回归测试（断言「同一世界，入口不同 ⇒ 结论必须相同」），否则修完没有钉子。
- **注意**：改完必须重跑三查询并**重写 §5 的三个结论**（不是改数字，是改「33 金」「穷尽」的措辞地位）。

### 3.5 对头条数字的影响（逐项）

| 头条数字 | 判定 | 理由 |
|---|---|---|
| `gold_peak` 1861 / f12 767 / Q2 1021 | **下界仍成立**，但**不是不可达证据** | `GOLD_PEAK` 在 `expand` 采样（`solve_local.rs:1553-1570`），采的是**生成过**的态（含随后被剪的态）⇒ "至少生成过 767"是硬的；但被剪祖先会抑制后代生成 ⇒ 767 只是下界 |
| 「f12 差 **33 金**」 | **方向可能反转**（不是数值误差） | 若补上 §3.4 的支配维度，f12 可能生成 ≥800 的态 ⇒ 缺口可能 <33，甚至可买（报告自己写了「可能已可买」，但把它归为「低估」，实际是**可能推翻**） |
| Q1 `max_hp=0` / `exp=68148`、Q2 `min_kills=0` / `exp=8687` | **仅为「剪枝搜索内最优」** | `goals=0` ⇒ 空集上的最值；且搜索面本身被 gold 盲剪污染 ⇒ 不能读作「无解/下界」 |
| Q3 `cap_hit=false` 的「穷尽无解」 | **措辞必须降级** | 实测 Q3 金币峰值 23、买臂 0（`shop_goods` 最低价 50）⇒ 实务反转风险低；但「队列耗尽」只证明「**在 gold 盲剪的搜索图内**耗尽」，不构成「模型内无解」的证明 |

---

## 4. 其余各面的取证与判定

### 4.1 F1（绿，两黄注）

| 核对点 | 结论 | 证据 |
|---|---|---|
| ① `idx = ety - 41` 边界 | 模型 `.get(...).unwrap_or(0)` ⇒ 0；引擎是下标（`(t-41) as usize`）⇒ 越界即 panic/异常 | `solve_local.rs:961-970`；`scene.rs:2307`；`battle.rs:98`。差异只在 `ety<41 且 entity_category==8` 的面上出现，属**模型更防御**，不产生错值 ⇒ 黄1 |
| ② item25 判定点 | 同为击杀时刻的 `find_item(25) >= 0`，同一 `PlayerCombat` 快照 | 引擎 `scene.rs:2304`/`battle.rs:188`；模型 `solve_local.rs:966`（`&pc` 为击杀前快照，与引擎同序） |
| ③ 难度倍率是否给更多金 | **不给**：`scale_enemy_stats` 只缩放 hp/atk/def，`gold: r.gold` 原样 | `crates/game-core/src/combat.rs:80-91` ⇒ 报告 §6 风险 4「有倍率则赏金被低估」**假设不成立** ⇒ 黄2（写进 F9） |
| ④ 双计风险 | **无**。①`expand_inner` 开头闭包步 `out.push(s); return;` ⇒ 闭包 Kill 与走格 Monster 互斥；②两臂各自 `ents.remove(slot)`；③实测：单杀一次 = 800（反例②里 800 < 900 买不起），双倍下 = 1600（③里 1600-900=700 精确） | `solve_local.rs:1558-1562`、`:1536`、`:1635`、`:1623-1650`；§3.2 实跑 |

> 补充：引擎的「撞怪给金」是在 `scene.rs:3778-3785` 用 `self.player.gold += gold; if gold>0 { popup }` 直接加，
> 与 `gain_gold`（`combat.rs:191`，`gold += amount; if amount>0 {Some}`）**同构**；求解器 `ns.player.gold += kill_bounty(..)`
> 与引擎这条路径一致 ⇒ 「没用 `gain_gold`」不构成偏差（`gain_gold` 只被脚本/商贩效果调用：`scene.rs:2673`、`script_ops.rs:83`）。

### 4.2 F3（绿，两黄注）

| 检查 | 结论 |
|---|---|
| 清零隔离 | `solve_layer_with` 入口对 `LAYER_TRUNC`/`GOLD_PEAK`/`SHOP_BUYS` 全部 `set(...)`（`:370-372`），在 `level` 判空**之前** ⇒ 早退也不漏；`solve_layer` 是 `solve_layer_with` 的转发（`:349-357`） |
| 并行串味 | 三量都是 `thread_local!`（`:455-476`），跨线程天然隔离；memo 键含 `floor/entry/player/flags/root_ents/level_digest/policy`（`:389-397`），而 `ClosurePolicy` 派生 `Hash+Eq` 且 `ban: &'static [i32]` 按**内容**哈希/比较（`:1075-1084`）⇒ Q3 的禁用集不会与 Q1/Q2 串味 |
| 采样覆盖 | `expand` = `expand_inner` + 「入口态 ∪ 全部生成态」取最大（`:1553-1570`），而 `expand_inner` 是**唯一**后继生成器（闭包步也在其中）⇒ 没有绕过采样的展开路径 |
| `red_buys=0` 归因 | **成立但仅在生成集合内**：`shop_goods.json` 17 行里**唯一** red 行是 id10 `src1/idx6 price=800`，f12 层峰值 767 < 800 ⇒ 买臂结构上不可能出红（买臂以 `gold>=price` 为前置，`:1594-1620`）。**黄**：该指标无法区分「没走到那家店」与「走到了但买不起」，报告写成「不是搜索取舍」略强 |

黄项（不阻断）：① `shop_buys`/`red_buys` 是**每次层调用的累加**，memo 命中会按缓存值**再放一次**（`:391-395`）
⇒ 583/5133 是「层调用 × 买臂」的量，不是唯一买臂数（报告 §6 风险 5 只声明了「非商品数」）；
② `GOLD_PEAK` 是**生成态**峰值（含被剪态），这个口径报告没写，而它恰好是 §3.5 的关键。

### 4.3 F4（绿）：三发变异我自跑

```
=== M1（删走格臂 ns.player.gold += kill_bounty(...)，solve_local.rs:1635）===
thread '...' panicked at crates/game-solver/tests/solve_global.rs:1579:9:
assertion `left == right` failed: ① 普通击杀余额须精确 = 赏金 100（引擎 scene.rs:2305-2309 非双倍臂）
  left: 0
 right: 100
（连带 solve_global_t连续买两次 也 FAILED）

=== M2（kill_bounty 内 base << 1 → base，:968）===
panicked at crates/game-solver/tests/solve_global.rs:1595:9:
assertion `left == right` failed: ② item25 双倍余额须精确 = 赏金<<1（引擎 scene.rs:2303-2305 双倍臂）
  left: 100
 right: 200

=== M3（删买臂 ns.player.gold -= price，:1606；复现批5 存活变异）===
panicked at crates/game-solver/tests/solve_global.rs:1641:5:
assertion `left == right` failed: ① 余额集合须 == 连买链 {122,72,22}（入场金 0 ⇒ 全部来自击杀）
  left: {122}
 right: {22, 72, 122}
（连带 solve_global_t红钥匙买臂计数_正控 也红了）
```

三发各自还原后 `git diff --quiet -- crates/game-solver/src/solve_local.rs` 均为 **CLEAN**（RESTORED_CLEAN）。
⇒ 批5 那个「扣金整行删掉仍全绿」的存活变异，**确实已被批6 的余额集合断言杀死**。

### 4.4 F5（绿，一保留）

- 三条钉子**都有存在性闸**：击杀给金先 `assert_eq!(real.len(), 35)` + 赏金合计 4839 + `assert!(!r1.goals.is_empty())`；连续买两张先 `r2.expansions > 0`、`shop_buys > 0`；红钥匙正控断言「买红态必须出现」。
- **逐值**：`g.post.gold == g47`（非双倍）/ `== g47 << 1`（双倍）、`gold_peak == g47`/`g47<<1`、余额集合 `{122,72,22}` 与 `{157,107,57,7}` 全集合相等 + `!contains(total-3*price)`。
- **真表**：`load_enemies()` 读 `data/tables/enemies.json`（35 行/4839），`load_shop_goods()` 读 `data/tables/shop_goods.json` 并断言 id8 `price=50 AddKeys{yellow:5}`、id10 `price=800 AddKeys{red:1}` ⇒ 不是硬编码常量。
- 保留：`buy_chain()` 与实现是**同一套闭式算术**（`gold>=price` 逐级减）⇒ 属 B 级复推，不是独立 oracle。

### 4.5 F6（绿）

```
$ cargo run --release -q -p game-solver --bin solver -- query --all --out /tmp/q_run1.json   # 3m56.977s
$ cargo run --release -q -p game-solver --bin solver -- query --all --out /tmp/q_run2.json   # 5m53.9s（与 gates 并行）
Q1 max_hp=0(feasible) / Q2 min_kills=0(feasible) / Q3 reachable=false
```

比对（`json.load` 深比较）：`run1 == run2 == docs/findings/solver-queries.json` **三者完全相同**，
含 q1 `gold_peak=1861 / shop_buys=583 / red_buys=0 / f12=767 / exp=68148 / cap=true`、
q2 `exp=8687 / trunc=[[6,200000],[7,200000]]`、q3 `exp=641 / cap=false / gold_peak=23 / buys=0`。
⇒ 报告 §5 表格逐格为真，且**确定性成立**。
口径未偷改：`truncated_layers`/`node_cap_hit`/`layer_node_limit`/`node_cap_env` 在 `ac179f9..HEAD` 内**零 diff**（值的变化全部由数据/搜索面变化引起）。

### 4.6 F7（绿）：逐 hunk 分类

| 文件 | hunk | 分类 |
|---|---|---|
| `bin/solver.rs` | `solve_global_input` 返回 `(GlobalInput, Vec<ShopGoodDef>)` | 管道（`solve-global` 仍 `&[]` ⇒ 旧入口零变化） |
| `query.rs` | `run_query/run_all` 增 `shops` 透传 | 管道 |
| `solve_global.rs` | `GlobalResult` +4 诊断字段 | 可观测性 |
| `solve_global.rs` | `gold_peak`/`gold_peak_floor`/`shop_buys` 聚合 | 可观测性（不参与决策/键/支配） |
| `solve_global.rs` | `landmark_dead(..., shops)` + §"给该色钥匙的商店也计入" | **语义补边且放宽剪枝**（`shops=&[]` 时恒不计） |
| `solve_local.rs` | `kill_bounty` + 两处击杀臂加金 | **语义补边（本批正题）** |
| `solve_local.rs` | `Cell::Shop` 走臂/买臂 + `shop_offer`/`apply_shop_effect` | 语义补边（批5 H4，在复核范围内） |
| `solve_local.rs` | `GOLD_PEAK`/`SHOP_BUYS` + `expand` 采样包装 | 可观测性 |
| — | **`bound` / `node_cap` / 队列排序 / `Key` / `weakly_ge` / `dominates`** | **零改动**（⇒ 没有调参凑数；也正因为零改动，F2 的红才成立） |

### 4.7 F8（绿）：门禁原样输出

```
$ python3 gates/cli.py check
[✓ PASS] original-integrity / assets-manifest / reference-seed-integrity / knowledge-format
[✓ PASS] docs-links / naming-ledger / rust-tests
[✓ PASS] test-ratchet — total=1084(基线1075) 无回退
[✓ PASS] table-content-hash / table-evidence / state-machine-anchors
[✓ PASS] asset-truth-manifest / state-baseline
== 汇总 == PASS=13 FAIL=0 SKIP=0
```

（首次跑时 total=1085，是因为我自己的探针文件贡献了 `zz_review_f2_probe(test): 1`；删除探针后复跑 = 1084，
与报告「1075 → 1084」一致。`git diff --name-only ac179f9..HEAD | grep -c data/status/baselines` = **0** ⇒ 基线未被改。）

### 4.8 F10（绿）：红线零夹带

```
 crates/game-solver/src/bin/solver.rs               |  21 +-
 crates/game-solver/src/query.rs                    |  18 +-
 crates/game-solver/src/solve_global.rs             | 105 +++++-
 crates/game-solver/src/solve_local.rs              | 204 ++++++++++-
 crates/game-solver/tests/solve_global.rs           | 389 +++++++++++++++++++-
 crates/game-solver/tests/solve_local.rs            |   8 +-
 docs/findings/2026-11-15-solver-queries.md         | 118 ++++++-
 docs/findings/2026-11-16-solver-batch6-report.md   | 187 ++++++++++
 docs/findings/reviews/2026-11-16-solver-batch5-review.md | 352 +++++++++++++++++++
 docs/findings/solver-queries.json                  |   2 +-
 docs/findings/reviews/2026-11-16-solver-roi-segmented-proposal.md | 257 ++++++++++++++
 11 files changed, 1623 insertions(+), 38 deletions(-)
```

`original/` `reference/` `crates/game-core/` `data/tables/` `gates/` **零改动** ✓。
提交链完整性：`a5431dd`（`docs/superpowers/proposals/2026-11-16-solver-roi-segmented-proposal.md`）是**纯文档**，
对批6 功能面**无污染**；但它把两件事夹在一条未推送链里——若把 `d5331af..16db0f5` 当批6 推，建议**先说明 `a5431dd` 的归属**（或让它单独成段），否则 `git log` 读起来像批6 附带了一份设计稿。

---

## 5. 黄项清单（不阻断，但有保留）

| # | 面 | 内容 | 建议 |
|---|---|---|---|
| 黄1 | F1 | `ety < 41` 时模型给 0、引擎是数组下标（越界异常）。当前面上不可达，属「模型更防御」 | 在 `kill_bounty` doc 注明「越界面：模型取 0，引擎不可定义」 |
| 黄2 | F1 | 报告 §6 风险 4「若关卡有倍率则赏金被低估」**事实错误**：`scale_enemy_stats` 不缩放 gold（`combat.rs:80-91`） | 删掉或改写为「赏金与难度倍率无关（已核）」 |
| 黄3 | F9 | 报告 §6 风险 3「`CanonicalBag` 截断可能丢 item25」**无实现落点**：`find_item(25)` 读 `PlayerCombat` 全量字段，bag 只进 GKey（`state.rs:70-89`） | 改写为「bag 截断只影响去重键面，不影响赏金判定」 |
| 黄4 | F9 | 报告 §6 风险 1 把 F2 写成「33 金缺口可能是低估（真实缺口更小甚至已可买）」，**降级了严重性**：它能让可解变不可解 | 按 §3 升格为阻断项，并给出修复方案 |
| 黄5 | F9 | 报告未写 `GOLD_PEAK` 是「生成态采样（含随后被剪态）」，导致 767/1861 的读法含糊 | 补一句口径，并注明「不是不可达证据」 |
| 黄6 | F9 | §5.1 结论 4「队列自然耗尽（模型内穷尽）」未带 F2 限定 | 改为「在 gold 盲剪的搜索图内耗尽」 |
| 黄7 | F3 | `shop_buys`/`red_buys` 是「层调用 × 买臂」，memo 命中会再放一次；583/5133 不能当唯一买臂数 | 报告已声明一半（「非商品数」），补「含 memo 回放重复」 |
| 黄8 | F3 | `red_buys=0` 的归因无法区分「没走到该店」与「走到但买不起」 | 归因句加「（在生成集合内）」 |
| 黄9 | F11 | 赏金公式现在有**三份副本**（`scene.rs:2304-2309`、`battle.rs:187-193`、`solve_local.rs:961-970`） | 建议在 `game-core` 抽 `pub fn bounty_for(enemy_base, ety, player) -> i32`，引擎两处与求解器共用（本批不必做，但要登记） |
| 黄10 | F5 | `buy_chain()` 与实现同式（闭式复推），不是独立 oracle | 可选：用一次真实 `spend_gold` 调用做交叉验证 |

---

## 6. 我最想要的两句话

1. **F2 不是「数值低估」，是健全性缺陷**：同一张图、同一组实体、同一玩家，只因为 BFS 同深度入队次序不同，
   `reachable` 就从 `true` 翻成 `false`——富态（1600 金）被穷态（800 金）按 `hp/kills/steps` 弱支配删除，
   900 金的红钥匙买不到，有解世界被判无解（§3.2 实跑四条，含两条正控）。
2. **批6 现在不能推**（红=1）：修法很小（`weakly_ge`/`dominates` 加一条 `a.gold >= b.gold`，金是单调资源，
   支配安全性可证且不丢剪枝力），但**修完必须重跑三查询并重写「33 金」「穷尽无解」的措辞**——
   这两个头条结论目前只能读作「在 gold 盲剪的搜索图内」的观测值，不能当作世界的性质。

---

## 7. 复核执行记录（可复制）

```bash
# 1) F4 三发变异（各自还原后 git diff --quiet 必须无输出）
cp crates/game-solver/src/solve_local.rs /tmp/solve_local.orig.rs
#   M1: 删 :1635 的 ns.player.gold += kill_bounty(self.level, &player, ety);
#   M2: :968 的 base << 1 → base
#   M3: 删 :1606 的 ns.player.gold -= price;
cargo test --release -q -p game-solver --test solve_global solve_global_t
cp /tmp/solve_local.orig.rs crates/game-solver/src/solve_local.rs && git diff --quiet -- crates/game-solver/src/solve_local.rs && echo CLEAN

# 2) F6 三查询（两次，必须 identical，且等于 docs/findings/solver-queries.json）
cargo run --release -q -p game-solver --bin solver -- query --all --out /tmp/q_run1.json
cargo run --release -q -p game-solver --bin solver -- query --all --out /tmp/q_run2.json
python3 -c "import json;a=json.load(open('/tmp/q_run1.json'));b=json.load(open('/tmp/q_run2.json'));c=json.load(open('docs/findings/solver-queries.json'));print(a==b==c)"

# 3) F8 门禁
python3 gates/cli.py check

# 4) F2 反例（临时探针，源码见 §3.3；跑完删除）
cargo test --release -p game-solver --test zz_review_f2_probe -- --nocapture
```

复核结束时：`crates/game-solver/tests/zz_review_f2_probe.rs` 已删除；
工作区除既有未跟踪文件外无改动（`git status --porcelain` 仅 `??` 项）。
