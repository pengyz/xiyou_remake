# S3 跨层回拾 + 战斗成本 LB + 三项遗留验收（2026-11-22）

状态：**已证实**（判据/变异/gates 均有本地 commit 与输出记录；A=9706644，B=60ecc80，C=本 commit）。
上游文档：[提案 §3.5/§5.7](../superpowers/proposals/2026-11-20-hierarchical-solver-proposal.md)、
[S2 findings 边界 7/8/9](2026-11-20-outer-skeleton-s2.md)、[S2 复核](reviews/2026-11-20-outer-skeleton-s2-fix-review.md)、
[keydemand](2026-11-20-key-demand.md)。

## A 片：跨层回拾（用户裁定 1/2 落地）

- 外层块图从「段内链」扩为**全可达层块图**：`OuterGraph::build_all(doc)` 55 层 383 块
  （判据 `s3_build_all_covers_full_block_graph`，include_str 真 doc 钉住）。
- `RunSpec.masked_exits`：断链注入开关（build 时跳过指定 (floor,cell) 出口边），具名回显。
- 回头边允许：trans 展开不限方向；目标层序早于当前层 ⇒ `OuterResult.reentry_edges += 1`
  （零奖励零惩罚定价——回拾路径只付步数，无任何收割特判，`s3_no_hardcoded_harvest` grep 判据）。
- keys/held 跨层携带本就是 OuterState 字段，本片判据显式钉死：
  `s3_keys_carry_across_floors`——f51 拾黄钥在 f10 门扣减（steps=3, yellow=0）；
  surplus 键回拾路径 steps=5, yellow=1（门需求按次收取的既有口径下精确定价）。
- **断链判据** `s3_reentry_when_chain_breaks`：mask f10 前向出口 ⇒ frontier 空、
  reentry≥1、不截断、搜索自行收敛；不 mask ⇒ 锚可达。

变异对照（全先红后还原）：

| 变异 | 红的判据 |
|---|---|
| M-A1 禁回头边（floors 序 forward only） | s3_reentry_when_chain_breaks |
| M-A2 换层清钥匙 | s3_keys_carry_across_floors |
| M-A3 mask 短路（false&&） | s3_reentry_when_chain_breaks（frontier 空断言） |

**【2026-11-23 更正】** 复核发现本片 findings 的「防硬编码」判据强度需如实标注：
`s3_no_hardcoded_harvest` 只 grep 3 个子串（reentrygain/harvest_/reentrybonus），
换名硬编码可绕过——该判据是**防回归哨兵，非完备证明**（复核 F6，按指令降级为黄）。

## B 片：战斗成本 LB 入边权（用户裁定 3/4 落地）

- 入边权 = 1 步 + **max(counter_loss)**（块内**当前可杀**怪；`game_core::rules::sensitivity`
  敏感表，armor_gate=true 口径，solver 零重算）。收益 = Σ `combat::kill_reward`（精确值，
  可杀才结算）。`--combat-cost` 默认**关** ⇒ 与 S2 逐字节一致（cmp 基线全等）。
- 怪物来源：`solve_global_input(1)` LevelSpec 实体格（ty 41..=75）按 cells 注入块表
  `monsters` 字段（v2 可选，向后兼容）；夹具测试用块表直写。
- 可采纳性（模块 doc + 此处双记）：max(counter_loss) 是逐怪损血之和的下界 ⇒ 保守可采纳；
  打不动按 0 计（此刻不可行≠永远不可行，涨攻后清块合法，计 0 才不破下界）。
  **hp 坐标 = 累计 LB 代价，可为负，禁作硬可行性剪枝**——否则真实路线被
  「按此刻属性收的 LB」误杀（实测：真数据 + hp≤0 剪枝 ⇒ 搜索 415 pops 即死；
  去掉剪枝 ⇒ 1.62M pops 完整收敛）。负 hp 前沿条目是「LB 已超当前血量预算」的诊断信号。

变异对照（全先红后还原）：

| 变异 | 红的判据 |
|---|---|
| M-B1 删损血项（hp/cost 不动） | lb_into_edge_weight + worst_counter_loss_of_block（2 红） |
| M-B2 删击杀收益 | lb_into_edge_weight（gold 断言） |
| M-B3 max 改首个可杀 | worst_counter_loss_of_block |
| M-B4' 打不动也计收益 | unkillable_counts_zero_lb（gold=0 断言） |
| （等价变异）打不动计入 max | counter_loss=-1 ⇒ 行为等价（同 S2 M-B 结论） |

真数据第 1 段实测（--combat-cost，--max-pops 5000000，release 58s）：

| 指标 | combat off | combat on |
|---|---|---|
| pops | 643,024 | 1,623,198 |
| 前沿 | 1,095 | 1,272 |
| killable | 53 | 262 |
| 前沿 top1 steps | 52 | 7,373（含 LB 损血） |
| 最短可杀 steps | 64 | 9,178（LB 代价坐标，非真损血可行性验证——复核 F2） |
| top1 hp | 4,300 | −2,028（LB 超预算诊断位） |

排序**显著**改变（判据：LB 边权变异 ⇒ 前沿排序变化，M-B1 红）。

### 【2026-11-23 S4 修单】带价钥匙商店购买边 + SwapYellowKey（复核 F4 红）

复核证伪了本文件旧边界 3（见下）——第 1 段**确有**带价钥匙商店，外层现已建显式购买行动：

- **AddKeys 购买边**（真锚 f6b1 蓝¤50 / f7b4 黄×5¤50 / f12b0 红¤800 / f12b6 黄¤1000 /
  f15b7 蓝¤200 / f31b2 蓝+黄¤1000 / f38b3 黄×3¤200 / f39b3 蓝×3¤2000）：
  gold≥price ⇒ 扣价、入键、计 1 步；金不足无此臂（判据
  `s4_addkeys_purchase_opens_gate` / `s4_purchase_respects_gold_price`）。
- **SwapYellowKey**（唯一真锚 f28b0=(1,8)，语义经 game-data shop_goods_defs.rs 钉死）：
  **黄-1 → ¤+100**——是「卖」不是「买」（键→金的流动性逆向边，非钥匙来源；
  钥匙来源=拾取+AddKeys 购买）。判据 `s4_swap_yellow_key_sell_action`。
- 变异对照（全先红后还原）：

| 变异 | 红的判据 |
|---|---|
| M-F4a 删 AddKeys 分支 | s4_addkeys_purchase_opens_gate |
| M-F4b 删 SwapYellowKey 分支 | s4_swap_yellow_key_sell_action |
| M-F4c 删 gold≥price 门 | s4_purchase_respects_gold_price |

- 投影守卫判据 `s4_real_doc_priced_key_shop_anchors`（初始即绿，非 TDD 红）
  把真实锚清单钉进测试面，旧「无锚点」断言永久封死。
- 真数据 combat-on 数字随购买边更新（--max-pops 5,000,000，release 8m13s）：

| 指标 | S3-B（无购买边） | S4（购买边落地） |
|---|---|---|
| pops | 1,623,198 | 4,553,792 |
| 前沿 | 1,272 | 3,630 |
| killable | 262 | 985 |
| top1 steps / hp / gold | 7,373 / −2,028 / 0 | 7,073 / −1,792 / 63 |

  top1 gold=63>0 证明商店臂在真数据上实际开火（经济链活了）；步数/hp 均改善。
  注：killable 985 条全部 fatal——combat-on 的 hp 坐标是累计 LB 代价（可为负），
  `fatal = counter_loss ≥ hp` 在负 hp 诊断位上恒真；存活可行性须由内层
  solve_local 按 S4 验收口径（见下）判定，外层数字是搜索面/排序诊断量。

**【S4 验收口径（写死）】此后凡引用外层前沿/killable 数字做验收（含 S4 及以后），
必须在 `--combat-cost on` 口径下进行；off 口径只用于 S2 回归对照（逐字节不变）。**

## C 片：三项遗留验收（提案 §5.7）

### C1 LB 下界表落盘

`data/tables/outer_lb_seg1.json`（CLI `--lb-out` 产出）。口径（文件内注明）：
sensitivity_table @ 段起点 atk=10/def=10/held 空，armor_gate=true，difficulty×1；
counter_loss=-1=打不动（LB 计 0）；lb_reward=kill_reward 精确值。
内容：35 敌全表 + 60 个含怪块的块级 LB（块级合计 lb_loss=18,845、lb_reward=289）。

### C2 H5 前沿宽度实验（第 1 段，combat off 前沿 1095 条）

- 维数构成（distinct 值数）：atk 8 / def 7 / hp 24 / yellow 14 / red 1 / blue 5 /
  steps 49；键组元组 62 种（最常见 (3,0,2)=46 条）。
- 宽度主轴是 **steps×keys**：steps 直方 52..~（52:1, 53:5, 54:6, 55:13, 56:13, 57:27,
  58:29, 59:33, 60:38, 61:42, 62:49 …），path 长度分布峰值在 58-63（~106 条）。
- killable=53 条分布在 steps 64..97（71:5, 72:5, 80:5 为峰）。
- combat on 后（1272 条）hp 维从 24 → **1058** 个 distinct 值：战斗成本把 hp 从
  「拾取决定的粗粒度坐标」变成「路径决定的细粒度坐标」，前沿宽度主轴移到 hp×steps。
  这印证边界 8 的判断：不结算战斗成本时，外层前沿严重欠分辨。

### C3 StatPct 饱和阈值闭式

`crates/game-core/src/rules/shop.rs`（game-core 单源，solver 复利公式副本已删）：
- 单步：`stat_pct_compound_step(atk,pct) = atk + floor(atk*pct/100)`（i32 饱和）；
- **零增益阈值**：`stat_pct_zero_gain_threshold(pct) = ceil(100/pct) - 1`，pct=3 ⇒ **33**
  （atk ≤ 33 每买 +0 属性仍付 1 步 = 纯浪费；判据 `zero_gain_threshold_closed_form` 钉住）；
- H6 已拍板方案 a（1 步成本）不变。中间溢出边界（saturating_mul 先饱和再 /100）
  与 S2 solver 侧继承语义逐位一致（测试钉 1,095,216,659）。

TDD：红=E0432 unresolved imports（先测试后实现）。

## f51..f54 链路不变判据

三片各自提交前均复跑 `outer --out … --max-pops 700000` 并 `cmp` S2 基线
（pops=643024，前沿 1095）：**逐字节全等 PASS**。S3 改的是外层搜索结构
（回拾/LB/公式宿主），默认路径行为零漂移。

## 语义边界与遗留假设

1. 打不动怪按 0 计 LB：保可采纳，代价是前沿含「负 hp 诊断条目」（非可行解声明）。
2. 块粒度怪物归属： monsters 按 cells 全量归块，首入块即收块内最坏可杀怪 LB——
   多格块会高估（LB 意义下仍可采纳）。
3. 【2026-11-23 更正：本条已失效——原主张「第 1 段真实块表无带价钥匙商店」被复核证伪
   （实查 block-interfaces.json：f6b1=¤50蓝×1、f7b4=¤50黄×5、f12b0=¤800红×1 等 8 锚在），
   带价 AddKeys 购买行动与 SwapYellowKey 卖边（f28b0）已于 a87a730 建入外层，守卫判据
   s4_real_doc_priced_key_shop_anchors 已补齐 9 锚全断言。以下为历史原文】
   商店带价购买（AddKeys price>0）未建为搜索行动：第 1 段真实块表无带价钥匙商店
   （f52b8=GiveGold(1000) 免费），规则缺真实锚点，留待真锚出现再接经济链购买边。
4. 【2026-11-23 更正：随第 3 条失效——带价商店锚点已落地，gold 边权口径见 a87a730】
   gold 的商店边际转化率：gold 已入状态与 pareto 维（B 片），「金→钥匙当量」折算
   需要带价商店锚点（同 3），当前只在 findings 记口径，不进边权。
