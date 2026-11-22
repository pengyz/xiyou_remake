# S2 外层骨架 对抗性复核报告

**日期**：2026-11-22
**复核者**：独立对抗性复核 agent（不参与实现）
**被告**：本地 commit `15dd40c`（敏感表）+ `b147e62`（外层骨架+CLI+findings），父 `9b70979`
**纪律**：只写本文件；未 commit、未改仓库代码；变异实验在 `/tmp/audit-copy`（git archive 副本 + assets/data 软链）

## 计数：绿 4 / 黄 4 / 红 1

---

## F1 敏感表正确性 —— 🟢 绿

**方法**：完全独立重算。用 `/tmp/audit23`（独立 crate）调 Rust 侧 dump 全表（35 怪全量 × 倍率 {1,30,60} × 11 组 (atk,def,hold23,hold24) 共 **1155 行**），与 Python 独立重实现的 Java 公式（`predictBattleHpLoss` ceil-hits + counter=(eatk−def)×(hits−1) + armor_gate 口径）逐行比对：**0 mismatch**。

- 覆盖普通怪（41/42/…）、trait 怪 49/53/74(bit0)、67/69(bit2→bit1)、boss75；
- 阈值语义逐点核对：`one_hit_atk = hp+edef`（hits=1 ⇔ eff−edef ≥ hp，无 off-by-one）、`zero_loss_def = eatk`（边界 `def == eatk` ⇒ `>` 不成立 ⇒ 0，正确）、`can_kill ⇔ eff > edef`、打不动 `loss = −1`；
- 23/24 持有态两维：错配不翻倍（else-if 语义）实测正确；倍率层用缩放表直算，阶跃点随缩放平移（测试 `multiplier_shifts_breakpoints_linearly` 独立验证通过）；
- 零复制属实：表内公式全部委托 `combat.rs:93-131`，无第二份常量。

**复现**：`cargo test -p game-core --lib rules::sensitivity`（6 passed）；比对脚本 `/tmp/audit23/check.py`。
**Confidence: 96**

## F2 killable 谓词 —— 🔴 红（被告漏了「损血 < hp」约束）

`outer.rs::anchor_killable`（crates/game-solver/src/outer.rs:998-1015）只判 `r.can_kill`（eff > edef），**完全不检查 `counter_loss < state.hp`**。用户口径明确要求「可杀判定必须同时满足损血 < 当前 hp」。

**实锤**（真数据实跑 `solver outer --max-pops 800000`，输出 `/tmp/audit23/outer.json`）：

- 前沿 1095 条、killable=true 53 条——与 findings §6 一致（数字如实）；
- **其中 8 条实际是致死路径**：最短可杀 `steps=64 atk=16 def=12 hp=5200`，对 boss75（hp100/atk65/def15）撞面损血 = (65−12)×(100−1) = **5247 > 5200 ⇒ 必死**。findings §6 把这条当「最短可杀」输出，是**污染段锚与前沿的直接证据**（另 7 条：steps 66/69/70… 同病）；
- boss75 hp=100（fixture id75=i34），atk=16 每击 1 伤、100 回合反击——任务书怀疑方向正确。

**复现**：`./target/debug/solver outer --max-pops 800000 --out /tmp/x.json`，对 killable 条目用公式独立重算损血。
**修法方向**：`anchor_killable` 改为 `row.can_kill && row.counter_loss >= 0 && row.counter_loss < state.hp`（敏感表已导出 counter_loss，零新公式）；findings §6 的「最短可杀」结论随之更正。
**Confidence: 99**

## F3 乐观门口径 —— 🟡 黄（如实标注，但全前沿被乐观门染色，下游不可当真值）

- 实测：**1095/1095 条前沿全部带 `optimistic_gate` 注记**（每条 30-40 处），即「锚可达 steps=52」整段结论都建立在 sealed4/sealed81/script_gate 免费通行之上；
- findings §5.1 确实如实标注「前沿是乐观下界」，每条边也带具名注记——**诚实性达标**；
- 反证检查：f4 的门是 7×door1 + 1×door3（真钥匙门，外层图按 cut_doors 计需求并强制 `state.yellow < e.yellow → skip`），f4 通过**不是**乐观门造成的误判；乐观门集中在 f51(12)/f52(5)/f53/f54/f10 的 sealed/script 门；
- 但 findings 未给出「收紧乐观门后锚是否仍可达」的敏感性说明——若 S3 把 52/64 步当真值消费，段锚规划会被污染。v1 只作诊断面尚可接受。

**Confidence: 90**

## F4 状态空间与用户口径的差距 —— 🟡 黄（有具名 Off，但三处缺口未声明）

1. **段内限定**：seg2/segfinal/branch 全部具名 `Off`（带 reason）——比静默好；但 findings **没有一处明确写「本片仍是段内局部细化、跨层回拾未建」**（用户 m04867 的核心缺口），也没把它列为 S3 验收条目。§5 的 6 条已知边界不含这条。
2. **战斗未建模**：外层状态从不结算打怪——打怪给的金币/属性/钥匙全为零，hp 只增不减。这使「52 步到锚」漏掉全部必经战斗成本，比乐观门更根本。§5 未声明此边界。
3. **提案例 vs 实作缺口**：提案 §7 S2 验收含 LB 下界表、H5 前沿宽度实验、§3.5 饱和阈值闭式导出+折叠测试——三样均未交付，findings 却写「全部判据已证实（标注处除外）」而未标注这些豁免。属超claim。

**Confidence: 92**

## F5 f13 矛盾探测 —— 🟢 绿（被告数字对，任务书 31 错）

**独立取证**：用 `/tmp/audit23` 直接对 `assets/raw/sprite{i}` 原始字节跑 `game_data::sprite::SpriteTable::parse`（零余字节严格解析），逐层数 ty10：

- **f13 = 3 处**：(6,4)(6,5)(6,6) —— 与被告一致；
- **f26 = 29 处**，其余层 0，**全塔 total = 32**。「31 堵」的说法错误（既不是 31 也不是 f26 29+5）；
- `probe-f13` 实跑复现：楼梯 (1,11)/(11,11) 可达、金箍棒 (6,3) 不经火环不可达 ⇒ 无矛盾，火环=宝物门；
- 引擎口径核对：`solve_local.rs:2116` `!(self.walkable(x,y) || stair_like)` 与 probe 的 `walk || stair_cells.contains` 一致，放行 (1,11) 这种「楼梯在不可走地形」的格属实。
- 与 2026-11-18 findings（f13 三昧真火只是一列 x=6,y=4..6）交叉印证。

**Confidence: 97**

## F6 Dijkstra 支配剪枝健全性 —— 🟡 黄（论证为健全，但依赖堆序的隐式前提）

- `dominated_seen`（outer.rs:1045-1078）**接收 steps 参数但从未使用**——被归档条目的支配比较不含步数维。健全性靠一个隐式前提兜住：所有转移成本 ≥1（邻块/换层/购买均 +1），二叉堆按 steps 弹出 ⇒ 归档条目 steps 必 ≤ 当前 steps，支配成立。**这是对的，但脆弱**：未来加任何 0 成本边（如免费脚本传送）即静默失效。建议把 steps 显式进支配元组或加注释断言。
- `picked` 完全相同才比、held 超集维保守不比（宁滥勿缺）——与文档声明一致；终局 `pareto()` 跨 picked 集比较，因拾取单调受益亦健全。
- hp 维在状态里且进支配（更多 hp 支配更少 hp）——但注意外层根本不结算损血（见 F4-2），hp 维当前恒为拾取和，支配语义形式健全、实质半空。
- 未找到「被剪但实际更优」的反例。

**Confidence: 85**

## F7 门禁与夹带 —— 🟢 绿

- `gates/cli.py check` 复跑：**PASS=13 FAIL=0 SKIP=1（visual，显式打印）TOLERATED=0**；ratchet total=1271 vs 基线 1259，+12 如实；
- 两笔 commit name-status 无越界：只触及 `crates/game-core/src/rules/{mod.rs,sensitivity.rs}`、`crates/game-solver/src/{lib.rs,outer.rs,bin/solver.rs}`、`docs/findings/2026-11-20-outer-skeleton-s2.md`；
- 变异复现（/tmp 副本）：**M1**（armor_gate true→false）⇒ 2/6 测试 FAIL（commit message 写 3/6，**实际 2**——小笔误）；**OM2**（购买 1 步→0 步）⇒ `stat_pct_purchase_costs_one_step_and_saturates_budget` FAIL，min_killable 判据抓住。两发都被抓，可复现；
- MAX_BUYS=256 与 H6 的关系：购买**确实计 1 步**（OM2 证明），帽是工程收敛手段而非成本替代；但 256 是拍脑袋值（提案 H6 本意是「步数成本自然封顶，不许拍脑袋帽」）——因免费复利状态空间确实不收敛，帽有正当性，且 seg1 内阈值远低于帽下 atk 上限，不咬合；帽已显式声明为 v1 边界（§5.2）。降级为备注不计黄。附带：§5.2 说「256 次 ≈ atk 10→~3900」算错了，10×1.03^256 ≈ 19300——数字笔误不影响结论。

**Confidence: 93**

---

## 两句话结论

1. **S2 骨架能不能进下一段开发**：能——工程骨架（具名报错/乐观门注记/变异门禁）纪律是过关的，F1/F5/F7 三个头号面实测干净；但必须先修 F2 红项并把 F4 的三处缺口（跨层回拾未建、战斗未建模、LB/H5/饱和项未交付）补进 findings 的已知边界与 S3 验收清单，否则下一段会在被污染的前沿上开工。
2. **F2 修法方向**：`anchor_killable` 增加「`counter_loss < state.hp`」判据（敏感表已导出该值，零新公式、零复制），同时把 hp 损耗作为锚到达结算写回状态（或至少在 frontier 输出 `fatal=true` 诊断列），findings §6「最短可杀 steps=64」随之更正为按存活口径重算。

## 附：复核产物（/tmp，不入库）

- `/tmp/audit23/`：独立 crate（sprite 逐字节解析 + 敏感表 dump + Python 对照脚本 `check.py`，1155 行 0 mismatch）
- `/tmp/audit-copy/`：变异副本（M1/OM2 复现）
- `/tmp/audit23/outer.json`：真数据 outer 实跑输出（1095 条前沿）
