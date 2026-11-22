# S3 跨层回拾 + 战斗成本 LB 对抗复核（2026-11-22）

复核者：独立对抗复核 agent。对象：`9706644`（S3-A）/ `60ecc80`（S3-B）/ `960a301`（S3-C），
父 `d6fd4d6`。只读复核：不 commit、不改代码；变异在 /tmp 副本（git archive + ln -s assets）。

## 结论计数：绿 4 / 黄 2 / 红 1

## F1 回拾边定价 —— 绿

判定：**回拾/换层边与层内邻块边走同一条 edge-cost 函数，无系统性低估。**
- 逐行核对 `crates/game-solver/src/outer.rs:833-845`（邻块边）与 `:861-886`（换层边）：
  两处完全同构——`combat_cost && !picked(to)` 时同样调 `block_combat_cost`，
  `cost += loss`、`ns.hp -= loss`、`ns.gold += reward`。回头边经带怪块照样计 ΣLB。
- findings A 片「回拾只付步数」是对 combat-off 口径的如实陈述（`s3_keys_carry_across_floors`
  5 步精确定价即在此口径下）；B 片落地后回头边已并入同一成本函数。
- Confidence: 95。

## F2 可采纳性与排序语义 —— 黄（低危）

判定：机制正确，一处标注不完整。
- `dominated_seen`（`outer.rs:941-951`）**包含 hp 维**（`*hp >= state.hp`），
  hp 可负的态不会因被当作可行而排前——支配比较正确。
- 「最短可杀 steps=9178」是 **LB 代价坐标最小**，非「真损血可行」证明
  （top1 hp=−2028 即 LB 超预算诊断条目）。findings 只在 top1 行标注「（含 LB 损血）」，
  9178 行未同样标注；模块 doc 与边界 1 有完整口径，但实测表内该行措辞
  「最短可杀」有歧义。修法：findings 该行补注「steps 含 LB 损血，非真可行保证」。
- Confidence: 90。

## F3 killable 53→262 成因 —— 绿

判定：**谓词未被改，数字差异来自搜索面变大。**
- `git show d6fd4d6:...outer.rs:857` vs `960a301:...outer.rs:994`：
  `killable = can_kill && counter_loss >= 0`、`fatal = killable && counter_loss >= hp`
  逐字节一致。
- 成因：combat on 后 gold/hp 进入状态与支配维（B 片），支配剪枝留更多互不支配的
  态到达锚并存活 Pareto——搜索面变大，非谓词放宽。
- 本机复跑（release，--combat-cost --max-pops 5000000，57s）精确复现：
  pops=1,623,198 / 前沿=1272 / killable=262 / top1 steps=7373 hp=−2028 /
  最短可杀 steps=9178。与 findings 表逐项全等。
- Confidence: 95。

## F4 「默认关」的永久化 + 商店购买锚点 —— 红

判定：**findings 边界 3 的核心依据是假的；经济链购买边缺失且被错误文档掩盖。**
- findings 边界 3 称「第 1 段真实块表无带价钥匙商店」。**实 disproven**：
  `docs/findings/block-interfaces.json` 第 1 段内有两处带价 AddKeys 商店：
  - f6b1：ty78 timepoint，goods `[[50, "AddKeys", {"blue":1}]]`（50 金→蓝钥×1）
  - f7b4：ty78 timepoint，goods `[[50, "AddKeys", {"yellow":5}]]`（50 金→黄钥×5）
  外层搜索（`outer.rs:887-911`）只处理 ty77/78 的 StatPct 购买，
  AddKeys/带价钥匙购买边完全未建——用户裁定「钥匙可商店购买」在外层未落地，
  且「无锚点留待真锚出现」的搁置理由与自家单一真相源直接矛盾。
- SwapYellowKey 动态补充：外层 OuterState 无对应行动（solve_local 内层已实现，
  `solve_local.rs:1790`；真实锚 src=1,idx=8 未进块表投影）。findings 未提 SwapYellowKey
  在外层的缺位。
- findings 未写明「S4 必须在 --combat-cost on 口径下验收」；combat on 与 off 的
  killable/前沿口径差异 5 倍，不下验收口径会被默认关的「绿」掩盖。
- 复现：`python3 -c` 遍历 block-interfaces.json seg floors 的 exports.goods
  （price>0 或 key 含 AddKeys）⇒ 上述两条。
- 修法方向：外层为带价 AddKeys 建显式购买行动（gold 扣价、键入态、buys 维
  或计步 1）；findings 边界 3 原地更正；S4 验收口径写死 combat-cost on。
- Confidence: 98（纯数据面断言，一条命令可复现）。

## F5 C 片正确性 —— 绿

- `stat_pct_zero_gain_threshold(3)`：`ceil(100/3)−1 = 34−1 = 33` ✓（shop.rs 闭式
  `(100+pct−1)/pct − 1` 手算核对，pct=50→1、100→0、1→99 全对）；
  饱和边界 `stat_pct_compound_step(i32::MAX/2, 100) = 1,095,216,659` 手算一致 ✓。
- `data/tables/outer_lb_seg1.json` Σlb_loss=18,845 独立重算 ✓；**全部 35 行敌敏感行**
  与 60 条块级行用独立 Python 复算（predict_battle_hp_loss 公式，enemybasehp/atk/def
  fixtures）逐一比对：**0 处不匹配**；块级 lb_loss=max(可杀 counter_loss)、
  lb_reward=Σ可杀赏金语义与实现一致（含 32 个多怪块，如 f53b0 七怪 lb_loss=3825）。
- solver 侧复利副本已删净：`grep stat_pct crates/game-solver/src/` 仅剩
  `outer.rs:906-907` 对 `game_core::rules::shop::stat_pct_compound_step` 的调用，
  无本地公式残留 ✓。
- Confidence: 95。

## F6 判据与变异 —— 黄（grep 判据降级）

变异重放（/tmp/s3review = `git archive 960a301`，CARGO_TARGET_DIR 隔离）：

| 变异 | 结果 |
|---|---|
| M-A1 禁回头边（trans forward-only） | `s3_reentry_when_chain_breaks` 红（另带 `s3_build_all_covers_full_block_graph` 红）✓ 真红 |
| M-A2 换层清钥匙 | `s3_keys_carry_across_floors` + 3 个 B 片判据红 ✓ 真红 |
| M-B1 删损血项（保留 reward） | `s3_combat_cost_lb_into_edge_weight` + `worst_counter_loss_of_block` 2 红 ✓ 与 findings 表逐条一致 |

`s3_no_hardcoded_harvest` 强度评估：仅对 outer.rs 查 3 个子串
（`reentrygain`/`harvest_`/`reentrybonus`）。任何换名硬编码（如 `reentry_edge_reward`、
`+ 100` 常量、把奖励写进 gold 结算行）均 trivially 绕过，且只看单文件。
**按指令降级为黄**：该判据只能防最粗的字面量回归，不构成「无硬编码收割」的证明。
- Confidence: 90。

## F7 门禁与夹带 —— 绿

- `gates/cli.py check` 本机复跑：**PASS=13 FAIL=0 SKIP=0**（含 rust-tests 全绿、
  test-ratchet 1283≥1259、state-baseline 金样逐字节）。
- 三笔 commit --stat 越界核查：9706644/60ecc80 只动 solver.rs+outer.rs；
  960a301 动 game-core/rules/{mod,shop}.rs + solver + data/tables/outer_lb_seg1.json +
  findings。无越界；shop.rs 落 game-core/rules/ 符合「公式 game-core 单源」裁定 ✓。
- f51..f54 基线逐字节比对**独立复跑**：`./target/release/solver outer --out
  /tmp/review_off.json --max-pops 700000` → `cmp /tmp/outer_seg1_baseline_pre_s3.json`
  ⇒ **BYTE-IDENTICAL**（pops=643024、前沿 1095、killable 53）✓。
- 微瑕：commit message 称「outer::tests 83 passed」，实际 outer:: = 15 tests、
  83 是 game-solver lib 全量（数字张冠李戴，非实质问题）。
- Confidence: 95。

## 总评（两句话）

1. **S3 可以进下一段**：核心机制（全图建图、断链自动回拾、跨层键携带、LB 边权、
   公式单源、默认路径零漂移）全部实测复现且变异全红，工程面扎实。
2. **F1/F2 均非红**；唯一红是 F4——带价 AddKeys 商店锚点真实存在（f6b1/f7b4）却被
   findings 边界 3 误判为「无锚点」而搁置购买边，修法方向是外层建显式购买行动 +
   findings 原地更正 + S4 验收口径写死 combat-cost on。
