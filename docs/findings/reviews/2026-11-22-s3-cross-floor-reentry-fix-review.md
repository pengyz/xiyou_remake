# S4 修单（a87a730）独立定向复核（2026-11-23）

复核者：独立定向复核 agent。对象：`a87a730`（父 `960a301`），只验上轮
[复核](2026-11-22-s3-cross-floor-reentry-review.md)（红 F4 / 黄 F2/F6）修单五点，不重开面。
只读复核：不 commit；变异在 /tmp 副本（`git archive a87a730` + `CARGO_TARGET_DIR=/tmp/s4target`）。

## 结论计数：绿 4 / 黄 1

## P1 AddKeys 购买行动 —— 绿

- **并列同路径**：逐行核对 `crates/game-solver/src/outer.rs:899-913`——StatPct/AddKeys/
  SwapYellowKey 三分支在 `state.buys >= MAX_BUYS` 同一闸下，AddKeys 臂同样
  `heap.push(HeapItem { steps: steps + 1, …, buys += 1 })`，与 StatPct 行动同支配/堆路径 ✓。
- **f6b1 独立重算**（¤50 蓝×1 判据 `s4_addkeys_purchase_opens_gate`）：杀怪 41 恰得 50 金
  → 买蓝扣金 50−50=**0** ✓ → door3 扣蓝 1−1=**0** ✓；步数拾取(1)+击杀(1+c41)+购买(1)+过门(1)
  +末块(1)=**4+c41** ✓（c41 由 sensitivity_table 现算，与 LB 边权同源）；且断言
  `frontier 全部 steps ≥ expect` 钉死「购买必须付 1 步」✓。金门独立判据
  `s4_purchase_respects_gold_price`（¤49 < ¤50 ⇒ frontier 空）✓。
- **变异 M-F4a**（/tmp 副本删 AddKeys 分支）：`s4_addkeys_purchase_opens_gate` **FAILED**
  （其余 s4 判据不受牵连）——真红 ✓。
- Confidence: 97。

## P2 SwapYellowKey 卖边 —— 绿

- **方向核对（A 级证据属实）**：`game-data/src/defs.rs:644` `SwapYellowKey { gold: i32 }`
  解析于 `defs.rs:731`；`data/tables/shop_goods.json` src=1 idx=8 行 price=0、effect
  `{gold:100}`、fail_hint「没有黄钥匙」（键数门——只对「卖」语义成立）、evidence
  「a.java:7394-7402 黄-1→¤+100」。`crates/game-data/tests/shop_goods_defs.rs:33`
  断言 `(1,8) 黄-1→¤+100` 钉死。**是卖不是买**；内层 `solve_local.rs:1790-1795` 同向
  （yellow−1, gold+gold），外层 `outer.rs:917-928` 分支一致（yellow<1 无臂、yellow−1、
  gold+=g）。方向无反转 ✓。
- **变异 M-F4b**（分支条件改 `__MUT_F4B__` 永假）：`s4_swap_yellow_key_sell_action`
  **FAILED**（gold=100 条目消失）——真红 ✓。
- Confidence: 97。

## P3 投影守卫 `s4_real_doc_priced_key_shop_anchors` —— 黄（守卫真实但覆盖 4/9 锚）

- **遍历真数据属实**：`include_str!("../../../docs/findings/block-interfaces.json")`
  编译期嵌入真实单一真相源，无夹具；遍历全部 floors/blocks/exports/goods。
  本人用独立 Python 遍历同一文件复算：带价 AddKeys 共 **8 锚**（f6b1/f7b4/f12b0/f12b6/
  f15b7/f31b2/f38b3/f39b3，价 50/50/800/1000/200/1000/200/2000）+ SwapYellowKey 1 锚
  （f28b0，¤100），与 findings 清单逐条全等 ✓。
- **但断言只钉 4/9**：测试仅 `contains` 断言 f6b1(6,50,蓝1)/f7b4(7,50,黄5)/f12b0(12,800,红1)
  + f28b0(28,{gold:100})；**f12b6/f15b7/f31b2/f38b3/f39b3 五锚无断言**，且按
  (floor, price, params) 匹配未钉 block id（如 f12b0 与 f12b6 同层仅靠价区分）。若真表
  这五锚漂移/消失，守卫仍绿——「真实锚清单钉进测试面」的说法只对 4 锚成立。AddKeys/
  SwapYellowKey 分支本身是泛型处理（任何带价 goods 均出臂），行动覆盖面无缺口；
  缺口只在守卫的断言清单。修法：补 5 条 contains + 两条总计数断言（addkeys.len()==8、
  swap.len()==1），并钉 block id。
- Confidence: 95（纯测试面断言核对，一条命令可复现）。

## P4 findings 更正 —— 黄（边界 3 未原地更正）

- 9178 行已补「LB 代价坐标，非真损血可行性验证——复核 F2」✓；
  grep 哨兵已加日期注「防回归哨兵，非完备证明（复核 F6）」✓；
  【S4 验收口径（写死）】combat-cost on 已成段写死 ✓；
  combat-on 数字表（pops 4,553,792 / 前沿 3,630 / top1 7073/−1792/63）**本人 5M pops
  combat-on 复跑逐项全等**（且 top1 路径确实经过 f6b1 商店块，gold=63>0 证臂开火）✓；
  killable=985 / fatal=985 本人复算 `sort|uniq -c` 确认 ✓。
- **缺口**：「语义边界与遗留假设」第 3 条**原文未动**——仍写着被证伪的
  「第 1 段真实块表无带价钥匙商店…留待真锚出现再接经济链购买边」，无原地日期注
  （AGENTS.md §3.6 要求「在原文原地标注日期更正」）。更正只在上方新增的
  【2026-11-23 S4 修单】段（「复核证伪了本文件旧边界 3（见下）」）；只读边界清单的读者
  在第 3 条处看不到任何更正标记。第 4 条「同 3…需要带价商店锚点」随之整体过时
  （锚点已存在、购买边已落地）。修法：第 3/4 条各加
  `> 【2026-11-23 更正】已证伪，见上文 S4 修单段` 并改写为现状。
- Confidence: 96。

## P5 门禁与回归 —— 绿

- `python3 gates/cli.py check` 亲自复跑：**PASS=13 FAIL=0 SKIP=0**（含 rust-tests 全绿、
  test-ratchet 1287≥1259、state-baseline 74 场景金样逐字节）✓。
- `cargo test -p game-solver --lib` 亲自复跑：**87 passed; 0 failed** ✓。
- 默认路径（combat off）逐字节：`git archive a87a730` 重建 release solver（/tmp/s4target，
  确保二进制对应修单 commit 而非陈旧产物）→ `outer --out /tmp/s4_off.json --max-pops
  700000` → `cmp /tmp/outer_seg1_baseline_pre_s3.json` ⇒ **BYTE-IDENTICAL** ✓。
- `git show a87a730 --stat`：只动 `crates/game-solver/src/outer.rs`（+206/−13）与
  `docs/findings/2026-11-22-s3-cross-floor-reentry.md`（+44），无越界 ✓。
- Confidence: 97。

## 附加语义点（如实记录，不判级）

combat-on 口径下本人复算：**killable=true 985 条，fatal=true 985 条——100% fatal**。
机理属实：hp 坐标是累计 LB 代价（可为负），`fatal = killable && counter_loss >= hp`
在 hp<0 的诊断条目上恒真；本轮前沿 3630 条无一 hp≥0（最高也是负的诊断位），故 fatal
列在 combat-on 下退化为常量 **true，零信息量**；killable 列仍保留「打得动」过滤信息。
findings 的注明（「killable 985 条全部 fatal…外层数字是搜索面/排序诊断量，存活可行性
由内层 solve_local 判定」）**诚实且机理正确**——足够；建议（非必须）后续在 combat-on
输出表直接不印 fatal 列或印「fatal=985/985」形式，避免读者误读为可行性声明。

## 总评

1. **F4 红已实质修复**：购买/卖边落地、判据与变异全真红、门禁与 S2 零漂移全部亲自复现。
2. **两处黄均为文档/守卫完备性**：投影守卫只钉 4/9 锚、findings 边界 3 未原地更正
   （原 false 断言仍在第 3 条原文裸露）——均为小改可修，**不阻塞**。
3. **一句话结论：可以推 S3 四笔（9706644/60ecc80/960a301/a87a730）+ docs 收尾**；
   建议收尾 commit 顺手补 P3 五锚断言与 P4 边界 3/4 原地日期注。
