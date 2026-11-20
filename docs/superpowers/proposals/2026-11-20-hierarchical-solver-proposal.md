# 分层求解器设计提案：块合并 + boss 里程碑分段的全局最优架构

- 日期：2026-11-20 ｜ 角色：spec-writer（`prompts/spec-writer.md`，产出归宿 `docs/`）
- 状态：**设计提案（非实现计划）**——只定结构与判据，不含切片工时与接线代码
- 取代/继承关系：继承并部分取代 `docs/superpowers/proposals/2026-11-16-solver-roi-segmented-proposal.md`（下称「ROI 提案」）的硬门槛/软收益二分与门槛表思想；本文新增**块合并抽象层**与**boss 里程碑分段**两层结构
- 用户裁定（2026-11-20，原文为设计前提，本文不弱化）：
  1. **全局最优**：最小代价满足通行约束，非逐层最优之和。经济链（杀怪→金→商店→属性→减损）是规则；「延迟收割」是涌现行为，禁止硬编码。
  2. **资源约束是多层状态**：钥匙可回早前层补拾/商店买/SwapYellowKey；禁止单层出层硬约束，只做软目标 + 具名诊断。
  3. **规则化**：每个机制/道具对应一条规则，规则进 `crates/game-core/src/rules/`；决策留优化器；禁止对道具/商店硬编码策略。
  4. **道具价值三分类**（实测锚点见 §1.3）。
- 行号说明（继承 ROI 提案惯例）：本文行号为本会话 `grep`/`sed` 实测，引用前请现场复核（行锚漂移是惯犯——本文写作时已发现 ROI 提案的 `scene.rs:2596/2602` 现漂移到 `scene.rs:2478/2484`）。
- 【2026-11-20 修订记录】按复核报告 `docs/findings/reviews/2026-11-20-hierarchical-proposal-review.md`（绿 11/黄 6/红 1）修订：F2 商店扣费语义更正（price=0=免费、id6 免费可重复复利、id12 兑换）+ 新增 §3.5；F3/F8 敌表与档位键归因更正；F4 精确锚与段序重排；F5 回访白名单按资源类型推导；F6 S1 schema 钉死 + S2 消费契约；4 处自引用行锚更新；H4 作废（已闭式）、H6 新增、H5 前置为 S2 强制实验。

---

## §1 问题定义

### §1.1 目标

给定整塔 54 层的 `LevelSpec`（`crates/game-solver/src/solve_local.rs:67`）与初始态（f51 序章起点，`a.java:3484`），求**一条真引擎可逐帧回放的通关路线**，使达成通行约束的**全局代价最小**（主代价轴 = 血量损益；软轴 = 金/步，比较器先例 `crates/game-solver/src/objective.rs:64-68`：hp 大 → gold 多 → steps 少）。

**全局最优 ≠ 逐层最优之和**（用户裁定 1）。反例机制：某层「最优」出层态可能囤血不放金，而下一段商店的 `StatPct +3%`（`data/tables/shop_goods.json` id6，src1 idx1）只有在**层间带金**时才能兑现——逐层贪心永远看不到这笔交易。因此外层必须在段间保留**候选集**（Pareto 前沿）而非单点（§2.2）。

### §1.2 经济链是规则，不是策略

杀怪得金（`kill_reward`，`crates/game-core/src/combat.rs:217-225`：`enemy_base[ety-41].gold`，持 25 时 `<<1`）→ 商店转化（`shop_goods.json` 17 行货单）→ 属性变化改变后续损血（`predict_battle_hp_loss`，`combat.rs:106-131`）。这条链全部发生在规则层；求解器只读规则层的导出面。**任何「先杀 X 再买 Y」的顺序建议都必须由优化器从这条链上算出来**——「延迟收割」类行为（攻略语料的活例证，`docs/findings/2026-11-20-solver-algorithm-survey.md` §1.5）是涌现结果，禁止以硬编码顺序或「层数启发式」实现。

### §1.3 道具价值三分类（用户裁定 4，实测锚点）

| 类 | 语义 | 道具与锚点 | 对优化器的含义 |
|---|---|---|---|
| **常数类** | 拾取即生效，价值 = 道具所在层的 `floor_tier` | 29 威力丹/30 金刚丹（StatUp：f≤10 +1，否则 +tier，`crates/game-core/src/effect_registry.rs` StatUpEffect 臂）；31/32 血丹（HpUp：delta × tier，同文件）；33-40/79/80 装备（tier 固定加成） | 价值是**拾取时点的确定函数**：`value(ty, f)` 可闭式给出，外层只需按时点排序 |
| **时点类** | 价值取决于**使用/触发时刻**的状态 | 19 月桂露 = `(atk+def)×74/10`（`item_effect.rs` Heal 载荷）；击杀金币 ×2 依赖持 25 的时点（`combat.rs:217-225`）；商店 StatPct 的转化率按**店所在层**的属性口径 | 价值必须进外层状态：**「在哪一层花/用」本身是决策变量**，不可预折算 |
| **拓扑类** | 解锁/改变层间图 | 21 上行符/22 下行符（`change_floor(f±1)`，`item_effect.rs:140-150`；floorgraph `EdgeKind::ItemUp/ItemDown`，`floorgraph.rs:26-30`）；17 金勺子/18 玄明石拆 `ty11` 墙；85 黄金钥匙清全层黄门 | 影响**可达集**，须作为外层状态里的能力集（§3.1） |

`floor_tier` 证据：`effect_registry.rs:157-166`（`(f-1)/10` 钳 0、`f>50 ⇒ 0`、`+1`）；断崖表与证据见 ROI 提案 §2.2。

### §1.4 一级需求（用户 m04708 + m04815 追加，独立成节）

本提案的验收核心是下面四条，全部落进 §3 的状态/代价设计与 §5 的判据：

**a. 瓶颈诊断带数字**。当某 boss 的攻/防高于玩家当前可达属性时，求解器不得只报「打不过」，必须输出**达成条件与各路径价格**，且每条价格可复算：
- 拿装备：哪层哪件、tier 值多少（33-40/79/80 的加成来自装备表 + tier 语义，出处 `data/tables/` + `effect_registry.rs`）；
- 吃丹：哪层哪颗丹、`floor_tier(丹所在层)` 值多少（§1.3 常数类）；
- 商店消费：id6 `StatPct atk+3%`（`shop_goods.json` id6）——【2026-11-20 复核 F2 更正】**id6 免费（`price=0` = 不扣费）且可重复购买**（flags&2 购后 `param^=512` 重开报价，`scene.rs apply_merchant_offer` ↔ `a.java:7333-7455` 逐行对账）；`atk += atk*3/100` 是**逐次整数复利**（每次基数变化，`floor(atk*3/100)` 非恒定），可买次数须**迭代模拟**至目标达成或步数预算耗尽，不可用除法折算；id12 是**兑换**（黄-1→金+100，`a.java:7394-7404`）不是购买。其余 14 项货单单价已闭式落表（`shop_goods.json` 17 行与 Java switch 逐条一致）。
  「消费多少」必须可算：诊断输出里每个数字都要能给出来源（表 id + 公式），不允许输出裸的「建议变强」。

**b. 打不动谓词是规则层事实**。`predict_battle_hp_loss` 在 `player_atk <= enemy_def` 时 `result = -1`（`combat.rs:124-131` 的 `if player_atk > enemy_def` 不成立分支）——「打不动」不是启发式，是公式定义的谓词。同理，「太公杖前某些怪打不动」= 能力前置谓词：trait_flags bit0/bit1（49/53/74=1、69=2，`combat.rs:42-47 trait_flags_for`）配合 `effective_attack_vs_type`（`combat.rs:93-104`：持 23 时 bit0 怪攻 ×2、否则持 24 时 bit1 怪攻 ×2）。原文台词锚：太公杖「没有太公杖无法打败巨灵神」`crates/game-core/src/scene.rs:2478`、杨戬门槛 `scene.rs:2484`（【行锚漂移注】ROI 提案写作时的 `scene.rs:2596/2602` 现漂移至此）。外层状态合法性判定必须包含该谓词：** trait 怪在无对应道具时按「不可清除」进入硬阻挡口径**，与 `keydemand.rs` 假设 3（怪按可清）的保守化不同，这里是精确的规则层判定。

**c. 道具锁匙关系数据驱动**。以下关系全部从规则层/`CapsSnapshot`/dynreach 导出，求解器零硬编码：
| 关系 | 数据源 | 锚点 |
|---|---|---|
| 太公杖 23 →（trait bit0 怪）→ 通行 | `combat.rs:42-47,93-104` + `scene.rs:2478` | 能力谓词，非物品特判 |
| 捆仙绳 24 →（trait bit1 怪 69）→ 通行 | 同上 | 同上 |
| 下行符 22 → f0 | `floorgraph.rs` `item_edges`（`EdgeKind::ItemDown`） | 边存在性由道具表导出 |
| 金勺子 17 / 玄明石 18 → `ty11` 墙 | `CapsSnapshot`（`Obstacle{scope,target:11}`）+ dynreach 消除臂 | 能力 spec v2 §1/§3 |
| 门前置 flags（ty83/ty81/ty4） | `crates/game-core/src/rules/gate.rs` `evaluate` + `GateView` | 单一实现，引擎与求解器同吃一份 |
| 通行判定的类目表 | `entity_category_with_floor`（`script_host.rs:138`） | `keydemand.rs` 口径表 1:1 映射 |

已有的硬编码（`state.rs:40 QOL_ITEMS`、`solve_local.rs:3159 [(21,1,false),(22,-1,true)]` 字面量）按能力 spec v2 §6 的改造表逐步消除；本提案不新增任何同类硬编码。

**d. 怪物特化敏感表（边际价值阶梯）**【用户 2026-11-20 追加】。战斗结算对 (atk, def) 是**阶梯函数**：每只怪两个阶跃点——
- **零伤阈值**：`atk > 怪.def`（否则 `predict_battle_hp_loss = -1`，打不动）且反击归零需 `def >= 怪.atk`（`enemy_atk > player_def` 分支不成立 ⇒ result 0，`combat.rs:120-129`）；
- **秒杀阈值**：`atk >= 怪.hp + 怪.def`（`hits = 1` ⇒ 无反击、无 armor_gate 加项，损血 0）。

规则层须为每只怪（`enemy_base` id 41..75 共 **35 行**，`data/tables/enemies.json` 实测；ty76 是 cat1 障碍非怪，`entity.rs:253`）导出敏感表：给定当前 (atk, def)，**距零伤差多少、距秒杀差多少**，以及「+X 攻能掉档哪些怪、+X 防能掉档哪些怪」的边际清单。外层优化器据此排序候选增量（威力丹 +tier(丹层)、商店 id6 +3%、装备 +tier）——「先拿丹让一片怪归零、再把它们当免费金库收割」的顺序价值必须**自然涌现**（经济链规则化裁定），禁止硬编码收割策略。兼容性：`exact_pickup`/`hp_bound` 的 `certified_monotone_pickup`/撞怪零伤折叠判据假设零损血才折叠（`crates/game-solver/src/exact_pickup.rs:19,43,481`；`effect_registry.rs certified_monotone_pickup`）——**敏感表正是判断「何时零损血成立」的依据**，两者互补而非竞争。实现归属见 §4（单一真相源在 game-core）与 §5 判据 7。

---

## §2 两层结构：块合并 → boss 里程碑分段

### §2.0 为什么需要抽象（继承调研结论）

外部调研（`docs/findings/2026-11-20-solver-algorithm-survey.md`）确立三条：key-door 是 NP-hard（Helmert 2001，§2.1）⇒ 全塔逐格精确搜索不可行；事件图折叠（Sokoban macro-move 惯例 + MogicTower 的门/怪建图）⇒ 决策不在地板上；无四要素齐备先例 ⇒ 架构必须自建。本项目已有单层三件套（通用 BFS / exact_pickup DP / pure_pickup 闭式）是单层精确解；缺的是**层间抽象**与**段间权衡**。本文给出两级：块合并（层内 → 块）与 boss 里程碑（块序列 → 段）。

### §2.1 块合并（block merging）

**定义**：层内连通区域按「必经面」切分成块。块 = 一个**接口等价类**：从块外视角，块内部结构被折叠为「入口 → 出口（可多个）」的**导出面**。

- **必经面**：割点意义上的门/怪/咽喉格（`ty1-3` 门、`ty9-12` 阻挡、cat8 怪、cat16 商人、cat32 联动门）。可复用 `keydemand.rs::build_model` 的类目分类（`keydemand.rs:205` `GatePolicy`、`:256 build_model`）与 `dynreach.rs` 的动态可达判定。
- **导出面**：块折叠后的语义 = `(入口格集合, 出口格集合, 通行前置谓词集, 收益向量)`。收益向量含金/钥匙/道具拾取（常数类价值当场折算，时点类只记「在块内可发生」）。
- **三分类标注**：每个块必须标注属于哪一类，禁止混类：
  1. **纯拾取块**：满足 pure_pickup 闭式证书 ⇒ 整块折叠为 `out_hp = in_hp + Σ hp_delta`（`crates/game-solver/src/pure_pickup.rs:1-28`），零搜索；
  2. **单调 DP 块**：拾取集经 `certified_monotone_pickup` 认证（`exact_pickup.rs:481`）⇒ exact_pickup DP，仍精确；
  3. **通用搜索块**：含怪损血/商店/时点道具 ⇒ 通用 BFS + Pareto 支配（label = hp/gold/keys/steps 四维，Feillet 式支配，调研 §3.1）。
- **块的图**：层内块图 + 层间边（`floorgraph::edges_of` 的 105 条楼梯/脚本边 + `item_edges` 的道具边）合成**全塔块图**。事件图折叠依据：调研 §4「决策图不必逐格——以事件节点为顶点，行走代价用可达性最短路折叠」。
- **与 keydemand 的关系**：`keydemand` 的逐层「入场→上行梯」钥匙需求（54 层实测，`docs/findings/2026-11-20-key-demand.md` §6）是块图上的**一个投影**（只保留「到出口」的钥匙维）。块合并不重复实现它，而是把 `analyze_floor` 的模型（`keydemand.rs:515`）作为块切分的实现载体之一。

**块合并是只读分析**：S1 切片（§7）先不接线，只导出全塔接口表（每块的分类、前置谓词、导出面字段），供人审与 S2 消费。

### §2.2 boss 里程碑分段（segments）

**硬锚表**：段边界 = 主线上的**能力硬门槛**（不可绕过、必须在该段内取得）。**表按主线访问拓扑序排列**（【2026-11-20 复核 F4 更正】主线序是 `f51→f52→f53→f54→f1→…→f24→f50`，捆仙绳 f53 在太公杖 f19 **之前**被访问——本表已按此重排，勿按逻辑重要性排序）：

| 段序 | 里程碑 | 精确锚 (层,格) | 前置 | 证据 |
|---|---|---|---|---|
| 1 | 捆仙绳 24 | 71 号 boss **f53 (6,3)**，掉落 @(6,2) | — | `bossEventSpawns` 组[5]（`a.java:2041`）；`docs/findings/2026-11-18-item-acquisition-surface.md` §2.1 |
| 2 | 金勺子 17（拆墙前置） | 67 号 boss **f15 (6,5)**，掉落 **@(6,4)**（非 boss 格） | — | 同上组[3]；掉落坐标 jar sprite 实测（复核 F4） |
| 2' | 太公杖 23 | **f19 (6,3) 静态精灵**（A 级，jar sprite 实测；非台词推断） | 需拆墙手段（17/18；f19 周边格实测**非 ty11 墙**，「藏于墙内」的拆墙路径取证仍欠，见开放问题 2） | `scene.rs:2478` 台词为佐证；精确锚以 sprite 为准（复核 F4） |
| 3 | 杨戬门槛 | 24 / 79 / 80 任一 + 更高属性；79=f48 (8,10)、80=f43 (9,4)（**支线层**） | 依赖支线 traversal（§8.5 裁剪口径随之修正） | `scene.rs:2484` 台词 + jar sprite 实测 |
| 4 | f50 终局 | 仅能从 f24 (6,1) StairUp 进入；唯一门 (6,5) 槽 16 | — | `docs/findings/2026-11-18-floor-topology.md` §三 |

**X1 主线事实**：主线是一条链 `f51→f52→f53→f54→f1→…→f24→f50`（`witness.rs:363-364` 塔序注释；f24→f50 见 topology §三）。P4 复核 X1（`docs/findings/reviews/2026-11-19-witness-p4-review.md` :22）：现行 `up_line_next` 只认 `StairUp`（`witness.rs:367-377`），而 f49 无 StairUp、f50 只能从 f24 进入 ⇒ **主线接口必须用 `solve_local_exit_toward` 的定向语义（`solve_local.rs:547`），不能只靠「上行主线」**。本提案的段间拓扑直接采用 `floorgraph` 全边集（含 ItemUp/ItemDown），X1 的修复是分段外层的自然结果。

**滚动时域（rolling horizon）**：不做全塔一次性优化。外层按里程碑切段 `S_k = [锚_k, 锚_{k+1})`，逐段求解；段 k 的输出是**段间 Pareto 前沿 + 可重入资源表**，段 k+1 以整个前沿为入场候选集（不是单点）：
- **段间 Pareto 前沿**：每条前沿记录 `(hp, gold, keys(y,b,r), 关键道具集, atk, def, flags)`——状态维齐 ROI 提案 §4.2 的 boundary state。为什么必须前沿而非单点：§1.1 的反例（下一段的 StatPct 交易只有「带金」的前沿点能兑现）。
- **可重入资源表**：用户裁定 2（钥匙可回早前层补拾/商店买/SwapYellowKey）⇒ 段边界不是单向门。外层状态含「当前段 + 允许回访的早前段集合」；回访代价由块图上的最短路折叠。`SwapYellowKey`（`shop_goods.json` id12：黄-1 → 金+100）使钥匙与金**互兑**——资源约束因此是多层状态而非层内约束，这是「禁止单层出层硬约束」的机制根源。
- **软目标 + 具名诊断**：keydemand 的教训（`docs/findings/2026-11-20-key-demand.md` §8：needed 是下界、f29 `unreachable` 是假设 5 的产物）直接继承——外层对「段内钥匙缺口」只产出软目标与具名诊断（`needs_more_than_grid`/`blocked_by_assumption` 式标签，`keydemand.rs:85 FloorStatus`），绝不把 needed 变成硬约束。

---

## §3 外层状态空间与算法

### §3.1 状态

外层状态 = **段内块位置 + 资源 + 能力**：

```
OuterState {
  seg: SegmentId,                 // 当前段
  block: BlockId,                 // 段内块位置
  hp, gold: i32,
  keys: (y, b, r),
  atk, def: i32,                  // 一级需求 a/d：属性进状态（瓶颈诊断、敏感表都以它为参数）
  bag: CanonicalBag,              // 关键道具集（复用 solve_global.rs:213 GKey.bag 的 CanonicalBag，保 -1 无限语义）
  flags: u128,                    // 脚本旗标（witness 接力同维：witness.rs:29 out_flags + state.rs:62-64 ScriptFlags(u128)）
  revisits: Set<SegmentId>,       // 允许回访的早前段（可重入）
}
```

要点：
- `atk/def` 进状态是需求 a/d 的结构性后果：瓶颈诊断（a）和敏感表阶梯（d）的「候选增量」都以其为参数；而它们随 StatPct/丹/装备变化——这正是 ROI 提案缺口 G1（`solve_global.rs:251-254` `dominates` 支配维度缺 atk/def/gold）要补的。
- `bag` 用既有 `CanonicalBag`（`solve_global.rs:213`），不再造并行结构（能力 spec v2 §6 红线）。
- `flags` 维与 witness 接力（`witness.rs:29` + `state.rs:62-64`）同构，保证外层解可被回放层直接消费。

### §3.2 单调性驱动的下界表与抽象-细化

**单调性事实**（规则层）：`predict_battle_hp_loss` 对 `player_def` 单调不增、对 `player_atk`（跨过阈值后）单调不增（`combat.rs:106-131`）；闭式/DP 块的折算收益对 atk/def 单调。由此：

- **下界表**：对每个 (段, 目标谓词) 预计算**代价下界表** `LB[seg][atk][def] → (hp, gold, keys) 下界`。构造：按敏感表（需求 d）把段内怪分成「当前属性已零伤 / 未零伤」两档，未零伤档按最优序下界（多怪联合损血下界，可先用 hp_bound 的既有预算面）。表的键是 atk/def 的**档位**（敏感表的阶跃点）：攻侧每怪 2 阶跃（零伤 `> def`、秒杀 `>= hp+def`）35 怪 ≤ **70** 档、防侧每怪 1 阶跃（`>= atk`）= **35** 档【复核 F3 更正：敌表 35 行 id41..75，ty76 是 cat1 非怪】；难度倍率整体平移阶跃点 ⇒ 键须带**倍率维**；持 23/24 时 trait 怪有效攻 ×2 ⇒ 其攻侧阶跃点减半，键须含 **trait 位**。**归因修正（复核 F8）：LB 键有界只决定预表规模，不等于外层状态数可控——OuterState 前沿 ≥8 维，可控性依赖 Pareto 支配 + DSSR；前沿宽度实测前置为 S2 小实验（H5 → §7 S2 判据）。**
- **抽象-细化（abstraction-refinement）**：外层先在**块级抽象**上搜（块 = 单步，代价 = 下界表条目）；若抽象解的某块代价取到「依赖内部顺序」的区间（通用搜索块），才对该块下沉到单层精确三件套细化。收敛判据：抽象下界 == 细化代价 ⇒ 该分支关闭（optimality certificate）。这是对调研结论「label-setting 支配高适配 + 死锁/支配剪枝优先于换算法」（§7 表 #3、MCTS_Sokoban 教训）的直接采纳。

### §3.3 瓶颈诊断算法（需求 a + b + c 的汇合点）

诊断在「外层搜索判定某 boss 块不可通过」时触发，输出结构：

```
BottleneckDiagnosis {
  boss: ety, floor,
  need_atk: i32,    // 使怪可打/可秒杀所需 atk（从敏感表取阶跃点）
  need_def: i32,    // 使反击归零所需 def
  paths: [
    { kind: "equip", item: 33..40/79/80, floor, tier_bonus, reaches: bool, 代价: ... },
    { kind: "pill",  item: 29/30, floor, value: floor_tier(丹所在层), ... },
    { kind: "shop",  good: id6 StatPct, 免费复利：迭代模拟购买次数 n 使 atk 达标（每步 atk += floor(atk*3/100)），并附达成该 n 的路径代价（到店步数×块代价）, entity_flags_at_purchase: ..., ... },
    { kind: "trait", item: 23/24, 触发 trait_flags bit, ... },   // 需求 b：能力前置
  ]
}
```

每条 path 的数字可复算（需求 a）：来源一律是 `enemies.json`（阶跃点）、`items/装备表`（tier 值）、`shop_goods.json`（id6 参数，免费复利路径给买次 n 与迭代模拟轨迹）。shop path 的记录须带**购买时点的实体 flags 字段**（`flags&2` 购后重开报价，`scene.rs apply_merchant_offer`），诊断与回放共享同一份。**前置谓词（需求 b）参与合法性判定**：trait 怪（49/53/74/69）在无 23/24 时 `effective_attack_vs_type` 不翻倍（`combat.rs:93-104`），诊断路径必须把「拿杖」作为独立 path 列出，且外层搜索里该怪按不可清除计——这不是剪枝启发式，是谓词。

**c 的落点**：诊断里「哪层有这颗丹/这件装备/这家店」由获取面给出（`2026-11-18-item-acquisition-surface.md` §2：boss 掉落 + 静态精灵 + 脚本 + 商店四通道全枚举），不写死。

### §3.4 敏感表的优化器消费（需求 d 的算法面）

- 候选增量排序：对当前 OuterState，枚举所有可立即兑现的增量源（丹/商店/装备），按敏感表的**边际掉档清单**打分——「+X 攻使 n 只怪掉到零伤档 / m 只怪掉到秒杀档」，掉档收益 = 这些怪从「需战斗损血」变「零损血可收」的 hp/金差值。
- 顺序价值涌现：一次掉档改变后续拾取的可达性与代价（零伤怪 = 免费金库），该变化经 §3.2 的下界表与段间前沿自然进入后续搜索——**不写任何「先 A 后 B」的规则**。
- 与既有模块的兼容：`certified_monotone_pickup` 认证（零损血才闭式折叠，`exact_pickup.rs:19`）由敏感表提供「此时零损血成立」的判据；敏感表变化（属性掉档）后块的三分类（§2.1）可重算——通用搜索块可能降级为闭式块。

### §3.5 「免费无限 StatPct」对外层状态空间的冲击（复核 F2 红牌新增）

**事实**（A 级，`a.java:7333-7455` ↔ `scene.rs apply_merchant_offer` 逐行对账）：id6 `StatPct atk+3%` `price=0` = **不扣费**（Java `if good.price > 0 && !spend_gold` 才扣金），且购买后 `flags&2` 置位使 `param^=512` **重开报价、可无限重复购买**。推论：

- **atk 理论无上界**：零金成本、零次数上限，`atk += floor(atk*3/100)` 逐次复利 ⇒ 任何「atk ≤ B」的状态都不再是合法的状态空间上界。「上界论证」路线失效。
- **对最优性的扭曲是真实的、不是可忽略项**：任意两个可达同一目标的前沿点，属性高者恒不劣（血量代价单调不增）——atk/def 是**支配轴**上的「越多越好」维，若进支配比较，支配剪枝会把「少买几次」的点全部吞掉，留下「买到底」的单点；但每买一次至少要**走到店并交互**，步数与回访路径代价是真实代价轴。
- **处理方案（三选一，本文给裁决建议）**：
  1. 步数成本界：把「到店交互一次」计为最小步数成本，靠主代价轴的步数惩罚自然限制买次——不封顶，交给支配与 DSSR；
  2. 属性饱和支配：定义**饱和阈值** `sat = 全塔所有怪零伤+秒杀所需的最小 (atk, def)`（从敏感表 70+35 阶跃点闭式导出），`atk/def ≥ sat` 的状态折叠为**饱和态**（属性维塌缩成常量）——超饱和状态互相支配，状态空间重新有界；
  3. 显式开放：v1 把「无限磨属性」排除在搜索集外（每次求解限买次预算），作为近似并显式标注。
  **裁决建议**：v1 采纳 **2+3 组合**——敏感表给出闭式饱和阈值（零硬编码，符合需求 c），v1 搜索限买次预算并输出「预算内最优 + 免费属性未用尽」诊断；方案 1 的无界搜索留作 v2 实验项。⇒ 由此 **H6（饱和阈值假设）** 进 §9。
- **对层内三件套的传染**：`exact_pickup`/`pure_pickup` 的块折叠须声明「块内无商店交互」为前提；含 id6 店的块**强制归为通用搜索块**（§2.1 三分类标注新增一条规则），防止闭式证书漏掉免费属性增长。

---

## §4 与现有模块对接表

| 模块 | 复用什么 | 改什么 | 不动什么 |
|---|---|---|---|
| `crates/game-solver/src/solve_local.rs` 单层三件套（通用 BFS / `exact_pickup.rs` DP / `pure_pickup.rs` 闭式） | 全部保留，作为 §2.1 第三/二/一类块的求解器 | 无需改语义；块分类器调用 `pure_pickup_stat`/`reason_label`（`pure_pickup.rs:192-197`）取判定 | 搜索/剪枝/`ok`/`truncated` 契约 |
| `solve_local_exit` / `solve_local_exit_toward(_shops)`（`solve_local.rs:515,547,565`） | 出层状态接口（`ExitSolve.out_player/out_flags/out_ents`）是段间接力的唯一通道 | 段间接力改调 toward 变体以修 X1（§2.2） | 挑选比较器 `objective::cmp_exit`（`objective.rs:66`，P10 已定稿） |
| `witness.rs` 逐层接力 v2 | 回放骨架与「绝不造 witness」纪律（`witness.rs:16-19`） | 上行主线后继扩展为「段间边选择器」：从 floorgraph 全边集（含 Item 道具边）按外层解选边，替换只认 StairUp 的 `up_line_next`（`witness.rs:367`） | `blocked/capped` 状态语义、实体表按层归属 |
| `floorgraph.rs` | 105 条楼梯/脚本边 + `item_edges` 道具边（实测 213 条、52 扇 ty83 门，本文 §0 复跑） | 无 | — |
| `keydemand.rs` | 类目分类表、假设 1-5、Pareto 需求搜索（`keydemand.rs:515 analyze_floor`） | 不接硬约束（用户裁定 2）；其 `FloorStatus` 具名标签进外层诊断词汇表 | 54 层数字（§7 S1 的对照基线） |
| 能力 spec v2（`docs/superpowers/specs/2026-11-17-item-capability-rules-design.md`） | `CapsSnapshot` 四层单向依赖、`ITEM_USES` 语义、f40 例外 | S0 切片照旧执行；本提案的块导出面消费 `&CapsSnapshot` | `items.json` 拒载红线（BUG-053） |
| `crates/game-core/src/rules/gate.rs` | 门前置 flags 的 `evaluate` 单一实现 + `Relaxations` 显式放宽面 | 无；外层的门前置判定全部走 `GateView` | 「放宽必须随结论一起报」的纪律 |
| `crates/game-core/src/combat.rs` | `predict_battle_hp_loss`（:106）、`effective_attack_vs_type`（:93）、`trait_flags_for`（:42）、`kill_reward`（:217） | **新增**：怪物特化敏感表导出（需求 d 的归属裁定：落 `rules/` 下，与 combat 公式同源直算；solver 只消费导出表，禁止在 game-solver 重算/复制公式或常量——单一真相源纪律，先例：kill_bounty 重写赏金被红牌、C6d 上提 `kill_reward`） | 公式本体零改动 |
| `crates/game-core/src/effect_registry.rs` | `floor_tier`（:157-166）、StatUp/HpUp 的拾取层口径（StatUp/HpUp 臂用 `floor_tier(道具所在层)`）、`ITEM_USES`（:201） | 无 | — |
| `hp_bound.rs` / `objective.rs` | 预算面 / 出层比较器 | hp_bound 的零损血判据接敏感表（§3.4 兼容点） | ratchet 基线 |

---

## §5 判据与回放

1. **外层解必须真引擎逐帧回放**：`solver run` 产出的 witness 链由 game-core 逐帧执行（继承 P4/P5 范式；`witness.rs` 的 `blocked/capped` 语义不变）。任何「最优路线」若不能回放，一律不算数——求解器提议、确定性引擎裁决（调研 §6 的总结论：求解器与验证器共享同一规则层，规则层以引擎为唯一真相源，D5）。
2. **74 金样不动**：本提案全部切片零行为回归判据 = 74 金样逐帧不变；`python3 gates/cli.py check`（docs-only 也跑，写作时基准 PASS=13 FAIL=0）。
3. **每片变异对照表纪律**：继承 keydemand §4 的三发变异体例——每片判据必须配「变异必红」实验，且变异打在不同语义腿上，禁止同一行改三次。
4. **数字先自验**：本文引用的关键数字已复跑（2026-11-20）：`keydemand` CLI 实测 f4=(2,0,0)、f46=(3,0,0)、54 层分布 `(0,0,0)×29 … (3,0,0)×1`、ok 49 层，与 findings §6 一致；`floors` CLI 实测 54 层、213 条边（含 `item_edges` 每层 2 条道具边 = 108，即楼梯/脚本边 105）、52 扇 ty83 门，与 topology findings 一致。实施切片引用数字前同样必须复跑对应 CLI。
5. **需求 a 判据**：构造一个「boss 属性高于当前可达」的夹具层，断言诊断 JSON 含 `need_atk/need_def` 与三条 path，且每条 path 的数字可用表 id + 公式独立复算（测试内复算比对）。
6. **需求 b 判据**：trait 怪（ty49）夹具 + 无 23 背包 ⇒ 外层状态合法性判定拒绝该块（按不可清除）；持 23 ⇒ 放行。断言走 `effective_attack_vs_type` 语义而非求解器自写判断。
7. **需求 d 判据（单一真相源检查）**：game-core 内测试断言敏感表数值与 `predict_battle_hp_loss`/`effective_attack_vs_type` 公式逐点一致（对全 35 怪 × 采样属性网格比对，含难度倍率与 trait ×2 两维的阶跃点平移断言）；game-solver 侧测试**只断言消费了导出接口**（例如 solver 测试里 monkey-不改值、只查「solver 不含 `predict_battle` 公式副本」的 grep 级检查 + 接口调用存在），禁止在 solver 侧重定义任何公式常量。
8. **需求 c 判据**：grep 级检查——本提案新增代码中不得出现 `item == 23`/`item == 24`/`contains(&21)` 式字面量分支（能力 spec v2 §5.4 行为级注入证明同理适用）。

---

## §6 失效前提与不做清单

### §6.1 不做清单（继承调研 §7 不做清单 + ROI 提案）

| # | 不做 | 理由 |
|---|---|---|
| N1 | RL/MCTS 主求解 | 需求是可证明最优 + 确定性回放（调研 §4） |
| N2 | BDD/symbolic search | 网格相邻关系 BDD 不紧凑（调研 §2.2） |
| N3 | PDDL/CP-SAT 主求解 | 经济链（乘性、时点敏感）建模成本高、最优性弱（调研 §5）；CP-SAT 只留作远期交叉验证器缝 |
| N4 | 拉格朗日松弛主算法 | 资源是状态分量非边约束（调研 §3.1） |
| N5 | 单层出层硬约束 | 用户裁定 2；keydemand §8 四前提未裁定前更不许 |
| N6 | 硬编码任何道具策略/收割顺序/商店选择 | 用户裁定 1/3；§5 判据 8 拦截 |
| N7 | 在 game-solver 重算战斗/敏感表公式 | §4 归属裁定（单一真相源） |

### §6.2 失效前提（本设计成立的四个前提，类比 ROI 提案 P1-P4）

| # | 前提 | 失效后果 |
|---|---|---|
| F1 | 怪物表（`enemies.json` **35 行** id41..75；ty76 是 cat1 非怪）在整局静态（难度倍率除外，`combat.rs:80 scale_enemy_stats`） | 敏感表/下界表须按倍率重建（倍率**整体平移阶跃点**，表键带倍率维，§3.2）；档位/敏感表结构不变 |
| F2 | 商店货单与位置静态（17 行货单，店固定层） | 时点类的「店层口径」失效，转化率要进状态 |
| F3 | boss 掉落表 8 组静态（`a.java:2041`） | 硬锚表重建；段边界机制不变 |
| F4 | 主线链 `f51→…→f24→f50` 拓扑由实体表导出且不因 flags 改变**层间结构**（flags 只改门前置） | 段拓扑须动态化（当前 floorgraph 已支持边级 detail，风险可控） |

---

## §7 切片顺序（每片带可测试验收判据）

**S1 块合并只读分析（全塔接口导出表）**
- 产出：`solver blocks [--out]` 导出全塔块图 JSON——每块 `(层, 分类(闭式/DP/通用), 入口集, 出口集, 前置谓词集, 收益向量, 三分类标注)`；零代码接线。
- **Schema 钉死（复核 F6）**：导出表字段类型在 S1 落地前冻结并在文档内声明——收益向量**固定 4 维** `(dhp, dgold, dkeys[y,b,r], flags_delta)`；时点类道具不折算、以 `Deferral{item, at_block}` 结构表示；前置谓词统一序列化为 `PredicateRef`（规则层谓词注册表的 id + 参数），**禁止自由文本**。S2 消费契约：外层只接受该 schema，schema 不符直接 panic（fail-fast），并用 schema 断言测试锁定。
- 验收判据（可测试断言，非「测试全绿」）：
  a. f4 块图含 ≥2 个块且两块间的必经面 = keydemand 口径的门集合（对照 `docs/findings/2026-11-20-key-demand.md` §6 f4 行：8 门/6 拾取/2 硬阻挡）；
  b. 纯拾取块的集合 == `pure_pickup_stat` 当前全塔可闭式层集合（两实现互证）；
  c. 全塔块数与块边数与手抽 3 层（f24 双出口、f41 21 门、f29 阻挡）人工核对一致；
  d. 变异：把某块的前置谓词删一条 ⇒ 导出表该块标注变化（变异必红）。

**S2 外层骨架（状态机 + 敏感表 + 下界表）**
- 产出：`rules/` 敏感表导出（game-core）+ `OuterState`/前沿/回访状态机骨架 + `LB[seg][atk档][def档]` 下界表。**S2 消费契约（复核 F6）：OuterState 只消费 S1 schema 的 `PredicateRef`/收益向量接口，块表内字段零直读。**
- 验收判据：§5 判据 5/6/7 全绿；下界表对 3 个采样状态给「下界 ≤ hp_bound 真值」断言；敏感表与 combat 公式逐点一致测试绿；**H5 前置实验（复核 F8）：夹具段上实测段间 Pareto 前沿宽度并落表（记录维数/支配存活率），前沿宽度 > 1e4 时触发 §8.4 降级预案——该数字是 S3 架构可行性门槛，不是可选观测**；§3.5 饱和阈值闭式导出并附「sat 态折叠」单元测试。

**S3 分段求解（滚动时域 + 段间 Pareto + 诊断）**
- 产出：逐段求解器 + 段间前沿消费 + 瓶颈诊断输出。
- 验收判据：a. 段间前沿非单点（构造夹具：带金前沿点在下一段兑现 StatPct，断言该点不被单点丢弃）；b. f46（需带入 3 黄钥，findings §7 点名）在诊断中报告钥匙软目标而非硬约束失败；c. 诊断 JSON 每个数字可复算（§5 判据 5）；d. 变异：删掉某段前沿的 gold 维 ⇒ 下一段 StatPct 兑现断言红（变异打在「前沿完备性」腿上）。

**S4 回放闭环（witness 接段间边 + X1 修复）**
- 产出：witness 的边选择器改为消费外层解（含 Item 边与定向 toward），跑通 `f51→…→f24→f50` 全链回放。
- 验收判据：a. X1 复审用例转绿：`solver run` 在真数据产出 `status=witness` 且 `final.ending_done=true`（P4 复核 :22 的失败模式不再复现）；b. 每段边界 `out_player==in_player / out_flags==in_flags / out_hp==in_hp`（P4 复核 F1 同款逐层断言）；c. 74 金样不变（§5 判据 2）；d. 变异：witness 边选择器退回只认 StairUp ⇒ X1 用例红（变异打在 X1 腿上）。

顺序：S1 → S2 → S3 → S4（S1 只读零风险先行；S2 是 S3 的前置；回放闭环最后收口）。

---

## §8 未决问题

1. **f29 口径**：`unreachable`（findings §9）vs 当前树的 `blocked_by_assumption`（本会话复跑实测，xcheck 53/54、f29 一处不一致——工作区有未提交的 keydemand 改动，属 P14 在途结论）。诊断词汇表须统一两者；f29 真实前置（ty72 联动门 + ty11 墙）取证仍欠（findings §11.2）。
2. **P14 在途结论**：keydemand 标签重命名与 xcheck 口径收敛未定稿；本文按「当前树实测」引用并标注，不以 findings 旧值为准。
3. **商店扣费语义【2026-11-20 复核 F2 已闭合】**：`price=0` = **免费**（`a.java` `if good.price > 0 && !spend_gold` 才扣费），id6 StatPct 免费且可重复（flags&2 购后 `param^=512` 重开报价），id12 是兑换不是购买；全部 17 行货单的扣费语义已与 `a.java:7333-7455` 逐行对账闭式落表。初稿「单价待取证」叙事作废。**残留开放点**：免费无限 StatPct 引出的 atk/def 无上界与饱和阈值口径见 §3.5 与 H6。
4. **覆盖 vs 精确的取舍**：抽象-细化（§3.2）的收敛判据在通用搜索块密集的段（f41/f46）可能退化为全细化；label 爆炸时先试 DSSR（调研 §7 #3），DSSR 也爆则该段降级为「覆盖式近似 + 显式标注」，不许静默。
5. **可重入状态空间**：`revisits` 集合的幂等展开可能爆炸。【2026-11-20 复核 F5 修正裁剪口径】初稿「只允许回访有商店/钥匙拾取的段」会**误伤能力支线**：杨戬门槛（24/79/80）依赖 f48 (8,10) ty79、f43 (9,4) ty80，幸运币 25 依赖 f41 下行符——这些段没有商店/钥匙拾取却被硬需求。改为**「回访段白名单按资源类型推导」**：白名单 = 含 {商店(id6/id12 可交互)} ∪ {关键能力道具拾取面（`CapsSnapshot` 能力集差分非空）} ∪ {钥匙拾取/兑换点} 的段，推导规则数据驱动、逐段在诊断里具名输出理由；仍要作为假设进诊断输出。

---

## §9 假设区（C 级，不得混入结论）

| # | 假设 | 级 | 验证途径 |
|---|---|---|---|
| H1 | 「延迟收割」类涌现行为在当前数据上确会被外层选中（即经济链的时点收益差足够大到改变搜索选择） | C | S3 落地后对照逐层贪心基线的差值实测 |
| H2 | 下界表的档位键（攻侧 ≤70 × 防侧 35 × 倍率 × trait 位）使 LB 表规模可控（<1e4 条目/段） | C | S2 实测表规模 |
| H3 | f50 唯一门（槽 16）后的 ty72 召唤与 `load_level_script(65)` 时序不构成额外段边界（topology §五 P5 未决） | C | S4 回放实测 |
| H6 | §3.5 的饱和阈值（全塔零伤+秒杀所需最小 atk/def）闭式可导且折叠后前沿不产生等价环 | C | S2 单元测试 + 夹具段实测 |
| H5 | 段间前沿规模在 ≥8 维下可被支配 + DSSR 有效剪枝（不爆炸）。【复核 F8：已从纯假设前置为 S2 强制小实验，见 §7 S2 判据；仍属 C 级假设，实测前不进结论】 | C | **S2 判据强制实测**（前沿宽度落表） |

---

## 本提案遗留的开放问题（呈用户）

1. **f29 / P14 口径收敛**：`unreachable` vs `blocked_by_assumption` 标签统一 + f29 真实前置取证，是否在 S1 前先做独立小切片？
2. **太公杖拆墙路径取证**（复核 F4 衍生）：f19 (6,3) 静态精灵锚已 A 级，但 f19 周边格实测**非 ty11 墙**、且 f19 无 ty9-12 实体——「需 17/18 拆墙方可抵达」的具体路径与前置取证仍欠（原「商店单价取证」问题已按 F2 闭合移除）。硬锚表 2' 段的入场谓词在取证落定前按保守口径（需 17）建模，是否先做独立取证小切片？
3. **可重入裁剪口径**（§8.5）：「只允许回访有商店/钥匙拾取的段」这条裁剪是否可接受为 v1 口径？
4. **覆盖降级的报告格式**（§8.4）：某段降级为近似时，验收契约要求什么（gap 值？上界？人工复核清单）？
5. **敏感表导出的 CLI 形态**：`solver sensitivity --out` 是否需要门禁基线（第 14 项账单），还是只做 game-core 内一致性测试（§5 判据 7）？
