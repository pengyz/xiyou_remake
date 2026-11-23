# S6 —— 拾取/需求对齐（pickup-demand alignment）实施方案

> 状态：提案（待裁定）。作者：s6-plan（对抗性评审 + 方案设计）。日期：2026-11-23。
> 唯一改动面：本文件。未改代码、未改工作树、未 push。
> 证据分级遵 AGENTS.md §3：本文用到的 A 级=落盘 trace/JSON/实测命令输出；B 级=可复现静态推理（含行号）；
> C 级=推测（只在 §7/附录出现，不进结论）。

## 一句话结论

**选方案 C = A′（单层软目标，落点在「精确 DP 的终态比较器」而不是 `cmp_exit`）+ B′（外层账本降级为「估计」并新增 credited-vs-delivered 审计，把双记账变成具名残差）**，因为
（i）lead 的方案 A 按字面实现**对 f3 零效果**——f3 走 `exact_pickup`，每出口只产 1 条 `LayerGoal`，`cmp_exit` 连第二个候选都看不到（`exact_pickup.rs:293-305`）；
（ii）方案 B 单独会把「f4 走不通」变成「连能到 f4 的候选都没有」，不推进验收目标；
（iii）C 的两次改动都在**已存在的数据流**上打补丁，默认 `key_floor=None` ⇒ 74 金样与 f3 金样逐字节不变。

---

## 0. lead 已知事实复核台账（不照抄）

| # | lead 的说法 | 复核判定 | 我的证据（锚） |
|---|---|---|---|
| 1 | seg1 走通 11 层 `[51,52,53,54,1,2,3,2,1,2,3]` 后 blocked 于 f4 `no_exit_toward` | **确认** | `docs/findings/outer-chain-real.json` → `segments[0]`: `status=blocked`, `floors_solved=11`, `blocked={"floor":4,"reason":"no_exit_toward","detail":"4 无指向 f5 的出层解"}`；`pops=4553792`, `frontier_len=3630`, `frontier_alive=2645`, `candidates_considered=5`, `truncated=false` |
| 2 | f3 地上 8 把黄钥匙，**「内层一把黄钥匙都没捡」** | **数字对、结论错（推翻）** | 8 把位置确认（`docs/findings/block-interfaces.json` f3 各块 `exports`：ty26 于 (1,1)(4,1)(4,3)(5,2)(6,1)(6,3)(2,8)(11,8)）。但 f3 第 2 次访问**捡了 5 把、花掉 5 把、出层带 0**：route 里 5 个 `door` token（`door 1,4`/`door 1,6`/`door 9,2`/`door 9,8`/`door 9,11` = 5 次 door1 开门），同层 `ents_out` 仍剩 ty26 三把于 (6,1)(4,3)(1,1) ⇒ 8−3=5 把被拾取；蓝钥 ty28(5,3) 不在 `ents_out` ⇒ 捡 1 带 1 ⇒ `keys_out=[0,1,0]` 自洽。第 1 次访问 `route=["1,11"]`（1 token 向下换层）才是真的什么都没捡 |
| 3 | f4 入场最低需求 (2,0,0)，门在 **(11,10)**，无黄钥匙打不开 | **需求确认，坐标要更正** | (2,0,0) 与 `docs/superpowers/proposals/2026-11-20-hierarchical-solver-proposal.md` §5.4 的 keydemand 实测一致（`objective.rs:36` `next_demand` 调 `keydemand::analyze_floor`）。方块表里 f4 的两扇必经 door1 是 **(11,8)**（f4b1）与 **(1,8)**（f4b2），不是 (11,10)（`block-interfaces.json` f4b1/f4b2 `cut_doors`）。上梯 ty7 在 (1,11)→f5 |
| 4 | `outer.rs:841 settle_entry_pickup` 乐观记账，不看内层实际捡没捡 | **确认** | `crates/game-solver/src/outer.rs:841` 起：块首次进 `state.picked` 后，遍历 `ex.class=="const"` 的 export 逐件 `certified_monotone_pickup` 入账（hp/atk/def/三钥/bag）；调用点 `outer.rs:1022`（搜索 pop）、`outer.rs:1113`（replay 快照）。函数体内无任何「内层路线是否踩到该格」的判断 |
| 5 | 内层目标 `cmp_exit` = hp 大→gold 多→steps 少，无钥匙维度；f3 走 `exact_pickup` | **确认且更糟** | `objective.rs:66-70` 逐字：`b.hp.cmp(&a.hp).then(b.gold.cmp(&a.gold)).then(a.steps.cmp(&b.steps))`。**但**两条快速路径不只「绕过维度」，而是**在更早的地方把候选集裁掉了**：`exact_pickup.rs:293-305` 每个出口只留一个终态（`hn > hb ‖ (hn==hb ∧ n.steps<sb)`），`goals` 每出口 1 条 ⇒ `cmp_exit` 无选择余地。`pure_pickup` 全拾取，钥匙天生最大 |
| 6 | m04638：禁止单层出层硬约束，只做软目标，缺口具名 `key_shortfall` 不改 `ok`；m04650：最优=全局最小代价 | **遵守**（§1.3/§2 显式对齐；与 `objective.rs:20-31` 模块 doc 的「demand 绝不进入比较」存在张力，见 §1.3 处理方式） | `objective.rs:20-31`；`tests/objective.rs:9-13` 判据清单；`tests/objective.rs:165-175` 钉死断言 |

**新增（lead 未提，但决定方案形状）**：外层账本的偏差**不止钥匙**。`budget_drifts` 实测：f53 处 outer `atk=12 / def=11 / gold=18 / hp=-215`，内层真值 `atk=730 / def=730 / gold=1021 / hp=427`（`outer-chain-real.json` → `witness.budget_drifts`）。原因：f52 入场 `atk10/def11` → 出层 `atk730/def730` 是**脚本跳变**，块表里没有任何这样的 export（f53 的 exports 只有 2 件 ty32 长寿丹）。⇒ 外层 `killable/fatal`（决定候选顺序与 5 次尝试）是用一份假属算出来的。这一点决定：**B 的「以交付为真相」如果照字面做，会把整个外层成本模型连根拔起**（§1.2）。

---

## 1. 对抗性评审（A / B / C）

### 1.1 方案 A（`key_floor` 软目标，改 `cmp_exit`）：**红**（按字面）→ 改落点后 **黄转绿**（A′）

**判定**：按 lead 字面描述（`cmp_exit` 改成「满足 key_floor 优先→hp→gold→steps」）判 **红**，最小反例只有一条但致命：

> **最小反例（A 级实测）**：f3 第 2 次访问，`toward=f4`、`key_shortfall={need:(2,0,0),have:(0,1,0)}`，
> 而该层走的是精确 DP（金样 `tests/exact_pickup.rs:314 exact_dp_f3_reaches_4027` 断言 `st.adopted`）。
> DP 在 `exact_pickup.rs:293-305` 已经**每个出口只放一个终态**进 `goals`，`cmp_exit` 的输入集合大小 = 出口数（f3 最多 3）。
> 所以在 `cmp_exit` 上加 key_floor，**无论怎么排序都改变不了 f3 的出层态**。

逐条回答 lead 的三个攻击点：

**① 等血时软排序能否真的改变路线？（用数据回答，不靠推理）**
分路径，答案不同，这正是方案 A 的病根：
- **普通 BFS 路径：能**。`solve_local.rs:2900-2915`：每到一个出口格就 `goals.push(Goal{…keys:(yellow,blue,red)…})`，**逐终态入列、不按出口裁剪** ⇒ `goals` 里同一出口可以有多个不同拾取子集 ⇒ `best_goal`（`solve_local.rs:653`）有真实选择空间，key_floor 插进去会改变选择。
- **精确 DP 路径：不能**（见上）。f3 走的就是这条。
- **纯拾取闭式路径：不需要**。它一次捡全部（`pure_pickup.rs`），钥匙计数天然是上界。

顺带把「捡钥匙要掉血吗」问清楚：**不掉**。f3 剩下的 3 把黄钥 (6,1)(4,3)(1,1) 全都紧邻现有 route 的 token（route 里有 `5,1`/`6,2` 邻 (6,1)、`4,2`/`5,3` 邻 (4,3)、`1,2` 邻 (1,1)），拾取本身不耗血、周围没有怪 ⇒ 代价只有步数。**这是 B 级推演 + A 级路线数据**，最终以 §5 的 S6a 判据实测为准。

**② `exact_pickup` / `pure_pickup` 是否完全绕过 `cmp_exit`？**
- `exact_pickup`：**是，且是结构性的**。`solve_local.rs:911` 在通用 `Search` 之前调用 `try_exact_dp`，命中即 `return (true, goals)`（`solve_local.rs:919` 附近），后续 `cmp_exit` 只能在这 ≤出口数 个 goal 之间挑。⇒ A 对 f3 无效，最小反例即上面的实测。
- `pure_pickup`：**是，但无害**。全拾取 ⇒ `keys_out` 逐色单调最大 ⇒ 任何 `need` 都可满足（若本层钥匙够）。但它不看需求，可能多捡（多花步数）；f54 出层 3227hp/`adopted` 是既有实测。

**③ 软排序会不会牺牲 hp 换钥匙？判据怎么划？**
取决于 key_floor **插在哪一位**，这不是风格问题，是 `tests/objective.rs` 已经钉死的东西：
- 若按 lead 字面放在 **hp 之前** ⇒ 直接打红 `tests/objective.rs:165 demand_fields_are_diagnostic_only` 的 A 段（`score(4100,0,0,90) vs score(4000,0,[2,0,0],88)` 断言 `Less`＝血多者必须赢）。同时违反与「不为攒钱牺牲血」（`objective.rs:14-19`）同源的原则：hp 是不可逆的生存货币。
- 放在 **gold 之后、steps 之前**：`hp → gold → 满足 floor → steps`。此时
 - 既有两条钉死断言**全绿**（A 段：hp 不同 ⇒ keys 无从翻盘；B 段：`score(4027,500,0) vs score(4027,100,[2,0,0])` ⇒ gold 不同 ⇒ keys 无从翻盘）；
 - 钥匙只能吃掉**步数**这一维，**永不牺牲 hp/gold**。
- **判据划法（写进 S6a）**：`hp` 与 `gold` 必须与 `key_floor=None` 时**逐值相等**，只允许 `steps` 增加，且增加量必须有上界（f3 期望 ≤ +6 步）。这条判据本身就是「会不会亏」的答案：不会——因为排序位次已经把它限制成纯步数开销；若实测出现 hp 下降，说明 DP 的单调性假设被破坏（捡钥匙的路线绕过了加血件），必须当**缺陷**处理而不是「拿血换钥匙」的合理代价。

**结论（A）**：字面 **红**；把落点从 `cmp_exit` 下移到「DP 终态比较器 + 排序位次修到 gold 之后」之后 **黄→绿（A′）**，且**不需要**删改任何既有断言（只需给模块 doc 与判据名补上「keys 只在 hp/gold 之后断 tie」的新契约，并新增两条断言）。

### 1.2 方案 B（外层以内层交付为真相）：**红**（单独）

**判定红，理由不是「想法错」，而是「它修的是症状、代价是模型的根」**：

**① 会不会失去前瞻？（会，而且是致命的）**
如果 `settle_entry_pickup` 只采纳内层实际交付，那么外层状态里**永远不会有任何一把没被内层主动捡起的钥匙**。而内层只被要求「指向下一层的出层解」——它对 f4 的 `(2,0,0)` 一无所知（`outer_witness.rs:200` 的 layer 调用签名里没有 demand；`outer_witness.rs:385` 算出来的 `demand` 只喂 `record_floor` 做诊断）。
> **最小反例（B 级推演 + A 级现状）**：f4 的入场需求是 (2,0,0)，f3 实际交付 (0,1,0)。B 生效后，外层所有通向 f4 的候选在到达 f4 时都得 `state.yellow ≥ 2` 才能继续；而外层能记的钥匙最多是内层恰好带出来的 (0,1,0)。⇒ `apply_outer_edge` 的 `state.yellow < y` 分支（`outer.rs` 的 trans/edge 校验）把**每一条**到 f4 的路都拒掉 ⇒ witness 拿不到可消费候选。`blocked.reason` 会从 `no_exit_toward@f4` 变成「无候选」，`floors_solved` 从 11 掉到 ≤6。这不是进展，是退化。
> lead 说的「回拾机制已能工作」需要更正：`[51,52,53,54,1,2,3,2,1,2,3]` 里的下潜回访是**外层路径自身的折返**（`path_to_tasks` 把连续同层块归组），不是「因为缺钥匙所以回拾」的机制；代码里不存在钥匙驱动的回拾触发（`outer_witness.rs` 全文无 repair/backtrack 分支）。S5 的「4 次重访」是路径折返计数，不是机制存在的证据（S5 复核 F7 已把「回拾=3」更正为 4，但那是口径修正，不改这个结论）。

**② 边权/成本模型是否依赖 exports 的乐观值？改动面多大？**
依赖，且改动面是**结构性的**：`block_combat_cost`（战斗 LB 扣血/加金）、`apply_outer_edge` 的钥匙/属性校验、`dominated_seen` 的 11 元组、`pareto()` 支配、`replay_outer_path` 的快照、`path_to_tasks` 的 `entry_budget`，全部吃 `settle_entry_pickup` 写出的那份 `OuterState`。要把它换成「内层交付」，只有两条路：
- (a) 在搜索内调内层求解器 —— `pops=4,553,792`，每次层解秒级 ⇒ 天文数字，直接排除；
- (b) 把搜索改成「逐层 rollout」——那就不再是 frontier 搜索，`pareto/dominated_seen` 全部作废，等于重写 S2/S3。

**③ 性能：状态空间会不会爆炸？**
会。真实状态（尤其是脚本跳变后的 atk730/def730）让整数支配几乎失效：`dominated_seen` 靠「同 floor/pos 且四维不劣」去重，而真实 hp/atk/def 是陡变的连续量（f52 一次 +720/+720），Pareto 前沿会退化成逐态保留；再叠加 rollout 的不可回溯性，frontier 会从 3630 涨到不可控。
**旁证**：现有账本虽然假，但它的**假是均匀的**（同一块序必得同一账），所以去重/支配能工作——这正是它现在还能在 4.5M pops 内跑完的原因。

**结论（B）**：单独 **红**。可取的部分抽出来做 **B′**：不把账本当真，但**把「账本承诺 vs 内层交付」的差额作为具名残差落盘**，并用它驱动链路级 repair（§2.4、§4.3）。这条改动小、可测、能立刻回答「哪一层欠账」。

### 1.3 方案 C = A′ + B′：**绿**（带条件）

| 维度 | 判定 | 说明 |
|---|---|---|
| m04638（禁单层硬约束） | 绿 | A′ 只改**出层态之间的偏好次序**，`ok = reached_goal && !truncated` 不动；找不到满足 floor 的终态时仍返回 hp 最优解 + `key_shortfall` 具名 ⇒ 单层从不因钥匙判失败。B′ 的 repair 在**链路层**触发，不在单层 |
| m04650（全局最小代价） | 绿（带已知限度） | floor 只吃 steps 维，hp/gold 不变（§1.1③ 判据钉死）⇒ 付出的代价是可核算的步数；限度见 R4/R8（`next_demand` 是一步前瞻且逐色取最大，可能过付步数，但不会过付 hp/金） |
| 与 `tests/objective.rs` 既有断言 | 绿 | 两条钉死断言逐字保留（keys 位次在 gold 之后 ⇒ 二者都不可能被翻盘）；新增 2 条断言表达新契约 |
| 回归面 | 绿 | `key_floor=None` 时新比较器必须**与 `cmp_exit` 全等**（判据 `cmp_exit_with_floor_none_eq_cmp_exit` 采样对比）⇒ 74 金样 + f3 金样（4027/81/14）逐字节不变 |
| 端到端收益 | 黄（未证） | 能否**真**把 seg1 推过 f4，只有 S6c 那一次真链跑能回答；这不是设计缺陷，是证据缺口，必须明码标注 |

**代价（显式）**：`solve_layer_with` 的签名与 `LAYER_MEMO` 的键各加 1 元（10→11 元）；`try_exact_dp` 加 1 参；`ExactStat` 加 2 个诊断字段；`OuterWitness` 加 1 个审计字段。四签名 + 一 memo 键 + 两结构体字段，属于「小改动、大表面积」，必须靠编译期与判据一起守。

---

## 2. 机制设计（数据流与权威表）

### 2.1 权威表（谁是真相，谁只是提示）

| # | 交接口 | 生产方 | 权威级别 | 消费者 | 禁止 |
|---|---|---|---|---|---|
| 1 | 每层钥匙需求 `Demand{keys:[u8;3],state}` | `objective::next_demand(levels,floor)`（`objective.rs:36`）→ `keydemand::analyze_floor`（真地图 BFS） | **提示（下界）**，可保守 | 链路 → 单层 `key_floor` | 不得据此判失败；不得写进 `ok` |
| 2 | 单层出层目标 `LayerGoal{hp,gold,keys,steps,route,post}` | `solve_layer_shops`（三条路径之一） | 层内**真值**（`validate_outcomes` 逐步回放核对，`exact_pickup` 撞不上就 `Off`） | `cmp_exit_with_floor` → `materialize_exit` | 不得用外层账本篡改 |
| 3 | 单层交付 `ExitSolve{out_player,out_flags,out_hp,keys_out,route,…}` | 真引擎（`materialize_exit`） | **唯一真相** | witness 的 `cur_player`/`cur_flags`（**现状已如此**：`outer_witness.rs:434` `cur_player = got.out_player.clone()`，`:477` 附近 `cur_flags = got.out_flags`） | 禁止任何「外层推断值」覆盖它 |
| 4 | 外层账本 `OuterState`（`settle_entry_pickup` `outer.rs:841`） | 块表 exports 乐观入账 | **估计**（仅用于路径提议/候选裁剪） | outer search、`replay_outer_path`、`path_to_tasks.entry_budget` | **禁止**写回 `WitnessFloor.out_*`；**禁止**参与 `budget_drifts` 之外的任何判定 |
| 5 | 账本审计 `credit_audit{credited,delivered,missing}` | 新增：block exports 的 `const` 清单 vs 内层 `ents_in/ents_out` diff + `keys_out` | **诊断（具名残差）** | 链路 repair（§4.3）、JSON | 不得被当成 ok/失败依据 |

### 2.2 双记账怎么被「设计掉」

现状其实**没有把两本账混用**（第 3 行已是唯一真相），混的是**候选提议的质量**：外层拿第 4 行的假账去决定「值不值得走这条路」。所以「设计掉双记账」= 把第 4 行**显式降级**并由第 5 行兜底，而不是让第 4 行去抄第 3 行（那要付出 §1.2② 的代价）。三条硬规则 + 三条可执行判据：

- **R1（结构）**：`WitnessFloor` 的每个 `out_*` 字段只能由 `got.out_*` 赋值。判据 `witness_out_fields_come_from_inner`：在测试注入的 layer 桩里让 `got.out_hp` 与 outer 估计值不同，断言落盘的是 `got` 的值（变异：改成读 outer 估计 ⇒ 红）。
- **R2（边界）**：`entry_budget` 只进 `BudgetDrift`，不进比较/过滤。判据 `entry_budget_not_consumed`：把 `entry_budget` 整体置 0，断言候选集合与 `blocked` 完全不变（变异：某处误用 ⇒ 红）。
- **R3（具名残差）**：新增 `credit_audit`，对每个任务列出 `credited`（外层记的 const exports）与 `delivered`（内层 diff 出来的实际拾取）的差。判据 `credit_audit_f3_reports_missing_keys`：f3 第 2 次访问必须报出 `credited_yellow=8, delivered_yellow=5, missing_yellow=2`（8 地上、捡 5、还需 2 带走）。变异：审计恒填空 ⇒ 红。
- 附带：`settle_entry_pickup` 更名/注释为 `estimate_entry_pickup`（只做语义标注，不改行为），让「乐观」在代码里可 grep、不再是隐藏事实。

### 2.3 A′ 的传播链（签名级）

```
next_demand(levels, task.floor)                     // outer_witness.rs:385（已有，只进诊断）
  └─ key_floor: Option<[u8;3]> = 条件式（见下）
       └─ layer(input, entry, &cur_player, cur_flags, ents, task.toward, shops, key_floor)
            // outer_witness.rs:200 注入点：solve_local_exit_toward_shops(+1 参)
            └─ solve_layer_shops(+1) → solve_layer_with(+1)   // solve_local.rs:404 / :776
                 ├─ pure_pickup 闭式（忽略 key_floor：全拾取已最大）
                 ├─ try_exact_dp(+1)  → exact_pickup::run(...) → 终态比较器（改）
                 └─ 通用 Search → goals 逐态 push → cmp_exit_with_floor（改）
            └─ LAYER_MEMO 键 10 元 → 11 元（+ Option<[u8;3]>）   // solve_local.rs:1022-1040 键类型
  └─ record_floor(..., keys_out, key_shortfall, credit_audit)    // 落盘
```

**`key_floor` 的条件式**（关键，防误伤第 1 次访问）：

```
key_floor = if graph 存在 floor → task.toward 的 StairUp 边 { Some(need) } else { None }
need      = next_demand(levels, task.floor).keys     // 逐色取最大，保守
```

理由：`next_demand` 是「本层所有 StairUp 后继的最大需求」。f3 第 1 次访问 `toward=f2`（**下行**折返，route 只有 1 token），若把 f4 的 (2,0,0) 传给这次访问，就会在一条向下换层的 1 步转场上强行捡钥匙 —— 那才是真正的「单层硬约束」误伤。条件式把 key_floor 严格绑定到「这次访问的目的地是下一层」。

**比较器（唯一真相点）**：

```rust
// objective.rs 新增；cmp_exit 保留为 with_floor(None) 的薄包装（既有断言不动）
pub fn cmp_exit_with_floor(a, b, f: Option<[u8;3]>) -> Ordering {
    b.hp.cmp(&a.hp)                                        // 软1（不动）
     .then(b.gold.cmp(&a.gold))                            // 软2（不动）
     .then(match f { Some(n) => meets(b,n).cmp(&meets(a,n)) // 新增：满足 floor 者优先
                   , None => Ordering::Equal })
     .then(a.steps.cmp(&b.steps))                          // 软3：只在这里分胜负
}
```

`exact_pickup.rs:299-305` 的终态 `better` 判据改为调用同一函数（避免两处比较器漂移），并保留 `hp_at/steps` 取值的现有形状。

### 2.4 链路级 repair（B′ 的落点，S6d 条件触发）

不引入新搜索：`repair` 就是**在链路层把 `credit_audit.missing` 变成一次额外的层任务**：

```
若 任务 T 的 solve 产生 key_shortfall{need,have} 且 hat != 0（即仍缺）
   候选 = 已访问过的层中，其 const exports 含缺失色钥匙且「该格未被 delivered」的最近一层 L
   若 L 有下行边可达：在 T 之前插入一次 L 的访问任务，key_floor = 剩余缺口，toward = T.floor
   否则：具名 key_shortfall 落盘 + blocked.reason="key_shortfall"（不许静默）
```

这条链路复用外层已有的「路径可折返」能力（现路径 `…3,2,1,2,3` 已经折返），不新增搜索维度；但**只在 S6c 实测证明一步前瞻不够时才实现**（§8 排序理由）。

---

## 3. 三条快速路径的落位

| 路径 | 现状 | A′ 下的行为 | 是否要改 | 适用范围 / 降级 |
|---|---|---|---|---|
| **`pure_pickup` 闭式** | 一次捡全部（`pure_pickup.rs`），f54 `adopted` | **天然兼容**：全拾取 ⇒ `keys_out` 逐色单调最大 ⇒ 任何 `need` 都可满足（只要本层真有该色钥匙） | **不改**（`key_floor` 传入但不消费） | 适用：无怪可打/无门/纯拾取层。不当之处：可能多捡（多步）；因为它不看 need，**不能**用它来证明"钥匙够了"以外的任何最优性 |
| **`exact_pickup` 精确 DP** | 每出口 1 个终态（hp max → steps min），`MAX_PICKUPS=20`/`MAX_STATES=6_000_000`，f3 实测 4,741,000 态 / 3.08s | **必须改**：终态比较器换成 `cmp_exit_with_floor`；正向支配（`pm1⊇pm2 ∧ steps1≤steps2`）**保持健全**（超集⇒钥匙≥且步数≤），不动状态空间 | **改 1 处比较器 + 加 2 个诊断字段** | 若撞 `MAX_STATES` ⇒ **fail-closed** 落通用搜索，`ExactStat.state_cap` 与 `exact_give_up_reason` 必须具名；此时 `key_shortfall` 照旧落盘（不许把「预算用完」写成「不存在带钥匙的解」）|
| **通用 `Search`（BFS）** | 逐终态 push `Goal`（`solve_local.rs:2907`），同出口多候选 | **天然兼容**：`cmp_exit` 已有真实选择空间 ⇒ 换成 `cmp_exit_with_floor` 即生效 | **改 1 处调用** | 受 `SOLVER_LAYER_LIMIT`（默认 200,000，`layer_node_limit`）约束：截断必须 `truncated=true` 且 `ok=false`，不许把截断解当最优 |

**无法表达钥匙目标的路径的处置**：`pure_pickup` 不需要（全拾取已最大）；`exact_pickup` 若因 `key_floor` 的某色在本层根本不可拾取（`need[c] > 0` 且本层 `pickables` 无该色）⇒ 仍传 floor（叶子里自然没有满足者，退化为原比较），但**必须有具名标记** `key_floor_unreachable_in_layer`，让链路知道「本层补不了，得回拾」——这正是 §2.4 repair 的触发输入。

---

## 4. 失败语义

### 4.1 不变式（一条都不许动）

1. `ok = reached_goal && !truncated`（`solve_local.rs` 既有契约，S5 复核 F2 已确认 ok/truncated 不正交）。
2. `truncated` 必须由节点/状态预算触界置位，并落 `(floor, nodes)` 实数。
3. 任何「没跑完/没找到」都必须有 reason tag，禁止落成「看起来像结论」的空结果。

### 4.2 新增的具名诊断（只加字段，不改判定）

| 字段 | 何时非空 | 语义 | 谁消费 |
|---|---|---|---|
| `key_shortfall{need,have,missing}` | 出层 `keys_out` 不满足 `key_floor`（或 `have` 缺色） | **缺口提示**，`ok` 不变 | JSON 审计、repair |
| `credit_audit{credited,delivered}` | 该层有 `const` exports 且外层给过账 | 外层承诺 vs 内层交付 | JSON 审计、repair |
| `key_floor_unreachable_in_layer` | `need[c]>0` 但本层无该色拾取物 | 「本层补不了」 | repair 触发 |
| `exact_give_up_reason`（已有 `exact_pickup_stat`） | DP 未采纳 | `not_applicable/state_cap/replay_mismatch` | 判据区分「模型缺陷」与「预算不够」 |

**禁止的三件事**：
- 把「找不到满足 floor 的终态」写成 `ok=false`（违反 m04638）；
- 把「DP 撞 `MAX_STATES`」写成「无解」（违反 §4.1③）；
- 把 `credit_audit` 的差额当成失败（它只是残差）。

### 4.3 外层如何据此生成回拾

见 §2.4。要点：**回拾的触发条件是具名残差，不是 `ok=false`**；触发后仍走既有层任务机制（`toward` + `key_floor`），不新增搜索维度；repair 本身有预算（最多 N 次插入，N 初值 3，撞界 ⇒ `blocked.reason="repair_budget"` 而不是静默放弃）。

---

## 5. 切片与判据（每片一个可独立验证的 commit）

> 总原则：**`key_floor=None` ⇒ 逐字节不变**。这条既是回归闸门，也是每片的「默认分支」。
> 判据的红/绿以「变异」为准：故意打坏实现的一处，判据必须变红；否则判据是装饰。

### S6a — 单层软目标的深度落点（fixture 级，不碰链路）

改动：`objective::cmp_exit_with_floor`（新）+ `cmp_exit` 改薄包装 + `exact_pickup` 终态比较器 + 通用 `Search` 调用点 + `try_exact_dp`/`solve_layer_with`/`solve_layer_shops`/`solve_local_exit_toward_shops` 各 +1 参 + `LAYER_MEMO` 键 +1 元 + `ExactStat` +2 字段。

| 判据名 | 断言什么 | 怎么打红（变异） | 期望数值 |
|---|---|---|---|
| `s6a_exact_dp_key_floor_f3` | f3 fixture、`toward=4`、`key_floor=(2,0,0)`：`ok`、`adopted`、`!truncated`、`keys_out.yellow ≥ 2`、`keys_out.blue ≥ 1`、`hp == 4027`、`gold ≥ 1256`、`steps ∈ [81, 88]` | M1：把终态比较器的 floor 维度删掉 ⇒ `keys_out` 回到 `[0,1,0]`；M2：把 floor 维度排到 hp 之前 ⇒ `hp` 掉到 <4027；M3：把色序 y/b 换位 ⇒ `keys_out.yellow` 不足 | 期望 `keys_out=[2,1,0]`（或 yellow≥2 的等价态），`steps=83±2` |
| `s6a_exact_dp_none_unchanged` | 同 fixture 但 `key_floor=None` ⇒ 逐字段等于今天（4027/81/14、`doors=5`、`kills=7`、`exit=(11,11)`） | M4：默认分支误把 floor 当 0 处理 ⇒ 红 | 与 `tests/exact_pickup.rs:314` 现有断言逐字相同 |
| `s6a_cmp_none_equals_cmp_exit` | 全等性：对 `ExitScore` 网格采样（hp∈{4000,4027}、gold∈{0,100,500}、steps∈{80,81,88}、keys∈{0,2}³ 抽 200 组）断言 `cmp_exit_with_floor(a,b,None) == cmp_exit(a,b)` | M5：None 分支漏掉 steps ⇒ 红 | 200/200 相等 |
| `s6a_keys_never_outrank_hp_or_gold` | 保留既有两条钉死断言（`tests/objective.rs:165` 的 A/B 段逐字不动）+ 新增：同 hp 同 gold 时 keys 多者赢 | M6：把 keys 提到 gold 前 ⇒ B 段红 | A/B 段 `Less`；新段 keys 多者 `Less` |
| `s6a_layer_memo_distinguishes_key_floor` | 同输入、floor 先 `(0,0,0)` 再 `(2,0,0)` ⇒ 第二次必须重算（`goal.keys` 不同），且 `memo_hit=false` | M7：键不加 floor 元 ⇒ 第二次命中旧解（`keys_out` 不变）⇒ 红 | 两次 `keys_out` 不同 |
| `s6a_state_budget_still_holds` | f3 该 fixture 的 `exact_stat.states < MAX_STATES` 且 `adopted`；若撞顶必须 `state_cap` 具名 | M8：把 `MAX_STATES` 调到 1 ⇒ 必须走 `state_cap` 而非静默 | states ≈ 4.9M（多捡 2 件 ⇒ 位图子集 ↑，允许上浮 ≤ 6M）|

**回归闸门**：`cargo test -p game-solver` 全绿 + 74 金样 sha 全同（`docs/findings/2026-11-23-s5-convergence.md` 的口径）+ `gates/cli.py check` PASS 数与改动前一致。

### S6b — 链路接线 + 交付审计（进程内，不跑真链）

改动：`outer_witness` 把条件式 `key_floor` 传进 `layer`（`:200`）；`record_floor` 增 `credit_audit`；`WitnessFloor` 增字段；`path_to_tasks` 不动。

| 判据名 | 断言什么 | 打红 | 期望 |
|---|---|---|---|
| `s6b_witness_passes_demand_forward` | 用现有 f3 fixture 造一个 2 层链（f3→f4）：layer 桩必须收到 `key_floor=Some([2,0,0])`；且**第 1 次** f3 访问（toward=f2 下行）收到 `None` | M1：无条件传 demand ⇒ 第 1 次访问也带 floor ⇒ 红；M2：传 `Some([0,0,0])` ⇒ 红 | 两次调用分别为 `None` / `Some([2,0,0])` |
| `s6b_f3_delivers_two_yellow` | 用真 f3 引擎跑该链：f3 任务 `keys_out.yellow ≥ 2`、`key_shortfall` 为空、`hp == 4027` | M3：把 `key_floor` 传 `None`（回到今天）⇒ `keys_out=[0,1,0]` + `key_shortfall` 非空 ⇒ 红 | 与 S6a 首判据一致 |
| `s6b_credit_audit_reports_residual` | f3 第 2 次访问：`credited_yellow=8, delivered_yellow=5, missing=2`（8 地上、捡 5 带走 2 后剩 1） | M4：审计恒空 ⇒ 红 | 三数具名 |
| `s6b_entry_budget_not_consumed` | 把 `entry_budget` 全置 0，候选集合与 `blocked` 不变 | M5：误用 ⇒ 红 | 完全相同 |
| `s6b_out_fields_from_inner` | 桩里让 `got.out_hp` ≠ outer 估计 ⇒ 落盘取 `got` | M6：改读估计 ⇒ 红 | `out_hp == got.out_hp` |

### S6c — 真链一次（唯一一次 `--outer-seg`，8–13min）

| 判据名 | 断言什么 | 打红 | 期望 |
|---|---|---|---|
| `s6c_cross_f4` | `blocked.floor != 4`；`floors_solved ≥ 13`；witness 层序含 5..10；末任务 `Ending|BossKill` | 回退 S6a ⇒ 回到 today 的 `no_exit_toward@f4` | `floors_solved ≥ 17`（11+6） |
| `s6c_f4_entry_keys` | f4 任务的 `keys_in.yellow ≥ 2`（要落盘 `keys_in`；today 只有 `keys_out`） | 不落盘 ⇒ 判据无法执行 ⇒ 红 | `[2,1,0]` 或 `[2,0,0]` |
| `s6c_replay_closed` | JSON 的 `replay != null` 且 `floors` 非空；`prefix_replay.status=ok` 或具名（S5 复核 F4/F7 的两个红项在此闭环） | 不落盘 ⇒ 红 | 与 S5 复核要求一致 |

**注意**：S6c 若在 f5..f10 处再断且 `reason=key_shortfall`，**不得**就地抬 `SOLVER_LAYER_LIMIT` 或反复重跑——先按 §6.2 分类，再决定是否做 S6d。

### S6d — 链路级回拾（**条件**：仅当 S6c 在更深层出现 `key_shortfall` 才做）

| 判据名 | 断言什么 | 打红 | 期望 |
|---|---|---|---|
| `s6d_repair_inserts_revisit` | 人为把 f3 交付改成 1 把（注入），链路必须插入一次 f3 回访且 `key_floor=(1,0,0)`，之后 f4 通过 | 关掉 repair ⇒ `blocked.reason="key_shortfall"` ⇒ 红 | 插入 1 次、总步数增量可报 |
| `s6d_repair_budget_named` | 插入 3 次仍不够 ⇒ `blocked.reason="repair_budget"`（具名，不静默） | 无限插入 ⇒ 超时/死循环 ⇒ 红 | 具名 stop |

### S6e — 验收（同 S6c 那一次跑的数据，不额外跑）

见 §6。

---

## 6. 端到端验收

### 6.1 硬指标（一条，不许模糊）

`segments[0]`（`seg1_f51-f54_f1-f10`，`outer_chain.rs:50`，锚 boss75@f10 见 `outer_chain.rs:6`）：

- `blocked == null` 或 `blocked.floor > 4`；**必须**越过 f4；
- witness 层序包含 `5,6,7,8,9,10`；`floors_solved ≥ 17`；
- 末任务判定为 `Ending` 或 `BossKill`（f10 boss75 锚）；
- `replay != null` 且 `floors` 非空；`prefix_replay.status == "ok"`（或具名失败，附 token 定位）。

### 6.2 「模型缺陷」vs「搜索没跑够」的判别（禁止抬 cap 掩盖）

判据三条，按优先级判定：

1. **看 `blocked.reason` 与需求的关系**：若 `reason ∈ {no_exit_toward, key_shortfall}` 且「该层入口实际携带的钥匙」<「`next_demand` 对该层的需求」，且该层入口钥匙已是链路能给的最大值 ⇒ **模型缺陷**（缺的是钥匙来源，不是算力）。
2. **看预算余量**：`truncated=false` 且 `pops < cap` 且 `frontier_alive > 0` ⇒ **不是算力**；此时若仍无候选，必须打印「在 f4 处被 demand 拒掉的候选数」——若该数 = 全部候选，则缺陷在「没人给得起钥匙」，不在搜索。
3. **看截断**：`truncated=true` 或某层 `nodes == SOLVER_LAYER_LIMIT` ⇒ 才是**搜索没跑够**，此情形才允许讨论 cap，且必须在报告里同时给出「cap 提高后 `frontier_alive`/`floors_solved` 的对照」——**单跑一次高 cap 不算证据**（AGENTS：禁调阈值掩盖）。

**反例守卫**：若 S6c 的 `blocked.reason` 从 `no_exit_toward` 变成「无候选」而 `floors_solved` 下降 ⇒ 这是**退化**（方案 B 单独的症状），必须回滚而不是解释。

### 6.3 为什么「测试全绿」不等于正确

S6a/S6b 的判据全部是 fixture 级，只能证明「比较器与接线按契约工作」，**不能**证明 f4 真的过得去（f4 的可行性由真地图决定，而真地图只被 `keydemand` 与真引擎读过）。所以：**S6c 那一次真链跑是唯一能判定方案成败的证据**，其余判据的作用是「让它失败时能立刻归因」。

---

## 7. 风险清单

| # | 风险 | 触发条件 | 早期信号 | 缓解 |
|---|---|---|---|---|
| R1 | **DP 状态预算撞顶**（多捡 2 件 ⇒ 位图子集增加，f3 已 4.74M/6M） | `MAX_STATES` 或 `MAX_PICKUPS(20)` 触界 | `exact_stat.state_cap`；f3 单层耗时 >10s | fail-closed → 通用搜索 + `state_cap` 具名；不许静默；必要时按色剪枝（只保留 `min(count,need)` 桶），但剪枝必须证明支配健全（否则回落到 Pareto 出口集） |
| R2 | **memo 串味**：`key_floor` 不进 `LAYER_MEMO` 键 ⇒ 回放无 floor 的旧解 | 同层先无 floor 后有 floor | 第二次调用 `memo_hit=true` 且 `keys_out` 与第一次相同 | 键 10→11 元；判据 `s6a_layer_memo_distinguishes_key_floor` 直接打红 |
| R3 | **只改 `cmp_exit` 的假修**（lead 字面方案 A） | 只动 `objective.rs` | f3 `keys_out` 恒 `[0,1,0]` | S6a 首判据直接断言 `keys_out`，不看"代码改了没有" |
| R4 | **牺牲 hp/步数爆炸** | floor 维排在 hp/gold 之前，或步数增量为正无穷 | `hp < 4027`；`steps > 90` | 位次钉死在 gold 之后；判据把 `hp/gold` 相等作为硬断言 |
| R5 | **外层账本仍然是假**（atk730 vs 12）⇒ 候选选择基于假属性 | `budget_drifts` 的 atk/def/gold/hp 非空 | 每次跑都有一堆 drift | B′ 审计把它变具名（不静默）；**本轮不改**搜索口径（代价见 §1.2②），若要改按 §1.2 的 (b) 单独立片 |
| R6 | **下一层需求又超**（一步前瞻不够；f46=(3,0,0) 是已知另一处） | f5..f10 某层 `key_shortfall` 复现 | S6c 的 `blocked.reason=key_shortfall` | S6d repair（链路回拾）；先具名再修 |
| R7 | **repair 抖动**（同层反复回访、钥匙不增） | `key_floor_unreachable_in_layer` + 回访数↑ | 同层访问 >3 次、`steps` 单调涨而 `keys` 不变 | repair 预算 N=3 + `blocked.reason="repair_budget"` 具名停止 |
| R8 | **过拾取**（`next_demand` 逐色取最大，需求偏保守） | 出层 `keys_out` > `need` 且这些钥匙全程没用上 | 步数涨而下游不消耗 | 需求只对「这次访问的下一层」取；出口余量 ≤1 把；把过拾取写进 JSON 供审计，不静默 |
| R9 | **`pure_pickup` 被误当"钥匙够"的证据** | 某层走纯拾取路径就宣称可跨 | 该层 `keys_out` 恰好等于 need 但下游仍堵 | 纯拾取路径的结论只能用于「本层钥匙可全拾」，不用于「跨层够用」 |

---

## 8. 工作量与顺序（按"最早暴露最贵风险"排，不按好做排）

| 顺序 | 切片 | 工作量 | 为什么在这个位置 |
|---|---|---|---|
| 1 | **S6a** | 0.5–1 天 | 它同时回答两个最贵的问题：**(a)** 精确 DP 在 6M 预算内能否产出「带 2 把钥匙且 hp 不降」的终态；**(b)** 新增的 floor 维是否真的只在 steps 上花钱。若 (a) 失败（撞顶），整个方案的形状要改（换 Pareto 出口集或走 §1.2 的 rollout），**这个风险必须在动链路之前暴露**——否则 S6b/S6c 都建立在一个跑不出来的内层上 |
| 2 | **S6b** | 0.5 天 | 接线 + 审计；依赖 S6a 的比较器。它的失败模式廉价（进程内 fixture），且能把「双记账」变成可 grep 的具名残差 |
| 3 | **S6c**（唯一一次真链） | 0.5 天 + 13min | 端到端唯一裁判。必须在 S6a/S6b 全绿后才跑，因为只有一次预算；跑之前先确认 `keys_in` 落盘（否则 f4 段无法归因） |
| 4 | **S6d**（条件） | 1–2 天 | 只有当 S6c 在 f5..f10 处报 `key_shortfall` 才做；否则是给未来的保险，不提前花 |
| 5 | **S6e** | 0.5 天 | 复用 S6c 的 JSON 出验收报告（含 replay 闭环） |

**依赖**：S6a → S6b → S6c；S6d 依赖 S6c 的结论；S6e 依赖 S6c 数据。
**不许并行**：S6a 与 S6b 改的是同一批签名（`solve_layer_with`/`try_exact_dp`），并行必然冲突。

---

## 附录 A：如果只允许做一件事

**只做 S6a 的首判据：`cargo test -p game-solver --test exact_pickup s6a_exact_dp_key_floor_f3 -- --nocapture`**（约 3–5s，release 更快）。

它断言：f3 fixture、`toward=4`、`key_floor=(2,0,0)` ⇒ `ok`、`adopted`、`!truncated`、`keys_out.yellow ≥ 2`、`hp == 4027`、`gold ≥ 1256`、`steps ∈ [81,88]`、`exact_stat.states < MAX_STATES`。

**为什么是它**：整份计划能不能成立，只取决于一个未知量——「在 6M 状态预算内，精确 DP 能否给出一个多带 2 把黄钥、且 hp 一点不掉的终态」。这一条判据同时覆盖三个最贵的风险：DP 预算（R1）、钥匙是否只能吃步数（R4）、以及「改 `cmp_exit` 没用、必须改 DP 终态」这个结构判断（R3）。它绿 ⇒ 方案 C 的形状成立，剩下的都是接线与验收；它红（撞顶或 hp 掉）⇒ 不必写后面的切片，直接改方案形状。
